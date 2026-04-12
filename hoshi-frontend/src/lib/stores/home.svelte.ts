import type { ContentWithMappings, ContentType } from '@/api/content/types';
import type { ContinueItem } from '@/api/progress/types';

export type MappedHomeSection = {
    trending: ContentWithMappings[];
    seasonal: ContentWithMappings[];
    topRated: ContentWithMappings[];
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