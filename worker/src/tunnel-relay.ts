import { DurableObject } from "cloudflare:workers";
import { TunnelRequest, TunnelResponse } from "./protocol";

export interface Env {
  TUNNEL_RELAY: DurableObjectNamespace;
  AUTH_TOKEN: string;
}

interface RequestLog {
  id: string;
  timestamp: number;
  method: string;
  path: string;
  status: number;
  durationMs: number;
}

export class TunnelRelay extends DurableObject {
  private resolvers = new Map<string, (res: TunnelResponse) => void>();
  private rejecters = new Map<string, (err: Error) => void>();

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);

    // Handle WebSocket connection upgrade from Tauri client
    if (url.pathname === "/connect") {
      if (request.headers.get("Upgrade") !== "websocket") {
        return new Response("Expected WebSocket connection", { status: 400 });
      }

      // Perform WebSocket upgrade
      const pair = new WebSocketPair();
      const [client, server] = Object.values(pair);

      // Associate this websocket with a tag to find it during request forwarding
      this.ctx.acceptWebSocket(server, ["client"]);

      // Close any previous client websockets to ensure we only have one active connection
      const websockets = this.ctx.getWebSockets("client");
      for (const ws of websockets) {
        if (ws !== server) {
          try {
            ws.close(1000, "Newer client connected");
          } catch (e) {
            // Ignore
          }
        }
      }

      return new Response(null, {
        status: 101,
        webSocket: client,
      });
    }

    // Handle request logs endpoint
    if (url.pathname === "/_locagate/logs") {
      const logs = (await this.ctx.storage.get<RequestLog[]>("logs")) || [];
      return new Response(JSON.stringify(logs), {
        headers: { "Content-Type": "application/json" },
      });
    }

    // Otherwise, relay the HTTP request to the connected client
    const websockets = this.ctx.getWebSockets("client");
    if (websockets.length === 0) {
      return new Response("Tunnel Offline - No client connected to LocaGate", {
        status: 502,
        headers: { "Content-Type": "text/plain" },
      });
    }

    const clientWs = websockets[0];
    const requestId = crypto.randomUUID();
    const startTime = Date.now();

    // Parse the request body if present
    let base64Body: string | null = null;
    if (request.body) {
      const buffer = await request.arrayBuffer();
      let binary = "";
      const bytes = new Uint8Array(buffer);
      for (let i = 0; i < bytes.byteLength; i++) {
        binary += String.fromCharCode(bytes[i]);
      }
      base64Body = btoa(binary);
    }

    // Serialize headers
    const headers: Record<string, string> = {};
    for (const [key, value] of request.headers.entries()) {
      headers[key] = value;
    }

    const tunnelRequest: TunnelRequest = {
      type: "request",
      id: requestId,
      method: request.method,
      path: url.pathname + url.search,
      headers,
      body: base64Body,
    };

    // Send the request to the client
    try {
      clientWs.send(JSON.stringify(tunnelRequest));
    } catch (err) {
      return new Response("Failed to send request to LocaGate client", { status: 502 });
    }

    // Wait for response from the client
    try {
      const tunnelResponse = await new Promise<TunnelResponse>((resolve, reject) => {
        const timeout = setTimeout(() => {
          this.resolvers.delete(requestId);
          this.rejecters.delete(requestId);
          reject(new Error("Gateway Timeout"));
        }, 30000); // 30s timeout

        this.resolvers.set(requestId, (res) => {
          clearTimeout(timeout);
          resolve(res);
        });

        this.rejecters.set(requestId, (err) => {
          clearTimeout(timeout);
          reject(err);
        });
      });

      // Construct HTTP response from the tunnel client's response
      const responseHeaders = new Headers();
      if (tunnelResponse.headers) {
        for (const [key, val] of Object.entries(tunnelResponse.headers)) {
          responseHeaders.set(key, val);
        }
      }

      let responseBody: Uint8Array | null = null;
      if (tunnelResponse.body) {
        const binaryString = atob(tunnelResponse.body);
        responseBody = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
          responseBody[i] = binaryString.charCodeAt(i);
        }
      }

      const durationMs = Date.now() - startTime;
      await this.logRequest({
        id: requestId,
        timestamp: startTime,
        method: request.method,
        path: url.pathname + url.search,
        status: tunnelResponse.status,
        durationMs,
      });

      return new Response(responseBody, {
        status: tunnelResponse.status,
        headers: responseHeaders,
      });

    } catch (err: any) {
      const durationMs = Date.now() - startTime;
      const status = err.message === "Gateway Timeout" ? 54 : 502;
      await this.logRequest({
        id: requestId,
        timestamp: startTime,
        method: request.method,
        path: url.pathname + url.search,
        status,
        durationMs,
      });

      return new Response(err.message || "Bad Gateway", { status });
    }
  }

  // Hibernation WebSocket API handlers
  async webSocketMessage(ws: WebSocket, message: string | ArrayBuffer) {
    if (typeof message !== "string") return;

    try {
      const data = JSON.parse(message);
      if (data.type === "response" && data.id) {
        const resolver = this.resolvers.get(data.id);
        if (resolver) {
          resolver(data);
          this.resolvers.delete(data.id);
          this.rejecters.delete(data.id);
        }
      }
    } catch (e) {
      // Ignore invalid JSON or protocol violations
    }
  }

  async webSocketClose(ws: WebSocket, code: number, reason: string, wasClean: boolean) {
    this.cleanPendingRequests(new Error("Client connection closed"));
  }

  async webSocketError(ws: WebSocket, error: any) {
    this.cleanPendingRequests(new Error("Client connection error"));
  }

  private cleanPendingRequests(err: Error) {
    for (const rejecter of this.rejecters.values()) {
      rejecter(err);
    }
    this.resolvers.clear();
    this.rejecters.clear();
  }

  private async logRequest(log: RequestLog) {
    try {
      const logs = (await this.ctx.storage.get<RequestLog[]>("logs")) || [];
      logs.unshift(log);
      if (logs.length > 100) {
        logs.pop();
      }
      await this.ctx.storage.put("logs", logs);
    } catch (e) {
      // Ignore storage logging failures
    }
  }
}
