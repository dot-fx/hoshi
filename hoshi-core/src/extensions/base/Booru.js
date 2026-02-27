class Booru extends Base {
    async search(_) { throw new Error("search not implemented"); }
    async getInfo(_) { throw new Error("getInfo not implemented"); }
    async autocomplete(_) { return []; }
}