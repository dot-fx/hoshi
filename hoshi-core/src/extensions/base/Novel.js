class Novel extends Base {
    async findChapterPages(_) {
        throw new Error("findChapterPages must return HTML string");
    }
}