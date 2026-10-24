import { defineConfig } from 'vitepress'
import { tabsMarkdownPlugin } from 'vitepress-plugin-tabs'

export default defineConfig({
  title: 'ergo',
  description: 'Simple helper functions, the same in JavaScript, Python, Rust, and Go.',
  base: '/ergo/', //for now the site is published at zootella.github.io/ergo; when dns for zootella.com points at github pages, this becomes '/' and docs/public/CNAME returns with the domain in it; construction.md has the steps
  sitemap: { hostname: 'https://zootella.github.io/ergo' },
  markdown: {
    config(md) {
      md.use(tabsMarkdownPlugin) //tabs that sync by key, so choosing rust on one example chooses it everywhere on the page
    },
  },
  themeConfig: {
    nav: [{ text: 'Functions', link: '/int' }],
    sidebar: [
      {
        text: 'Functions',
        items: [
          { text: 'Int', link: '/int' },
          { text: 'Text', link: '/text' },
          { text: 'Size', link: '/size' },
          { text: 'Time', link: '/time' },
          { text: 'Tick', link: '/tick' },
          { text: 'Tag', link: '/tag' },
          { text: 'Data', link: '/data' },
          { text: 'Hash', link: '/hash' },
          { text: 'Fuji hashes', link: '/fuji' },
          { text: 'Outline', link: '/outline' },
        ],
      },
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com/zootella/ergo' }],
  },
})
