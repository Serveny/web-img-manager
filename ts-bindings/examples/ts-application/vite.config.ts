import { defineConfig } from 'vite';

// https://vitejs.dev/config/
export default defineConfig(async ({ mode }) => {
  return {
    server: {
      port: 1870,
      strictPort: true,
    },
    build: {
      assetsDir: './static/',
      cssMinify: true,
      minify: true,
      sourcemap: false,
      outDir: './dist/app/',
    },
    assetsInclude: ['**/*.svg'],
    root: './',
    base: '',
  };
});
