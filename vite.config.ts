import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tauriConfig from './src-tauri/tauri.conf.json';

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  preview: { headers: { 'Content-Security-Policy': tauriConfig.app.security.csp } },
  envPrefix: ['VITE_', 'TAURI_'],
  optimizeDeps: { include: ['react', 'react-dom/client', 'react/jsx-runtime', 'react/jsx-dev-runtime', '@phosphor-icons/react'] },
  // Native CSP intentionally disallows data: images; keep sprite sheets local.
  build: { target: 'chrome105', assetsInlineLimit: 0 },
});
