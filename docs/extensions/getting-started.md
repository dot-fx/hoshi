# Getting Started

Extensions allow you to add new content sources to the app for players and readers.

## Start Coding

Hoshi currently supports three extension types:

- [Anime](./types/anime)
- [Manga](./types/manga)
- [Novel](./types/novel)

All extension types share a common base API. See [Common](./types/common) for shared behavior and utilities.

Extensions are written in JavaScript, but run inside a sandboxed environment with a limited API (not full Node.js or browser APIs).

Check the [Sandbox APIs](./sandbox) to see what is available inside Hoshi’s sandbox.

## Installing & Debugging

To install an extension, place it inside the app’s `extensions` directory.

The base directory is resolved using Tauri:
`app.path().app_data_dir()`

Typical locations per OS:

- **Windows:**  
  `C:\Users\<username>\AppData\Roaming\com.dot-fx.hoshi\extensions`

- **Linux:**  
  `/home/<username>/.local/share/com.dot-fx.hoshi/extensions`

- **macOS:**  
  `/Users/<username>/Library/Application Support/com.dot-fx.hoshi/extensions`

::: info
On Android, the app data directory also exists but is not directly accessible due to system sandboxing. Extensions must be installed using in-app methods instead of manually placing files.
:::

### Steps

1. Create a new folder inside `extensions` using your extension name.
2. Inside that folder:
    - Add a [manifest](./extension-manifest) file
    - Add your main `.js` extension file

### Debugging

You can use `console.log` normally.

Logs are available inside the app:
**Settings → Logs**

## Marketplace & Sharing

To share your extension, host your files online (for example, in a GitHub repository).

At minimum, you need:
- The extension manifest
- The extension source files

### Remote Installation

To allow users to install your extension remotely, it must be listed in a marketplace.

There is currently no official marketplace, but you can create your own by hosting a JSON file like this:

```json
{
  "extensions": [
    {
      "id": "my-extension",
      "name": "My Extension",
      "version": "1.0.0",
      "author": "Your Name",
      "ext_type": "anime",
      "language": "en",
      "main": "main.js",
      "icon": "https://website.com/favicon.ico",
      "manifestUrl": "https://raw.githubusercontent.com/USER/REPO/refs/heads/master/.../manifest.yaml"
    }
  ]
}
```

You can then distribute the URL of this JSON file so users can add your marketplace to the app.