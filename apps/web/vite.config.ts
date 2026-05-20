import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Note: no Cross-Origin-Isolation headers. Parameters are marshalled over a
// MessagePort (not a SharedArrayBuffer), so the app does not need — and is
// not complicated by — cross-origin isolation.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
  },
});
