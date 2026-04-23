import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "hoshi",
  description: "hoshi",
  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }]
  ],

  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    logo: '/icon_round.png',
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Docs', link: '/extensions/getting-started' }
    ],

    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Introduction', link: '/guide/introduction' },
          { text: 'Installation', link: '/guide/installation' },
          { text: 'Usage', link: '/guide/usage' }
        ]
      },
      {
        text: 'Extensions',
        items: [
          { text: 'Getting Started', link: '/extensions/getting-started' },
          { text: 'Extension Manifest', link: '/extensions/extension-manifest' },
          { text: 'Sandbox APIs', link: '/extensions/sandbox' },
          {
            text: 'Types',
            items: [
              { text: 'Common', link: '/extensions/types/common' },
              { text: 'Anime', link: '/extensions/types/anime' },
              { text: 'Manga', link: '/extensions/types/manga' },
              { text: 'Novel', link: '/extensions/types/novel' }
            ]
          }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/dot-fx/hoshi' },
    ]
  }
})
