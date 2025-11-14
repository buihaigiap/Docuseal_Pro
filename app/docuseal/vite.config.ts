import path from 'path';
import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig(({ mode }) => {
    const env = loadEnv(mode, '.', '');
    return {
      base: '/',
      build: {
        outDir: 'dist',
        emptyOutDir: true,
      },
      server: {
        port: 3000,
        host: 'localhost',
        proxy: {
          '/api': {
            target: env.BASE_URL ,
            changeOrigin: true,
            secure: false,
            configure: (proxy, options) => {
              proxy.on('proxyReq', (proxyReq, req, res) => {
                // Forward Authorization header
                const authHeader = req.headers.authorization;
                if (authHeader) {
                  proxyReq.setHeader('Authorization', authHeader);
                }
              });
            }
          },
          '/public': {
            target: env.BASE_URL ,
            changeOrigin: true,
            secure: false,
          },
        },
      },
      plugins: [react()],
      define: {
        'process.env.API_KEY': JSON.stringify(env.GEMINI_API_KEY),
        'process.env.GEMINI_API_KEY': JSON.stringify(env.GEMINI_API_KEY),
        'import.meta.env.VITE_API_BASE_URL': JSON.stringify('')
      },
      resolve: {
        alias: {
          '@': path.resolve(__dirname, '.'),
        }
      }
    };
});
