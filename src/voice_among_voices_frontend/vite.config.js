/// <reference types="vitest" />
import { fileURLToPath, URL } from "url";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import environment from "vite-plugin-environment";
import dotenv from "dotenv";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

dotenv.config({ path: "../../.env" });

export default defineConfig({
    build: {
        emptyOutDir: true,
        minify: true,
        sourcemap: true,
        rollupOptions: {
            output: {
                manualChunks(id) {
                    // Ensure core runtime chunks are loaded first
                    if (id.includes("node_modules")) {
                        if (id.includes("rapier")) return "physics";
                        if (
                            id.includes("reown") ||
                            id.includes("wagmi") ||
                            id.includes("viem")
                        )
                            return "evm";

                        return "vendor";
                    }

                    // Group related app code
                    if (id.includes("src/lib")) {
                        return "lib";
                    }
                },
            },
        },
    },
    optimizeDeps: {
        // include: [
        //     // "@reown/appkit",
        //     // "@reown/appkit-adapter-wagmi",
        //     "ic-siwe-js",
        //     // "@wagmi/core",
        //     // "viem",
        //     // "wagmi",
        //     // "@dfinity/agent",
        //     // "@dfinity/candid",
        //     // "@dfinity/identity",
        //     // "@dfinity/principal",
        // ],
        // esbuildOptions: {
        //     define: {
        //         target: "es2020",
        //         global: "globalThis",
        //     },
        // },
    },
    server: {
        proxy: {
            "/api": {
                target: "http://127.0.0.1:4943",
                changeOrigin: true,
            },
        },
    },
    plugins: [
        sveltekit(),
        wasm(),
        topLevelAwait(),
        environment("all", { prefix: "CANISTER_" }),
        environment("all", { prefix: "DFX_" }),
    ],
    test: {
        environment: "jsdom",
        setupFiles: "src/setupTests.js",
    },
    resolve: {
        alias: [
            {
                find: "declarations",
                replacement: fileURLToPath(
                    new URL("../declarations", import.meta.url),
                ),
            },
        ],
        dedupe: ["@dfinity/agent"],
    },
});
