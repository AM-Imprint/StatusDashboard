import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  preview: {
    allowedHosts: ["andrew.imprint.vpn"]
  },
  server: {
    proxy: {
      '/api': 'http://andrew.imprint.vpn:3000',
      '/ws': { target: 'ws://andrew.imprint.vpn:3000', ws: true },
    },
  },
})
