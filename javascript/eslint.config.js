import js from '@eslint/js'
import globals from 'globals'

//the recommended rules and nothing more: they catch real mistakes and leave style to style.md and format to prettier
export default [
  js.configs.recommended,
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node }, //ergo.js runs in both; the driver and the suite are node
    },
  },
]
