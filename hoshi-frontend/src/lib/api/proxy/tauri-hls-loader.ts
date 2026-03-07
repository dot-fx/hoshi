import Hls from "hls.js";
import { isTauri } from "@/api/client";
import type { ProxyParams } from "./proxy";
import { buildProxyUrl } from "./proxy";

// @ts-ignore
type LoaderConfig   = Hls.LoaderConfig;
// @ts-ignore
type LoaderCallbacks<T> = Hls.LoaderCallbacks<T>;
// @ts-ignore
type LoaderContext  = Hls.LoaderContext;
// @ts-ignore
type LoaderStats    = Hls.LoaderStats;

function extractProxyParams(url: string): ProxyParams {
    try {
        const u = new URL(url, "http://x");
        const target = u.searchParams.get("url");
        if (!target) return { url };
        return {
            url:       target,
            referer:   u.searchParams.get("referer")   ?? undefined,
            origin:    u.searchParams.get("origin")    ?? undefined,
            userAgent: u.searchParams.get("userAgent") ?? undefined,
        };
    } catch {
        return { url };
    }
}

export function createTauriLoader(): typeof Hls.DefaultConfig.loader {
    if (!isTauri()) {
        return Hls.DefaultConfig.loader;
    }

    // @ts-ignore
    return class TauriLoader implements Hls.Loader<LoaderContext> {
        private aborted = false;
        stats: LoaderStats = {
            aborted: false,
            loaded: 0,
            retry: 0,
            total: 0,
            chunkCount: 0,
            bwEstimate: 0,
            loading: { start: 0, first: 0, end: 0 },
            parsing: { start: 0, end: 0 },
            buffering: { start: 0, first: 0, end: 0 },
        };
        context!: LoaderContext;

        load(
            context: LoaderContext,
            _config: LoaderConfig,
            callbacks: LoaderCallbacks<LoaderContext>
        ): void {
            this.context = context;
            const { url } = context;
            const params = extractProxyParams(url);

            const startTime = self.performance.now();
            this.stats.loading.start = startTime;

            import("@tauri-apps/api/core").then(({ invoke }) => {
                if (this.aborted) return;

                invoke<number[]>("proxy_fetch_bytes", {
                    url:       params.url,
                    referer:   params.referer   ?? null,
                    origin:    params.origin    ?? null,
                    userAgent: params.userAgent ?? null,
                    range:     null,
                })
                    .then((raw) => {
                        if (this.aborted) return;

                        const bytes = new Uint8Array(raw);
                        const now = self.performance.now();

                        this.stats.loading.first = now;
                        this.stats.loading.end   = now;
                        this.stats.loaded = bytes.byteLength;
                        this.stats.total  = bytes.byteLength;

                        // hls.js distingue entre fragmentos binarios y playlists de texto
                        if (context.responseType === "arraybuffer") {
                            callbacks.onSuccess(
                                { url, data: bytes.buffer, code: 200 },
                                this.stats,
                                context,
                                null
                            );
                        } else {
                            const text = new TextDecoder().decode(bytes);
                            callbacks.onSuccess(
                                { url, data: text, code: 200 },
                                this.stats,
                                context,
                                null
                            );
                        }
                    })
                    .catch((err) => {
                        if (this.aborted) return;
                        callbacks.onError(
                            { code: 0, text: String(err) },
                            context,
                            null,
                            this.stats
                        );
                    });
            });
        }

        abort(): void {
            this.aborted = true;
        }

        destroy(): void {
            this.aborted = true;
        }
    };
}