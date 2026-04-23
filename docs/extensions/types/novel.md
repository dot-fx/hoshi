# Novel Extension

Novel extensions extend `Novel`.

Base methods like `search()`, `getMetadata()` and `getFilters()` work the same as in Base Extension.
The method `findChapters` work the same as in Manga Extension. 

## Example

```js
class MyNovel extends Novel {
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
        return "<p>This is chapter content</p>";
    }
}
```

## `findChapterPages(chapterId)`

- returns HTML string
- should contain the full chapter content

---

:::details Full template
```js
class MyNovel extends Novel {
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
        return "";
    }
}
```
:::