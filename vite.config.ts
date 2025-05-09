import { defineConfig } from "vite";
import { resolve } from "path";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        watch: {
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
        },
    },

    build: {
        rollupOptions: {
            input: {
                player: resolve(__dirname, "src/player/index.html"),
                playlist: resolve(__dirname, "src/playlist/index.html"),
                convert: resolve(__dirname, "src/convert/index.html"),
            },
        },
    },
    plugins: [svelte()],
}));
