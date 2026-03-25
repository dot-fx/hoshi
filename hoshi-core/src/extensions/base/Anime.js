class Anime extends Base {
    async search(_)                { throw new Error("search not implemented"); }
    async getMetadata(_)           { throw new Error("getMetadata not implemented"); }
    async findEpisodes(_)          { throw new Error("findEpisodes not implemented"); }
    async findEpisodeServer(_, __, ___) { throw new Error("findEpisodeServer not implemented"); }

    async _search(args) {
        const results = await this.search(args);
        return this._validateSearchResults(results);
    }

    async _getMetadata(id) {
        const meta = await this.getMetadata(id);
        return this._validateMetadata(meta);
    }

    async _findEpisodes(id) {
        const episodes = await this.findEpisodes(id);
        this._assertArray(episodes, "findEpisodes() return value");
        episodes.forEach((ep, i) => {
            const ctx = `episode[${i}]`;
            this._assertString(ep.id,    `${ctx}.id`);
            this._assertNullableNumber(ep.number, `${ctx}.number`);
            this._assertNullableString(ep.title,  `${ctx}.title`);
            this._assertNullableString(ep.url,    `${ctx}.url`);
            this._assertNullableString(ep.image,  `${ctx}.image`);
        });
        return episodes;
    }

    async _findEpisodeServer(episodeId, server, category) {
        const result = await this.findEpisodeServer(episodeId, server, category);
        if (typeof result !== "object" || result === null) {
            throw new Error(`[${this.constructor.name}] findEpisodeServer() must return an object`);
        }
        if (typeof result.headers !== "object" || result.headers === null) {
            throw new Error(`[${this.constructor.name}] findEpisodeServer().headers must be an object`);
        }
        if (typeof result.source !== "object" || result.source === null) {
            throw new Error(`[${this.constructor.name}] findEpisodeServer().source must be an object`);
        }
        this._assertString(result.source.url, "findEpisodeServer().source.url");
        this._assertArray(result.source.subtitles, "findEpisodeServer().source.subtitles");
        this._assertArray(result.source.chapters,  "findEpisodeServer().source.chapters");
        return result;
    }
}