const MCP_BASE_URL = process.env.NEXT_PUBLIC_MCP_BASE_URL;

export function resolveMcpBaseUrl(): string {
  return MCP_BASE_URL ?? (typeof window !== "undefined" ? window.location.origin : "http://127.0.0.1:8080");
}
