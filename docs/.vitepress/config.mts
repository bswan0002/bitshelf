import { cpSync } from 'node:fs'
import { join } from 'node:path'
import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'bitshelf',
  description: 'A shared shelf for work worth keeping, used by people and agents',
  base: '/',
  cleanUrls: true,
  // Raw measurement files live beside benchmarks.md so the same relative link
  // works on GitHub, in release archives and on the site.
  buildEnd(site) {
    cpSync(join(site.srcDir, 'benchmarks'), join(site.outDir, 'benchmarks'), { recursive: true })
  },
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
    ['meta', { name: 'theme-color', content: '#f2ece0', media: '(prefers-color-scheme: light)' }],
    ['meta', { name: 'theme-color', content: '#15130f', media: '(prefers-color-scheme: dark)' }],
  ],
  markdown: {
    theme: { light: 'gruvbox-light-medium', dark: 'gruvbox-dark-medium' },
  },
  themeConfig: {
    logo: { src: '/logo.svg', alt: '' },
    search: { provider: 'local' },
    outline: { level: [2, 3], label: 'On this page' },
    docFooter: { prev: 'Previous', next: 'Next' },
    footer: {
      message: 'Development documentation (main). Use the docs bundled with your release for its exact contract.',
      copyright: '© 2026 Ben Swanson',
    },
    nav: [
      { text: 'Get started', link: '/start' },
      { text: 'Recipes', link: '/recipes/design-docs' },
      { text: 'Reference', link: '/concepts/shelves' },
      { text: 'CLI', link: '/reference/' },
      { text: 'Ben Swanson', link: 'https://benswanson.dev' },
    ],
    sidebar: [
      { text: 'Introduction', items: [
        { text: 'Why bitshelf', link: '/' },
        { text: 'Get started', link: '/start' },
        { text: 'Using with agents', link: '/agents' },
        { text: 'Extending bitshelf', link: '/concepts/extensions' },
        { text: 'Install', link: '/install' },
        { text: 'Release matching', link: '/releases' },
        { text: 'Compatibility', link: '/compatibility' },
        { text: 'Platforms', link: '/platforms' },
        { text: 'Signing and provenance', link: '/signing' },
        { text: 'Verification', link: '/verification' },
        { text: 'Scale measurements', link: '/benchmarks' },
      ]},
      { text: 'Recipes', items: [
        { text: 'Design docs ready for implementation', link: '/recipes/design-docs' },
        { text: 'Focused handoffs between sessions', link: '/recipes/handoffs' },
        { text: 'Keep code that didn’t ship', link: '/recipes/reusable-code' },
        { text: 'Archive without deleting', link: '/recipes/archive' },
        { text: 'Let scratch material expire', link: '/recipes/expiring-shelves' },
      ]},
      { text: 'Reference', items: [
        { text: 'Shelves, bits, and configuration', link: '/concepts/shelves' },
        { text: 'Saving and editing', link: '/concepts/editing' },
        { text: 'Metadata mutations', link: '/concepts/metadata' },
        { text: 'Identifiers', link: '/concepts/identifiers' },
        { text: 'Frontmatter values', link: '/concepts/frontmatter' },
        { text: 'Finding and reading', link: '/concepts/finding' },
        { text: 'Timestamps', link: '/concepts/timestamps' },
        { text: 'Timestamp fixtures', link: '/concepts/timestamp-fixtures' },
        { text: 'Moving bits', link: '/concepts/moving' },
        { text: 'Deleting bits', link: '/concepts/deleting' },
        { text: 'Expiration and pruning', link: '/concepts/pruning' },
        { text: 'Shell completion', link: '/concepts/completion' },
        { text: 'Safety and recovery', link: '/concepts/safety' },
        { text: 'JSON contract', link: '/json' },
      ]},
      { text: 'Command reference', collapsed: true, items: [
        { text: 'bs', link: '/reference/' },
        ...['init', 'shelf', 'add', 'edit', 'move', 'aliases', 'list', 'search', 'show', 'open', 'context', 'validate', 'prune', 'completion'].map(name => ({ text: name, link: `/reference/${name}` })),
      ]},
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com/bswan0002/bitshelf' }],
  },
})
