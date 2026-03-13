class Manga extends Base {
    async search(_)            { throw new Error("search not implemented"); }
    async getMetadata(_)       { throw new Error("getMetadata not implemented"); }
    async findChapters(_)      { throw new Error("findChapters not implemented"); }
    async findChapterPages(_)  { throw new Error("findChapterPages not implemented"); }
    
    async _search(args) {
        const results = await this.search(args);
        return this._validateSearchResults(results);
    }

    async _getMetadata(id) {
        const meta = await this.getMetadata(id);
        return this._validateMetadata(meta);
    }

    async _findChapters(id) {
        const chapters = await this.findChapters(id);
        this._assertArray(chapters, "findChapters() return value");
        chapters.forEach((ch, i) => {
            const ctx = `chapter[${i}]`;
            this._assertString(ch.id,    `${ctx}.id`);
            this._assertString(ch.title, `${ctx}.title`);
            this._assertNullableNumber(ch.number, `${ctx}.number`);
            this._assertNullableNumber(ch.index,  `${ctx}.index`);
        });
        return chapters;
    }

    async _findChapterPages(chapterId) {
        const pages = await this.findChapterPages(chapterId);
        this._assertArray(pages, "findChapterPages() return value");
        pages.forEach((p, i) => {
            const ctx = `page[${i}]`;
            this._assertString(p.url,              `${ctx}.url`);
            this._assertNullableNumber(p.index, `${ctx}.index`);
        });
        return pages;
    }
}