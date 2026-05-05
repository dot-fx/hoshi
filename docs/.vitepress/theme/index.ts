import DefaultTheme from 'vitepress/theme'
import './custom.css'
import landing from './components/landing.vue'
import layout from './layouts/layout.vue'

export default {
    ...DefaultTheme,
    Layout: layout,
    enhanceApp({ app }) {
        app.component('landing', landing)
    }
}