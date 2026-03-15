import { isTauri } from '@/api/client';
import { contentApi } from "@/api/content/content";
import { buildProxyUrl } from "$lib/api/proxy/proxy";
import type { PlaylistItem, VideoSource } from '@/api/watchparty/types';
import type { WatchPartySocket } from '@/api/watchparty/ws';

export function extractProxyHeaders(headers: Record<string, string> = {}) {
    return {
        referer: headers["Referer"] || headers["referer"],
        origin: headers["Origin"] || headers["origin"],
        userAgent: headers["User-Agent"] || headers["user-agent"],
    };
}

export function getProxiedVideoUrl(source?: VideoSource | null) {
    if (!source) return "";
    return buildProxyUrl({
        url: source.url,
        ...extractProxyHeaders(source.headers)
    });
}

export function getProxiedSubtitles(source?: VideoSource | null) {
    if (!source?.subtitles) return [];
    const headers = extractProxyHeaders(source.headers);
    return source.subtitles.map(sub => ({
        ...sub,
        url: buildProxyUrl({ url: sub.url, ...headers })
    }));
}

export function getBaseWsUrl() {
    if (isTauri()) return 'ws://127.0.0.1:10090';
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    return `${protocol}//${window.location.host}`;
}

export async function resolveAndEmitSource(
    currentItem: PlaylistItem,
    hostSettings: { extension: string | null; server: string | null; isDub: boolean },
    socket: WatchPartySocket
) {
    const meta = currentItem.metadata;
    if (!meta || !hostSettings.extension) return;

    try {
        const res = await contentApi.play(
            meta.contentId,
            hostSettings.extension,
            meta.unitNumber,
            {
                server: hostSettings.server ?? undefined,
                category: hostSettings.isDub ? 'dub' : undefined
            }
        );
        if (res.type === 'video') {
            const data = res.data as any;
            socket.resolveSource({
                url: data.source.url,
                headers: data.headers || {},
                subtitles: data.source.subtitles || [],
                chapters: data.source.chapters || []
            });
        }
    } catch (err) {
        console.error("[Watchparty] Error al resolver fuente:", err);
    }
}