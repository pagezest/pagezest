import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
  optimizeDeps: {
    exclude: ['lucide-react'],
  },
  server: {
    /*
    proxy: {
      '/': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
    middlewareMode: true,
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        // Check if the request URL starts with the prefix /local-api
        if (req.url?.startsWith('/local-api')) {
          // Remove the /local-api prefix
          req.url = req.url.replace(/^\/local-api/, '');
          // Continue with local handling (no proxy)
          return next();
        }
        // Otherwise, proceed with proxying the request
        next();
      });
    },
    */
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
        //rewrite: path => path.replace(/^\/api/, '')
      }
    }
    /*
    */
  }
});
