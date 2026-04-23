# Anime Extension

Anime extensions extend `Anime` and add streaming-related methods.

Base methods like `search()`, `getMetadata()` and `getFilters()` work the same as in Base example.

## Example

```js
class MyAnime extends Anime {
    getStreamingSettings() {
        return {
            episodeServers: ["HLS"],
            supportsDub: true
        };
    }

    async findEpisodes(contentId) {
        return [
            {
                id: "naruto$1",
                number: 1,
                title: "Episode 1",
                url: null,
                image: null
            }
        ];
    }

    async findEpisodeServer(episodeId, server, category = "sub") {
        return {
            headers: {},
            source: {
                url: "https://example.com/stream.m3u8",
                subtitles: [
                    {
                        id: "en",
                        url: "https://example.com/subs/en.vtt",
                        language: "English",
                        is_default: true
                    }
                ],
                chapters: [
                    {
                        start: 0,
                        end: 90,
                        title: "Opening"
                    }
                ]
            }
        };
    }
}
```

## `getStreamingSettings()`

- `supportsDub` →
    - `true` → separate sub/dub streams
    - `false` → single stream with multiple audio tracks

## `findEpisodes(contentId)`

Url and image attributes in return are optional and not used by the app for the moment.

- `id` is required and used in `findEpisodeServer()`

## `findEpisodeServer(episodeId, server, category)`

headers, subtitles and chapters are optional

---

:::details Full template
```js
class MyAnime extends Anime {
    api = "";

    async getFilters() {
        return {};
    }

    getStreamingSettings() {
        return {
            episodeServers: ["HLS"],
            supportsDub: false
        };
    }

    async search(query, filters, page) {
        return [{
            id: "",
            title: "",
            image: null,
            url: null,
            nsfw: false
        }];
    }

    async getMetadata(id) {
        return {
            title: "",
            synopsis: null,
            image: null,

            eps_or_chapters: null,
            rating: null,
            year: null,

            genres: [],

            nsfw: false,

            anilist_id: null,
            mal_id: null,

            external_ids: {}
        };
    }

    async findEpisodes(contentId) {
        return [{
            id: "",
            number: 1,
            title: "",
            url: null,
            image: null
        }];
    }

    async findEpisodeServer(episodeId, server, category = "sub") {
        return {
            headers: {},

            source: {
                url: "",

                subtitles: [],
                chapters: []
            }
        };
    }
}
```
:::