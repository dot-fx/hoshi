import { isTauri } from "@/api/client";

export interface ProxyParams {
    url: string;
    referer?: string;
    origin?: string;
    userAgent?: string;
}

export function buildProxyUrl(params: ProxyParams): string {
    const query = new URLSearchParams();
    query.set("url", params.url);
    if (params.referer)   query.set("referer",   params.referer);
    if (params.origin)    query.set("origin",    params.origin);
    if (params.userAgent) query.set("userAgent", params.userAgent);
    return `/api/proxy?${query.toString()}`;
}

export const proxyApi = {

    async fetch(params: ProxyParams): Promise<Blob> {
        if (isTauri()) {
            const { invoke } = await import("@tauri-apps/api/core");
            const raw = await invoke<number[]>("proxy_fetch_bytes", {
                url:       params.url,
                referer:   params.referer   ?? null,
                origin:    params.origin    ?? null,
                userAgent: params.userAgent ?? null,
                range:     null,
            });
            return new Blob([new Uint8Array(raw)]);
        }

        const res = await fetch(buildProxyUrl(params));
        if (!res.ok) throw new Error(`Proxy error: ${res.status}`);
        return res.blob();
    },
};