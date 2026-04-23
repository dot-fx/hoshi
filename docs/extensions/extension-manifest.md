# Extension Manifest

Each extension must include a `manifest.yaml` file that describes how it should be loaded.

## Example

```yaml
id: my-extension
name: My Extension
version: 1.0.0
author: Your Name
icon: https://website.com/favicon.ico

main: index.js
type: anime                       # anime | manga | novel

language: en
nsfw: false
skip_default_processing: false

settings:
  - key: mirror
    type: select                  # string | boolean | select
    default: "website.com"        # string | boolean | select value
    options:
      - label: Website.com
        value: "website.com"
      - label: Website.net
        value: "website.net"
```


:::info When to use `skip_default_processing`
Enable this only if **all content provided by your extension** cannot be matched to standard databases (Anilist, MyAnimeList or Kitsu).

**Use this for:**
- Fully fan-made or unofficial sources
- Doujinshi-only sites
- Content outside typical anime/manga/novel databases
- Sources with completely non-standard naming
:::

## Settings Usage

Settings are available in your code via `__settings`:

```js
const quality = __settings.quality;
```