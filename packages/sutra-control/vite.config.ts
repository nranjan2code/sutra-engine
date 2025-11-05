import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { VitePWA } from 'vite-plugin-pwa'

export default defineConfig({
  plugins: [
    react(),
    VitePWA({
      registerType: 'autoUpdate',
      workbox: {
        globPatterns: ['**/*.{js,css,html,ico,png,svg}']
      },
      manifest: {
        name: 'Sutra AI Control Center',
        short_name: 'Sutra Control',
        description: 'Modern control center for Sutra AI system',
        theme_color: '#6366f1',
        background_color: '#0f1629',
        display: 'standalone',
        scope: '/',
        start_url: '/',
        icons: [
          {
            src: '/favicon.ico',
            sizes: '64x64 32x32 24x24 16x16',
            type: 'image/x-icon'
          }
        ]
      }
    })
  ],
  server: {
    port: 3000,
    proxy: {
      '/api': 'http://localhost:5000',
      '/ws': {
        target: 'ws://localhost:5000',
        ws: true
      }
    }
  },
  build: {
    outDir: 'dist',
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: false, // Disable for production
    
    rollupOptions: {
      output: {
        manualChunks: {
          // Vendor chunk for stable libraries
          vendor: ['react', 'react-dom', 'react-router-dom'],
          
          // UI framework chunk
          ui: ['@mui/material', '@mui/icons-material', '@emotion/react', '@emotion/styled'],
          
          // Charts and visualization (optimized)
          charts: ['recharts', 'd3', 'cytoscape', '@mui/x-charts', '@mui/x-data-grid'],
          
          // State management and utilities
          utils: ['zustand', 'date-fns', 'framer-motion']
        },
        
        // Optimize chunk names
        chunkFileNames: 'js/[name]-[hash].js',
        entryFileNames: 'js/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash].[ext]',
      }
    },
    
    // Chunk size warnings
    chunkSizeWarningLimit: 800, // 800KB
  },
  
  // Performance optimizations
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      '@mui/material',
      '@mui/icons-material',
      'recharts',
    ],
  }
})