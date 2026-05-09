import { type as getOsType } from "@tauri-apps/plugin-os";

export interface ProxyParams {
    url: string;
    referer?: string;
    origin?: string;
    userAgent?: string;
}

export function buildTauriProxyUrl(params: ProxyParams): string {
    const osType = getOsType();
    const proxyBaseUrl =
        osType === "linux"
            ? "proxy://localhost"
            : "http://proxy.localhost/proxy";

    const query = new URLSearchParams();

    query.set("url", params.url);

    if (params.referer) {
        query.set("referer", params.referer);
    }

    if (params.origin) {
        query.set("origin", params.origin);
    }

    if (params.userAgent) {
        query.set("userAgent", params.userAgent);
    }

    return `${proxyBaseUrl}?${query.toString()}`;
}

export const proxyApi = {
    async fetch(params: ProxyParams): Promise<Blob> {
        const res = await fetch(buildTauriProxyUrl(params));

        if (!res.ok) {
            throw new Error(`Proxy error: ${res.status}`);
        }

        return res.blob();
    },
};