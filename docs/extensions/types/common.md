# Base Extension

All extensions types share these methods:

```js
class MyExtension extends Anime {
  async getFilters() {
    return {
      genre: {
        label: "Genre",
        type: "select",
        options: [
          { label: "Action", value: "action" },
          { label: "Comedy", value: "comedy" }
        ]
      },

      tags: {
        label: "Tags",
        type: "multiselect",
        options: [
          { label: "Isekai", value: "isekai" },
          { label: "Romance", value: "romance" }
        ]
      },

      nsfw: {
        label: "NSFW",
        type: "boolean"
      }
    };
  }

  async search(query, filters, page) {
    const res = await fetch(`${this.api}/search?q=${query}&page=${page}`);
    if (!res.ok) return [];

    const data = await res.json();

    return data.map(item => ({
      id: item.slug,
      title: item.title,
      url: `${this.api}/media/${item.slug}`,
      image: item.image
    }));
  }

  async getMetadata(id) {
    const res = await fetch(`${this.api}/media/${id}`);
    if (!res.ok) throw new Error("Failed to fetch metadata");

    const data = await res.json();

    return {
      title: data.title,
      synopsis: data.synopsis ?? null,
      image: data.image ?? null,

      eps_or_chapters: data.episodes ?? null,
      rating: data.rating ?? null,
      year: data.year ?? null,

      genres: data.genres ?? [],

      anilist_id: data.mal_id ?? null,
      mal_id: data.mal_id ?? null,

      external_ids: {
        imdb: data.imdb_id
      }
    };
  }
}
```

---

### Notes

- `getFilters()` is optional
- `search()` if you do implement `getFilters()`, search method must implement it
- `getMetadata()` must return an object with at least `title`

:::tip Use AniList / MAL IDs when possible
Providing `anilist_id` or `mal_id` allows direct matching with trackers and avoids bad matches.
:::