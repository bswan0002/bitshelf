import { h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import './style.css'

export default {
  extends: DefaultTheme,
  Layout: () => h(DefaultTheme.Layout, null, {
    'layout-top': () => h('div', { class: 'development-warning', role: 'note' },
      'Early development — do not use bitshelf yet. Development and testing with disposable data only.'),
  }),
}
