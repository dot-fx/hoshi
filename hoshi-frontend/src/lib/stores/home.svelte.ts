import type { FullContent, ContentType } from '@/api/content/types';
import type { ContinueItem } from '@/api/progress/types';

export type MappedHomeSection = {
    trending: FullContent[];
    seasonal: FullContent[];
    topRated: FullContent[];
};

class HomeState {
    content = $state<Record<ContentType, MappedHomeSection | null>>({
        anime: null,
        manga: null,
        novel: null
    });
    continueItems = $state<ContinueItem[]>([]);

    hasData = $derived(this.content.anime !== null);
}

export const homeState = new HomeState();