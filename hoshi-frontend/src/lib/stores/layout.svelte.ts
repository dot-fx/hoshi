import type { Snippet } from 'svelte';

export const layoutState = $state({
    title: "Hoshi",
    showBack: false,
    backUrl: null as string | null,
    headerAction: undefined as Snippet | undefined,
    isMobile: false
});