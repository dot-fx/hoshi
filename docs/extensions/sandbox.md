# Sandbox APIs

The sandbox provides a browser-like environment. Common web APIs such as fetch, URL, URLSearchParams, atob, btoa, console, and timers are available with mostly familiar behavior.

However, this is not a full browser or Node.js environment. Some features are limited or unavailable. See below for exact behavior and differences.

:::warning Timers don’t actually wait
`setTimeout` and `setInterval` are provided for compatibility, but they **do not delay execution**. The callback runs immediately regardless of the time value.
:::

## `__settings`

A frozen object containing the settings your extension was called with. Read-only.

```js
const lang = __settings.language ?? "en";
const nsfw = __settings.nsfw     ?? false;
```

Settings are defined by whoever invokes your extension. Use them to make your extension configurable, preferred language, content filters, region, etc.

:::info Where do these come from?
Settings are defined in your extension manifest and injected into the sandbox at runtime.
:::

## `parseHTML(html)`

Parse an HTML string and query elements from it, similar to jQuery or `document.querySelector`.

```js
const $ = parseHTML(htmlString);
const items = $("ul.results li");
```

Returns a list of matched elements. Each element has:

```js
el.text()        // inner text content
el.html()        // inner HTML
el.outer()       // full outer HTML
el.attr("href")  // value of an attribute
el.find("span")  // query within this element
```

The list itself also has shorthand methods:

```js
$("h1").text()       // combined text of all matches
$("h1").html()       // inner HTML of the first match
$("a").attr("href")  // attribute of the first match
```

**Example: scraping a list:**

```js
const res  = await fetch("https://example.com/archive");
const html = await res.text();
const $    = parseHTML(html);

const links = $("article a.title").map(el => ({
    title: el.text(),
    url:   el.attr("href"),
}));
```

## `headless`

A headless browser for pages that require JavaScript to load content (e.g. SPAs, infinite scroll, client-rendered sites).

Use this when `fetch()` returns incomplete or empty HTML.


```js
if (!headless.available) {
    throw new Error("Headless browser is not available");
}

const result = await headless.fetch("https://example.com/page");
```

:::tip When to use headless
Use `headless` when `fetch()` doesn’t return the data you need (e.g. the site relies on JavaScript to render content).
:::

---

### `waitFor`

```js
"dom_ready"      // default
"network_idle"   // wait for requests to finish
".selector"      // wait for element to appear (recommended)
```

Use a selector whenever possible—it’s the most reliable.

---

### Example

```js
const result = await headless.fetch("https://example.com/app", {
    waitFor: ".results-loaded",
    block: ["images", "fonts"],

    javascript: `
    window.scrollTo(0, document.body.scrollHeight);
  `,

    capture: ["api/data"],

    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ key: "value" }),

    timeoutMs: 20000
});
```

## What's not available

- No `localStorage`, `indexedDB`, or any storage
- No `WebSocket` or `XMLHttpRequest`
- No Node.js built-ins (`fs`, `path`, `require`, etc.)
- No access to the host machine or filesystem