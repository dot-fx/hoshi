# Manga Extension

Manga extensions extend `Manga`.

Base methods like `search()`, `getMetadata()` and `getFilters()` work the same as in Base Extension.

## Example

```js
class MyManga extends Manga {
  async findChapters(contentId) {
    return [
      {
        id: "chapter-1",
        title: "Chapter 1",
        number: 1,
        index: 0
      }
    ];
  }

  async findChapterPages(chapterId) {
    return [
      {
        url: "https://example.com/page1.jpg",
      }
    ];
  }
}
```

## `findChapters(contentId)`

- `id` is required and used in `findChapterPages()`
- `number` what user sees, decimals allowed.
- `index` is used for ordering (0 = first, no decimal)

## `findChapterPages(chapterId)`

- pages should be in reading order

---

:::details Full template
```js
class MyManga extends Manga {
    async getFilters() {
        return {};
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

    async findChapters(contentId) {
        return [{
            id: "",
            title: "",
            number: 1,
            index: 0
        }];
    }

    async findChapterPages(chapterId) {
        return [{
            url: "",
        }];
    }
}
```
:::