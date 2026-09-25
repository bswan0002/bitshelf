import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'bitshelf',
  description: 'A shared shelf for work worth keeping, used by people and agents',
  base: '/',
  themeConfig: {
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
        { text: 'Install', link: '/install' },
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
        { text: 'Finding and reading', link: '/concepts/finding' },
        { text: 'Timestamps and sync', link: '/concepts/timestamps' },
        { text: 'Moving and aliases', link: '/concepts/moving' },
        { text: 'Expiration and pruning', link: '/concepts/pruning' },
        { text: 'Shell completion', link: '/concepts/completion' },
        { text: 'Safety and recovery', link: '/concepts/safety' },
        { text: 'JSON contract', link: '/json' },
      ]},
      { text: 'Command reference', collapsed: true, items: [
        { text: 'bs', link: '/reference/' },
        ...['init', 'shelf', 'add', 'edit', 'move', 'aliases', 'sync', 'list', 'search', 'show', 'open', 'context', 'validate', 'prune', 'completion'].map(name => ({ text: name, link: `/reference/${name}` })),
      ]},
    ],
    socialLinks: [{ icon: 'github', link: 'https://github.com/bswan0002/bitshelf' }],
  },
})
