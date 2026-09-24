import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'bitshelf',
  description: 'Local Markdown-first storage for people and agents',
  base: '/bitshelf/',
  themeConfig: {
    nav: [{ text: 'Guide', link: '/guide' }, { text: 'CLI', link: '/reference/' }],
    sidebar: [
      { text: 'Development documentation', items: [
        { text: 'Overview', link: '/' },
        { text: 'User guide', link: '/guide' },
        { text: 'JSON contract', link: '/json' },
        { text: 'Releases', link: '/releases' },
      ]},
      { text: 'Command reference', items: [
        { text: 'bs', link: '/reference/' },
        ...['init', 'shelf', 'add', 'edit', 'sync', 'list', 'search', 'show', 'open', 'context', 'validate', 'prune', 'completion'].map(name => ({ text: name, link: `/reference/${name}` })),
      ]},
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com/bswan0002/bitshelf' }],
  },
})
