import type { ContentWithMappings, ContentType } from '@/api/content/types';
import type { ContinueItem } from '@/api/progress/types';

// Definimos el tipo exacto que usará la UI
export type MappedHomeSection = {
    trending: ContentWithMappings[];
    seasonal: ContentWithMappings[];
    topRated: ContentWithMappings[];
};

class HomeState {
    // Ahora el estado guarda los datos ya mapeados (MappedHomeSection)
    content = $state<Record<ContentType, MappedHomeSection | null>>({
        anime: null,
        manga: null,
        novel: null
    });
    continueItems = $state<ContinueItem[]>([]);

    // Marcamos si ya tenemos una carga inicial exitosa basándonos en si hay datos mapeados
    hasData = $derived(this.content.anime !== null);
}

export const homeState = new HomeState();