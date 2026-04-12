import { goto } from '$app/navigation';
import { auth } from '@/stores/auth.svelte.js';
import { extensions } from '@/stores/extensions.svelte.js';
import { discordApi } from "@/api/discord/discord";

export async function initApp(setTouchDevice: (v: boolean) => void) {
    setTouchDevice(window.matchMedia('(pointer: coarse)').matches);

    await auth.restore();

    if (auth.isAuthenticated) {
        extensions.load();
    }
}

export function handleNavigation(pathname: string) {
    if (!auth.initialized) return;

    const isWatchparty = pathname.startsWith('/watchparty/');
    const isSetup = pathname.startsWith('/setup');

    if (!auth.user && !isSetup && !isWatchparty) {
        goto('/setup');
    } else if (auth.user && isSetup) {
        goto('/');
    }

    if (auth.user) {
        extensions.load();
    }
}

export function handleDiscordActivity(enabled: boolean, text: string) {
    if (!enabled) return;

    discordApi.setActivity({
        title: "Hoshi",
        details: text,
        isVideo: false,
        isNsfw: false
    }).catch(() => {});
}