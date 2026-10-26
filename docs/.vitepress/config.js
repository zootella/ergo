import { defineConfig } from 'vitepress'
import { tabsMarkdownPlugin } from 'vitepress-plugin-tabs'

export default defineConfig({
  title: 'zootella ergo', //after each page's title in the browser tab, like "Tick | zootella ergo"
  description: 'Simple helper functions, the same in JavaScript, Python, Rust, and Go.',
  sitemap: { hostname: 'https://zootella.com' }, //the site is at zootella.com, the root of this repository's custom domain, so it keeps vitepress's default base of '/'; construction.md has how the domain is set up
  cleanUrls: true, //links without .html, like /tick, which github pages answers with tick.html
  markdown: {
    config(md) {
      md.use(tabsMarkdownPlugin) //tabs that sync by key, so choosing rust on one example chooses it everywhere on the page
    },
  },
  themeConfig: {
    siteTitle: '<span class="site-title">zootella</span>', //the header's upper left, which links home, and which custom.css hides on the home page
    nav: [{ text: '<span class="nav-zootella">zootella </span>ergo', link: '/int' }], //the header's upper right, into the documentation, which reads "zootella ergo" on the home page and "ergo" everywhere else
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
