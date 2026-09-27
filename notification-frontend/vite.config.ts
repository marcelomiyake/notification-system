import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

const callerApiKey = process.env.NOTIFICATION_CALLER_API_KEY

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.spec.ts'],
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
        configure: proxy => {
          if (callerApiKey) {
            proxy.on('proxyReq', request => request.setHeader('x-api-key', callerApiKey))
          }
        },
      },
    },
  },
})
