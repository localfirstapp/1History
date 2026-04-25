import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

export default defineConfig({
  plugins: [tailwindcss()],
  root: 'pages',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        index:   resolve(__dirname, 'pages/index.html'),
        search:  resolve(__dirname, 'pages/search.html'),
        details: resolve(__dirname, 'pages/details.html'),
        db:      resolve(__dirname, 'pages/db.html'),
      },
    },
  },
  server: {
    proxy: {
      '/api':     'http://localhost:9960',
      '/details': 'http://localhost:9960',
      '/search':  'http://localhost:9960',
      '/db':      'http://localhost:9960',
    },
  },
})
