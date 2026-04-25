import Hls from "hls.js";
import { isTauri } from "@/api/client";
import type { ProxyParams } from "./proxy";

// @ts-ignore
type LoaderConfig       = Hls.LoaderConfig;
// @ts-ignore
type LoaderCallbacks<T> = Hls.LoaderCallbacks<T>;
// @ts-ignore
type LoaderContext      = Hls.LoaderContext;
// @ts-ignore
type LoaderStats        = Hls.LoaderStats;

/**
 * Converts a proxied /api/proxy?url=... URL into a proxy://localhost?url=...
 * URL that hits the Tauri custom URI scheme handler directly — no IPC,
 * no JSON serialization, real streaming to the WebView.
 */
function toProxySchemeUrl(url: string): string {
    try {
        const u = new URL(url, "http://x");
        const target = u.searchParams.get("url");
        if (!target) {
            // Not a proxied URL — pass through as-is via proxy scheme
            return `proxy://localhost?url=${encodeURIComponent(url)}`;
        }
        // Rebuild as proxy:// preserving all params
        const out = new URLSearchParams();
        out.set("url", target);
        const referer   = u.searchParams.get("referer");
        const origin    = u.searchParams.get("origin");
        const userAgent = u.searchParams.get("userAgent");
        if (referer)   out.set("referer",   referer);
        if (origin)    out.set("origin",    origin);
        if (userAgent) out.set("userAgent", userAgent);
        return `proxy://localhost?${out.toString()}`;
    } catch {
        return `proxy://localhost?url=${encodeURIComponent(url)}`;
    }
}

function makeStats(): LoaderStats {
    return {
        aborted:    false,
        loaded:     0,
        retry:      0,
        total:      0,
        chunkCount: 0,
        bwEstimate: 0,
        loading:    { start: 0, first: 0, end: 0 },
        parsing:    { start: 0, end: 0 },
        buffering:  { start: 0, first: 0, end: 0 },
    };
}

export function createTauriLoader(): typeof Hls.DefaultConfig.loader {
    if (!isTauri()) return Hls.DefaultConfig.loader;

    // @ts-ignore
    return class TauriLoader implements Hls.Loader<LoaderContext> {
        private controller = new AbortController();
        private loadId     = 0;
        stats: LoaderStats  = makeStats();
        context!: LoaderContext;

        load(
            context: LoaderContext,
            _config: LoaderConfig,
            callbacks: LoaderCallbacks<LoaderContext>
        ): void {
            this.context    = context;
            this.stats      = makeStats();
            // Each new load gets a fresh AbortController so we can cancel it precisely
            this.controller = new AbortController();
            const myId      = ++this.loadId;

            const { url, responseType } = context;
            const proxyUrl = toProxySchemeUrl(url);
            const t0       = performance.now();
            this.stats.loading.start = t0;

            fetch(proxyUrl, { signal: this.controller.signal })
                .then(async (res) => {
                    if (myId !== this.loadId) return; // superseded by a newer load

                    if (!res.ok) {
                        callbacks.onError(
                            { code: res.status, text: res.statusText },
                            context,
                            null,
                            this.stats
                        );
                        return;
                    }

                    const t1 = performance.now();
                    this.stats.loading.first = t1;

                    const buffer = await res.arrayBuffer();
                    if (myId !== this.loadId) return;

                    const t2   = performance.now();
                    const size = buffer.byteLength;

                    this.stats.loading.end = t2;
                    this.stats.loaded      = size;
                    this.stats.total       = size;
                    this.stats.chunkCount  = 1;
                    this.stats.bwEstimate  = (size * 8 * 1000) / Math.max(t2 - t0, 1);

                    if (responseType === "arraybuffer") {
                        callbacks.onSuccess(
                            { url, data: buffer, code: res.status },
                            this.stats,
                            context,
                            null
                        );
                    } else {
                        callbacks.onSuccess(
                            { url, data: new TextDecoder().decode(buffer), code: res.status },
                            this.stats,
                            context,
                            null
                        );
                    }
                })
                .catch((err) => {
                    if (myId !== this.loadId) return;
                    if (err.name === "AbortError") return; // intentional abort, HLS handles it
                    callbacks.onError(
                        { code: 0, text: String(err) },
                        context,
                        null,
                        this.stats
                    );
                });
        }

        abort(): void {
            this.loadId++;
            this.stats.aborted = true;
            this.controller.abort();
        }

        destroy(): void {
            this.loadId++;
            this.controller.abort();
        }
    };
}