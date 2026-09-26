import { h } from 'vue'
import type { Theme } from 'vitepress'
import DefaultTheme from 'vitepress/theme'

import '@fontsource/ibm-plex-sans/latin-400.css'
import '@fontsource/ibm-plex-sans/latin-400-italic.css'
import '@fontsource/ibm-plex-sans/latin-500.css'
import '@fontsource/ibm-plex-sans/latin-600.css'
import '@fontsource/ibm-plex-mono/latin-400.css'
import '@fontsource/ibm-plex-mono/latin-500.css'
import '@fontsource/ibm-plex-mono/latin-600.css'

import './style.css'
import HomePage from './components/HomePage.vue'

export default {
  extends: DefaultTheme,
  Layout: () => h(DefaultTheme.Layout, null, {
    'layout-top': () => h('div', { class: 'development-warning', role: 'note' }, [
      h('span', { class: 'tag' }, 'Prototype'),
      h('span', 'Early development. Use bitshelf only with disposable data.'),
    ]),
  }),
  enhanceApp({ app }) {
    app.component('HomePage', HomePage)
  },
} satisfies Theme
