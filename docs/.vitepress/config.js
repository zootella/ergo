import { defineConfig } from 'vitepress'
import { tabsMarkdownPlugin } from 'vitepress-plugin-tabs'

export default defineConfig({
  title: 'ergo',
  description: 'Simple helper functions, the same in JavaScript, Python, Rust, and Go.',
  sitemap: { hostname: 'https://zootella.com' }, //the site is at zootella.com, the root of this repository's custom domain, so it keeps vitepress's default base of '/'; construction.md has how the domain is set up
  cleanUrls: true, //links without .html, like /tick, which github pages answers with tick.html
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
