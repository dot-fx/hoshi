import type { FullContent, ContentType } from '@/api/content/types';
import type { ContinueItem } from '@/api/progress/types';
import { contentApi } from '@/api/content/content';
import { progressApi } from '@/api/progress/progress';
import type { CoreError } from '@/api/client';

export type MappedHomeSection = {
    trending: FullContent[];
    popular: FullContent[];
    topRated: FullContent[];
    seasonal: FullContent[];
    recentlyFinished: FullContent[];
    upcoming: FullContent[];
    topAction: FullContent[];
    topRomance: FullContent[];
    topFantasy: FullContent[];
    topScifi: FullContent[];
    topSports: FullContent[];
};

function mapSection(section: any): MappedHomeSection {
    return {
        trending: section?.trending || [],
        popular: section?.popular || [],
        topRated: section?.topRated || [],
        seasonal: section?.seasonal || [],
        upcoming: section?.upcoming || [],
        recentlyFinished: section?.recentlyFinished || [],
        topAction: section?.topAction || [],
        topRomance: section?.topRomance || [],
        topFantasy: section?.topFantasy || [],
        topScifi: section?.topScifi || [],
        topSports: section?.topSports || [],
    };
}

class HomeState {
    content = $state<Record<ContentType, MappedHomeSection | null>>({
        anime: null,
        manga: null,
        novel: null
    });

    continueItems = $state<ContinueItem[]>([]);
    loading = $state(false);
    error = $state<CoreError | null>(null);

    hasData = $derived(this.content.anime !== null);

    async load() {
        if (!this.hasData) this.loading = true;
        this.error = null;

        try {
            const [res, progRes] = await Promise.all([
                contentApi.getHome(),
                progressApi.getContinueWatching(20)
            ]);

            this.content = {
                anime: mapSection(res.anime),
                manga: mapSection(res.manga),
                novel: mapSection(res.novel)
            };
            this.continueItems = progRes.items || [];
        } catch (err) {
            console.error('Failed to load home content', err);
            if (!this.hasData) {
                this.error = err as CoreError;
            }
        } finally {
            this.loading = false;
        }
    }

    getContinueItems(mode: ContentType) {
        return this.continueItems.filter(item => item.contentType === mode);
    }

    getSection(mode: ContentType) {
        return this.content[mode];
    }
}

export const homeState = new HomeState();