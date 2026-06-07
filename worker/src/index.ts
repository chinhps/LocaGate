import { TunnelRelay, Env } from "./tunnel-relay";

export { TunnelRelay };

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);

    // 1. Health check
    if (url.pathname === "/_locagate/health") {
      return new Response("OK", { status: 200 });
    }

    // 2. Tauri Client connection upgrade
    if (url.pathname === "/connect") {
      const tunnelId = url.searchParams.get("id");
      const token = url.searchParams.get("token");

      if (!tunnelId) {
        return new Response("Missing tunnel id", { status: 400 });
      }

      // If AUTH_TOKEN is set, validate it
      if (env.AUTH_TOKEN && token !== env.AUTH_TOKEN) {
        return new Response("Unauthorized", { status: 401 });
      }

      const id = env.TUNNEL_RELAY.idFromName(tunnelId);
      const stub = env.TUNNEL_RELAY.get(id);
      return stub.fetch(request);
    }

    // 3. Request logs (for Tauri dashboard)
    if (url.pathname === "/_locagate/logs") {
      const tunnelId = url.searchParams.get("id");
      const token = url.searchParams.get("token");

      if (!tunnelId) {
        return new Response("Missing tunnel id", { status: 400 });
      }

      if (env.AUTH_TOKEN && token !== env.AUTH_TOKEN) {
        return new Response("Unauthorized", { status: 401 });
      }

      const id = env.TUNNEL_RELAY.idFromName(tunnelId);
      const stub = env.TUNNEL_RELAY.get(id);
      return stub.fetch(request);
    }

    // 4. Resolve Tunnel ID from Subdomain or Path
    let tunnelId: string | null = null;
    let host = url.hostname;
    
    // Support testing and local development with custom Host headers
    if (host === "localhost" || host === "127.0.0.1") {
      const hostHeader = request.headers.get("Host");
      if (hostHeader) {
        host = hostHeader.split(":")[0];
      }
    }


    // Check Path-based routing first: /t/tunnel-id/path
    if (url.pathname.startsWith("/t/")) {
      const parts = url.pathname.split("/");
      if (parts.length >= 3) {
        tunnelId = parts[2];
        // Rewrite the URL path to strip the /t/tunnel-id prefix
        url.pathname = "/" + parts.slice(3).join("/");
      }
    } else {
      // Check Subdomain-based routing
      const labels = host.split(".");
      if (labels.length > 2) {
        if (host.endsWith(".workers.dev")) {
          // e.g. mytunnel.locagate.workers.dev (if workers.dev wildcard was somehow routed)
          if (labels.length > 3) {
            tunnelId = labels[0];
          }
        } else {
          // e.g. mytunnel.locagate.dev -> mytunnel
          tunnelId = labels[0];
        }
      }
    }

    if (!tunnelId) {
      return new Response(
        "LocaGate Relay - Welcome! Please use a valid tunnel subdomain or path prefix (e.g. /t/tunnel-id/)",
        { status: 200, headers: { "Content-Type": "text/plain" } }
      );
    }

    // Forward the request to the Durable Object matching the tunnel ID
    const doId = env.TUNNEL_RELAY.idFromName(tunnelId);
    const stub = env.TUNNEL_RELAY.get(doId);

    // Reconstruct the request to point to the DO with rewritten path
    const modifiedRequest = new Request(url.toString(), {
      method: request.method,
      headers: request.headers,
      body: request.body,
      redirect: "manual",
    });

    return stub.fetch(modifiedRequest);
  },
};
