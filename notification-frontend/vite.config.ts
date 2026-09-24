import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.spec.ts'],
  },
  server: {
    proxy: {
      '/api': { target: 'http://127.0.0.1:8080', changeOrigin: true },
    },
  },
})
