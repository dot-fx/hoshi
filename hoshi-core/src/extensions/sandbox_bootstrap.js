"use strict";

globalThis.console = {
    _fmt: (args) => args.map(a =>
        (a !== null && typeof a === "object") ? JSON.stringify(a) : String(a)
    ).join(" "),
    log:   function(...args) { __native_log("[LOG] "   + console._fmt(args)); },
    info:  function(...args) { __native_log("[INFO] "  + console._fmt(args)); },
    warn:  function(...args) { __native_log("[WARN] "  + console._fmt(args)); },
    error: function(...args) { __native_log("[ERROR] " + console._fmt(args)); },
    debug: function(...args) { __native_log("[DEBUG] " + console._fmt(args)); },
};

globalThis.fetch = async function(url, options) {
    if (typeof url !== "string" || url.length === 0) {
        throw new TypeError("fetch: url must be a non-empty string");
    }

    options = options || {};

    const method      = (options.method || "GET").toUpperCase();
    const headersObj  = options.headers || {};
    const headersJson = JSON.stringify(headersObj);
    const body = (options.body !== undefined && options.body !== null)
        ? String(options.body)
        : "";

    const rawJson = __native_fetch(url, method, headersJson, body);
    const raw     = JSON.parse(rawJson);

    if (raw.error) {
        throw new TypeError("fetch failed: " + raw.error);
    }

    const responseBody = raw.body;
    return {
        ok:     raw.ok,
        status: raw.status,
        url:    url,
        headers: {
            get: (_name) => null,
            has: (_name) => false,
        },
        text: async function() { return responseBody; },
        json: async function() {
            try {
                return JSON.parse(responseBody);
            } catch (e) {
                throw new SyntaxError("fetch response is not valid JSON: " + e.message);
            }
        },
        arrayBuffer: async function() {
            const arr = new Uint8Array(responseBody.length);
            for (let i = 0; i < responseBody.length; i++) {
                arr[i] = responseBody.charCodeAt(i) & 0xFF;
            }
            return arr.buffer;
        },
    };
};

globalThis.URL = class URL {
    constructor(input, base) {
        let full = input;
        if (base) {
            if (/^https?:\/\//i.test(input)) {
                full = input;
            } else {
                const b = String(base).replace(/\/$/, "");
                full = input.startsWith("/")
                    ? b.replace(/(https?:\/\/[^/]+).*/, "$1") + input
                    : b + "/" + input.replace(/^\.\//, "");
            }
        }

        const m = String(full).match(
            /^(https?):\/\/([^/?#]+)([^?#]*)(\?[^#]*)?(#.*)?$/i
        );
        if (!m) throw new TypeError("Invalid URL: " + full);

        this.protocol = m[1].toLowerCase() + ":";
        this.host     = m[2];
        this.hostname = m[2].split(":")[0];
        this.port     = m[2].includes(":") ? m[2].split(":")[1] : "";
        this.pathname = m[3] || "/";
        this.search   = m[4] || "";
        this.hash     = m[5] || "";
        this.origin   = this.protocol + "//" + this.host;
        this.href     = full;
    }

    toString()     { return this.href; }
    toJSON()       { return this.href; }

    get searchParams() {
        return new URLSearchParams(this.search);
    }
};

globalThis.parseHTML = function(html) {
    return function $(selector) {
        const rawJson = __native_html_query(html, selector);
        const raw = JSON.parse(rawJson);
        if (raw.error) throw new Error(raw.error);

        const wrap = (item) => ({
            text:  ()    => item.text,
            html:  ()    => item.html,
            outer: ()    => item.outer,
            attr:  (name) => item.attrs[name] ?? null,
            find:  (sel)  => parseHTML(item.html)(sel),
            _raw: item,
        });

        const results = raw.map(wrap);
        results.length = raw.length;
        return results;
    };
};

globalThis.URLSearchParams = class URLSearchParams {
    constructor(init) {
        this._map = new Map();
        if (typeof init === "string") {
            init.replace(/^\?/, "").split("&").forEach(pair => {
                if (!pair) return;
                const idx = pair.indexOf("=");
                const k   = decodeURIComponent(idx < 0 ? pair : pair.slice(0, idx));
                const v   = decodeURIComponent(idx < 0 ? ""   : pair.slice(idx + 1));
                this._map.set(k, v);
            });
        } else if (init && typeof init === "object") {
            Object.entries(init).forEach(([k, v]) => this._map.set(k, String(v)));
        }
    }

    get(k)      { return this._map.has(k) ? this._map.get(k) : null; }
    getAll(k)   { return this._map.has(k) ? [this._map.get(k)] : []; }
    has(k)      { return this._map.has(k); }
    set(k, v)   { this._map.set(k, String(v)); }
    append(k,v) { this._map.set(k, String(v)); }
    delete(k)   { this._map.delete(k); }
    keys()      { return this._map.keys(); }
    values()    { return this._map.values(); }
    entries()   { return this._map.entries(); }

    toString() {
        const parts = [];
        this._map.forEach((v, k) =>
            parts.push(encodeURIComponent(k) + "=" + encodeURIComponent(v))
        );
        return parts.join("&");
    }
};

(function() {
    const CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    globalThis.btoa = function(str) {
        let out = "", i = 0, bits = 0, acc = 0;
        str = String(str);
        for (; i < str.length; i++) {
            acc  = (acc << 8) | (str.charCodeAt(i) & 0xFF);
            bits += 8;
            while (bits >= 6) {
                bits -= 6;
                out += CHARS[(acc >> bits) & 0x3F];
            }
        }
        if (bits > 0) out += CHARS[(acc << (6 - bits)) & 0x3F];
        while (out.length % 4) out += "=";
        return out;
    };

    globalThis.atob = function(b64) {
        b64 = String(b64).replace(/[^A-Za-z0-9+/]/g, "");
        let out = "", bits = 0, acc = 0;
        for (let i = 0; i < b64.length; i++) {
            const idx = CHARS.indexOf(b64[i]);
            if (idx < 0) continue;
            acc  = (acc << 6) | idx;
            bits += 6;
            if (bits >= 8) {
                bits -= 8;
                out += String.fromCharCode((acc >> bits) & 0xFF);
            }
        }
        return out;
    };
})();

if (typeof globalThis.File === "undefined") {
    globalThis.File = class File {
        constructor(bits, name, opts) {
            this.name = name;
            this.size = 0;
            this.type = (opts && opts.type) || "";
        }
    };
}

if (typeof globalThis.FormData === "undefined") {
    globalThis.FormData = class FormData {
        constructor() { this._data = {}; }
        append(k, v) { this._data[k] = v; }
        get(k)       { return this._data[k] || null; }
        has(k)       { return k in this._data; }
    };
}

if (typeof setTimeout === "undefined") {
    globalThis.setTimeout  = (fn, _ms, ...args) => { fn(...args); return 0; };
    globalThis.clearTimeout  = (_id) => {};
    globalThis.setInterval   = (fn, _ms, ...args) => { fn(...args); return 0; };
    globalThis.clearInterval = (_id) => {};
}