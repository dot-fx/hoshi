type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

function isTauri(): boolean {
    return typeof window !== "undefined" && "__TAURI__" in window;
}

function buildUrl(path: string, params?: object): string {
    if (!params) return `/api/${path}`;

    const query = Object.entries(params as Record<string, unknown>)
        .filter(([, value]) => value !== undefined && value !== null)
        .map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`)
        .join("&");

    return query ? `/api/${path}?${query}` : `/api/${path}`;
}

export async function api<T>(
    path: string,
    options?: {
        method?: HttpMethod;
        body?: unknown;
        params?: object;
    }
): Promise<T> {
    const method = options?.method ?? "GET";

    if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        return invoke<T>(path, options?.body as Record<string, unknown>);
    }

    const url = buildUrl(path, options?.params);

    const res = await fetch(url, {
        method,
        headers: {
            "Content-Type": "application/json",
        },
        credentials: 'include',
        body: method === "GET" ? undefined : JSON.stringify(options?.body),
    });

    if (!res.ok) {
        const text = await res.text();
        throw new Error(text || "API error");
    }

    return res.json();
}