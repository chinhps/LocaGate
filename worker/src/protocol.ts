export interface TunnelRequest {
  type: "request";
  id: string; // UUID
  method: string;
  path: string;
  headers: Record<string, string>;
  body: string | null;
}

export interface TunnelResponse {
  type: "response";
  id: string;
  status: number;
  headers: Record<string, string>;
  body: string | null;
}
