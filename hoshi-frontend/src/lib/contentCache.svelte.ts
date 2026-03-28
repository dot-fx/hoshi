class ContentCache {
    entries = $state<Map<string, any>>(new Map());
    maxSize = 15;

    get(cid: string) {
        const data = this.entries.get(cid);
        if (data) {
            this.entries.delete(cid);
            this.entries.set(cid, data);
        }
        return data;
    }

    set(cid: string, data: any) {
        if (this.entries.size >= this.maxSize && !this.entries.has(cid)) {
            const oldestKey = this.entries.keys().next().value;
            if (oldestKey) {
                this.entries.delete(oldestKey);
            }
        }
        this.entries.set(cid, data);
    }

    has(cid: string) {
        return this.entries.has(cid);
    }
}

export const contentCache = new ContentCache();