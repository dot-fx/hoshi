<script lang="ts">
    import { Button } from '@/components/ui/button';
    import { Input } from '@/components/ui/input';
    import { Send } from 'lucide-svelte';
    import type { ChatMessage } from '@/api/watchparty/types';
    import { tick } from 'svelte';
    import * as Avatar from "$lib/components/ui/avatar";
    import { i18n } from "@/i18n/index.svelte.js";

    let {
        chatHistory = [],
        onSendMessage
    }: {
        chatHistory: ChatMessage[];
        onSendMessage: (text: string) => void;
    } = $props();

    let currentMessage = $state('');
    let chatContainer: HTMLElement | null = $state(null);

    function getInitials(name: string) {
        if (!name) return '??';
        return name
            .split(' ')
            .map(n => n[0])
            .join('')
            .toUpperCase()
            .slice(0, 2);
    }

    $effect(() => {
        if (chatHistory.length > 0) {
            tick().then(() => {
                if (chatContainer) {
                    chatContainer.scrollTop = chatContainer.scrollHeight;
                }
            });
        }
    });

    function handleSubmit(e: Event) {
        e.preventDefault();
        const text = currentMessage.trim();
        if (!text) return;

        onSendMessage(text);
        currentMessage = '';
    }
</script>

<div class="flex flex-col h-full w-full bg-card">
    <div
            bind:this={chatContainer}
            class="flex-1 overflow-y-auto p-4 flex flex-col gap-4 scroll-smooth"
    >
        {#if chatHistory.length === 0}
            <div class="h-full flex items-center justify-center text-center px-4 text-muted-foreground text-sm font-medium">
                {i18n.t('watchparty.chat_empty')}
            </div>
        {:else}
            {#each chatHistory as msg (msg.id)}
                <div class="flex gap-3 items-start">
                    <Avatar.Root class="w-8 h-8 shrink-0 ring-1 ring-border/30 shadow-sm">
                        <Avatar.Image
                                src={msg.avatarUrl || undefined}
                                alt={msg.displayName}
                                class="object-cover"
                        />
                        <Avatar.Fallback class="bg-primary/10 text-primary text-xs font-bold">
                            {getInitials(msg.displayName)}
                        </Avatar.Fallback>
                    </Avatar.Root>

                    <div class="flex flex-col mt-0.5">
                        <span class="font-bold text-xs text-primary mb-0.5">{msg.displayName}</span>
                        <span class="text-sm text-foreground/90 break-words leading-snug">{msg.text}</span>
                    </div>
                </div>
            {/each}
        {/if}
    </div>

    <form onsubmit={handleSubmit} class="p-3 border-t border-border/40 flex gap-2 bg-muted/10">
        <Input
                bind:value={currentMessage}
                placeholder={i18n.t('watchparty.chat_placeholder')}
                class="h-11 flex-1 bg-muted/30 rounded-xl px-4 py-2 text-sm focus-visible:ring-1 focus-visible:ring-primary/50"
        />
        <Button
                type="submit"
                size="icon"
                class="h-11 w-11 rounded-xl shrink-0"
                disabled={!currentMessage.trim()}
        >
            <Send class="w-4 h-4" />
        </Button>
    </form>
</div>