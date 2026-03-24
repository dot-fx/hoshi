import type { ContentWithMappings, ContentType, SearchTracker } from "$lib/api/content/types";

class SearchState {
    query = $state("");
    contentType = $state<ContentType>("anime");
    searchMode = $state<"database" | "extension">("database");
    selectedExtension = $state<string>("");

    dbStatus = $state<string>("");
    dbGenre = $state<string>("");
    dbFormat = $state<string>("");
    dbNsfw = $state<boolean>(false);
    dbTracker = $state<SearchTracker>("anilist");

    extFilterValues = $state<Record<string, any>>({});

    results = $state<ContentWithMappings[]>([]);
    hasSearched = $state(false);

    hasData = $derived(this.results.length > 0);
}

export const searchState = new SearchState();