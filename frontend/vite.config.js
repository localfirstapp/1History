import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

export default defineConfig({
  plugins: [tailwindcss()],
  build: {
    outDir: '../static/dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main:    resolve(__dirname, 'src/main.js'),
        search:  resolve(__dirname, 'src/search.js'),
        details: resolve(__dirname, 'src/details.js'),
        db:      resolve(__dirname, 'src/db.js'),
      },
      output: {
        entryFileNames: 'js/[name].js',
        chunkFileNames: 'js/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash][extname]',
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
