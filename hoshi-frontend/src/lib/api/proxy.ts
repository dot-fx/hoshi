export interface ProxyParams {
    url: string;
    referer?: string;
    origin?: string;
    userAgent?: string;
}

const isAndroid = /Android/i.test(navigator.userAgent);

export function buildTauriProxyUrl(params: ProxyParams): string {
    const query = new URLSearchParams();
    query.set("url", params.url);
    if (params.referer)   query.set("referer",   params.referer);
    if (params.origin)    query.set("origin",    params.origin);
    if (params.userAgent) query.set("userAgent", params.userAgent);

    if (isAndroid) {
        return `http://proxy.localhost/proxy?${query.toString()}`;
    }
    return `proxy://localhost?${query.toString()}`;
}

export const proxyApi = {
    async fetch(params: ProxyParams): Promise<Blob> {
        const res = await fetch(buildTauriProxyUrl(params));
        if (!res.ok) throw new Error(`Proxy error: ${res.status}`);
        return res.blob();
    },
};