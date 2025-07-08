// vite.config.js
import { fileURLToPath, URL } from "url";
import { sveltekit } from "file:///Users/andri.schatz/dev/personal/voice_among_voices/node_modules/@sveltejs/kit/src/exports/vite/index.js";
import { defineConfig } from "file:///Users/andri.schatz/dev/personal/voice_among_voices/node_modules/vite/dist/node/index.js";
import environment from "file:///Users/andri.schatz/dev/personal/voice_among_voices/node_modules/vite-plugin-environment/dist/index.js";
import dotenv from "file:///Users/andri.schatz/dev/personal/voice_among_voices/node_modules/dotenv/lib/main.js";
import wasm from "file:///Users/andri.schatz/dev/personal/voice_among_voices/node_modules/vite-plugin-wasm/exports/import.mjs";
import topLevelAwait from "file:///Users/andri.schatz/dev/personal/voice_among_voices/node_modules/vite-plugin-top-level-await/exports/import.mjs";
var __vite_injected_original_import_meta_url = "file:///Users/andri.schatz/dev/personal/voice_among_voices/src/voice_among_voices_frontend/vite.config.js";
dotenv.config({ path: "../../.env" });
var vite_config_default = defineConfig({
  build: {
    emptyOutDir: true,
    minify: true,
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules")) {
            return "vendor";
          }
          if (id.includes("src/lib")) {
            return "lib";
          }
        }
      }
    }
  },
  optimizeDeps: {
    include: [
      // "@reown/appkit",
      // "@reown/appkit-adapter-wagmi",
      "ic-siwe-js"
      // "@wagmi/core",
      // "viem",
      // "wagmi",
      // "@dfinity/agent",
      // "@dfinity/candid",
      // "@dfinity/identity",
      // "@dfinity/principal",
    ],
    esbuildOptions: {
      define: {
        target: "es2020",
        global: "globalThis"
      }
    }
  },
  server: {
    proxy: {
      "/api": {
        target: "http://127.0.0.1:4943",
        changeOrigin: true
      }
    }
  },
  plugins: [
    sveltekit(),
    wasm(),
    topLevelAwait(),
    environment("all", { prefix: "CANISTER_" }),
    environment("all", { prefix: "DFX_" })
  ],
  test: {
    environment: "jsdom",
    setupFiles: "src/setupTests.js"
  },
  resolve: {
    alias: [
      {
        find: "declarations",
        replacement: fileURLToPath(
          new URL("../declarations", __vite_injected_original_import_meta_url)
        )
      }
    ],
    dedupe: ["@dfinity/agent"]
  }
});
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcuanMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvVXNlcnMvYW5kcmkuc2NoYXR6L2Rldi9wZXJzb25hbC92b2ljZV9hbW9uZ192b2ljZXMvc3JjL3ZvaWNlX2Ftb25nX3ZvaWNlc19mcm9udGVuZFwiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9maWxlbmFtZSA9IFwiL1VzZXJzL2FuZHJpLnNjaGF0ei9kZXYvcGVyc29uYWwvdm9pY2VfYW1vbmdfdm9pY2VzL3NyYy92b2ljZV9hbW9uZ192b2ljZXNfZnJvbnRlbmQvdml0ZS5jb25maWcuanNcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfaW1wb3J0X21ldGFfdXJsID0gXCJmaWxlOi8vL1VzZXJzL2FuZHJpLnNjaGF0ei9kZXYvcGVyc29uYWwvdm9pY2VfYW1vbmdfdm9pY2VzL3NyYy92b2ljZV9hbW9uZ192b2ljZXNfZnJvbnRlbmQvdml0ZS5jb25maWcuanNcIjsvLy8gPHJlZmVyZW5jZSB0eXBlcz1cInZpdGVzdFwiIC8+XG5pbXBvcnQgeyBmaWxlVVJMVG9QYXRoLCBVUkwgfSBmcm9tIFwidXJsXCI7XG5pbXBvcnQgeyBzdmVsdGVraXQgfSBmcm9tIFwiQHN2ZWx0ZWpzL2tpdC92aXRlXCI7XG5pbXBvcnQgeyBkZWZpbmVDb25maWcgfSBmcm9tIFwidml0ZVwiO1xuaW1wb3J0IGVudmlyb25tZW50IGZyb20gXCJ2aXRlLXBsdWdpbi1lbnZpcm9ubWVudFwiO1xuaW1wb3J0IGRvdGVudiBmcm9tIFwiZG90ZW52XCI7XG5pbXBvcnQgd2FzbSBmcm9tIFwidml0ZS1wbHVnaW4td2FzbVwiO1xuaW1wb3J0IHRvcExldmVsQXdhaXQgZnJvbSBcInZpdGUtcGx1Z2luLXRvcC1sZXZlbC1hd2FpdFwiO1xuXG5kb3RlbnYuY29uZmlnKHsgcGF0aDogXCIuLi8uLi8uZW52XCIgfSk7XG5cbmV4cG9ydCBkZWZhdWx0IGRlZmluZUNvbmZpZyh7XG4gICAgYnVpbGQ6IHtcbiAgICAgICAgZW1wdHlPdXREaXI6IHRydWUsXG4gICAgICAgIG1pbmlmeTogdHJ1ZSxcbiAgICAgICAgc291cmNlbWFwOiB0cnVlLFxuICAgICAgICByb2xsdXBPcHRpb25zOiB7XG4gICAgICAgICAgICBvdXRwdXQ6IHtcbiAgICAgICAgICAgICAgICBtYW51YWxDaHVua3MoaWQpIHtcbiAgICAgICAgICAgICAgICAgICAgLy8gRW5zdXJlIGNvcmUgcnVudGltZSBjaHVua3MgYXJlIGxvYWRlZCBmaXJzdFxuICAgICAgICAgICAgICAgICAgICBpZiAoaWQuaW5jbHVkZXMoXCJub2RlX21vZHVsZXNcIikpIHtcbiAgICAgICAgICAgICAgICAgICAgICAgIHJldHVybiBcInZlbmRvclwiO1xuICAgICAgICAgICAgICAgICAgICB9XG4gICAgICAgICAgICAgICAgICAgIC8vIEdyb3VwIHJlbGF0ZWQgYXBwIGNvZGVcbiAgICAgICAgICAgICAgICAgICAgaWYgKGlkLmluY2x1ZGVzKFwic3JjL2xpYlwiKSkge1xuICAgICAgICAgICAgICAgICAgICAgICAgcmV0dXJuIFwibGliXCI7XG4gICAgICAgICAgICAgICAgICAgIH1cbiAgICAgICAgICAgICAgICB9LFxuICAgICAgICAgICAgfSxcbiAgICAgICAgfSxcbiAgICB9LFxuICAgIG9wdGltaXplRGVwczoge1xuICAgICAgICBpbmNsdWRlOiBbXG4gICAgICAgICAgICAvLyBcIkByZW93bi9hcHBraXRcIixcbiAgICAgICAgICAgIC8vIFwiQHJlb3duL2FwcGtpdC1hZGFwdGVyLXdhZ21pXCIsXG4gICAgICAgICAgICBcImljLXNpd2UtanNcIixcbiAgICAgICAgICAgIC8vIFwiQHdhZ21pL2NvcmVcIixcbiAgICAgICAgICAgIC8vIFwidmllbVwiLFxuICAgICAgICAgICAgLy8gXCJ3YWdtaVwiLFxuICAgICAgICAgICAgLy8gXCJAZGZpbml0eS9hZ2VudFwiLFxuICAgICAgICAgICAgLy8gXCJAZGZpbml0eS9jYW5kaWRcIixcbiAgICAgICAgICAgIC8vIFwiQGRmaW5pdHkvaWRlbnRpdHlcIixcbiAgICAgICAgICAgIC8vIFwiQGRmaW5pdHkvcHJpbmNpcGFsXCIsXG4gICAgICAgIF0sXG4gICAgICAgIGVzYnVpbGRPcHRpb25zOiB7XG4gICAgICAgICAgICBkZWZpbmU6IHtcbiAgICAgICAgICAgICAgICB0YXJnZXQ6IFwiZXMyMDIwXCIsXG4gICAgICAgICAgICAgICAgZ2xvYmFsOiBcImdsb2JhbFRoaXNcIixcbiAgICAgICAgICAgIH0sXG4gICAgICAgIH0sXG4gICAgfSxcbiAgICBzZXJ2ZXI6IHtcbiAgICAgICAgcHJveHk6IHtcbiAgICAgICAgICAgIFwiL2FwaVwiOiB7XG4gICAgICAgICAgICAgICAgdGFyZ2V0OiBcImh0dHA6Ly8xMjcuMC4wLjE6NDk0M1wiLFxuICAgICAgICAgICAgICAgIGNoYW5nZU9yaWdpbjogdHJ1ZSxcbiAgICAgICAgICAgIH0sXG4gICAgICAgIH0sXG4gICAgfSxcbiAgICBwbHVnaW5zOiBbXG4gICAgICAgIHN2ZWx0ZWtpdCgpLFxuICAgICAgICB3YXNtKCksXG4gICAgICAgIHRvcExldmVsQXdhaXQoKSxcbiAgICAgICAgZW52aXJvbm1lbnQoXCJhbGxcIiwgeyBwcmVmaXg6IFwiQ0FOSVNURVJfXCIgfSksXG4gICAgICAgIGVudmlyb25tZW50KFwiYWxsXCIsIHsgcHJlZml4OiBcIkRGWF9cIiB9KSxcbiAgICBdLFxuICAgIHRlc3Q6IHtcbiAgICAgICAgZW52aXJvbm1lbnQ6IFwianNkb21cIixcbiAgICAgICAgc2V0dXBGaWxlczogXCJzcmMvc2V0dXBUZXN0cy5qc1wiLFxuICAgIH0sXG4gICAgcmVzb2x2ZToge1xuICAgICAgICBhbGlhczogW1xuICAgICAgICAgICAge1xuICAgICAgICAgICAgICAgIGZpbmQ6IFwiZGVjbGFyYXRpb25zXCIsXG4gICAgICAgICAgICAgICAgcmVwbGFjZW1lbnQ6IGZpbGVVUkxUb1BhdGgoXG4gICAgICAgICAgICAgICAgICAgIG5ldyBVUkwoXCIuLi9kZWNsYXJhdGlvbnNcIiwgaW1wb3J0Lm1ldGEudXJsKSxcbiAgICAgICAgICAgICAgICApLFxuICAgICAgICAgICAgfSxcbiAgICAgICAgXSxcbiAgICAgICAgZGVkdXBlOiBbXCJAZGZpbml0eS9hZ2VudFwiXSxcbiAgICB9LFxufSk7XG4iXSwKICAibWFwcGluZ3MiOiAiO0FBQ0EsU0FBUyxlQUFlLFdBQVc7QUFDbkMsU0FBUyxpQkFBaUI7QUFDMUIsU0FBUyxvQkFBb0I7QUFDN0IsT0FBTyxpQkFBaUI7QUFDeEIsT0FBTyxZQUFZO0FBQ25CLE9BQU8sVUFBVTtBQUNqQixPQUFPLG1CQUFtQjtBQVBvUCxJQUFNLDJDQUEyQztBQVMvVCxPQUFPLE9BQU8sRUFBRSxNQUFNLGFBQWEsQ0FBQztBQUVwQyxJQUFPLHNCQUFRLGFBQWE7QUFBQSxFQUN4QixPQUFPO0FBQUEsSUFDSCxhQUFhO0FBQUEsSUFDYixRQUFRO0FBQUEsSUFDUixXQUFXO0FBQUEsSUFDWCxlQUFlO0FBQUEsTUFDWCxRQUFRO0FBQUEsUUFDSixhQUFhLElBQUk7QUFFYixjQUFJLEdBQUcsU0FBUyxjQUFjLEdBQUc7QUFDN0IsbUJBQU87QUFBQSxVQUNYO0FBRUEsY0FBSSxHQUFHLFNBQVMsU0FBUyxHQUFHO0FBQ3hCLG1CQUFPO0FBQUEsVUFDWDtBQUFBLFFBQ0o7QUFBQSxNQUNKO0FBQUEsSUFDSjtBQUFBLEVBQ0o7QUFBQSxFQUNBLGNBQWM7QUFBQSxJQUNWLFNBQVM7QUFBQTtBQUFBO0FBQUEsTUFHTDtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUEsSUFRSjtBQUFBLElBQ0EsZ0JBQWdCO0FBQUEsTUFDWixRQUFRO0FBQUEsUUFDSixRQUFRO0FBQUEsUUFDUixRQUFRO0FBQUEsTUFDWjtBQUFBLElBQ0o7QUFBQSxFQUNKO0FBQUEsRUFDQSxRQUFRO0FBQUEsSUFDSixPQUFPO0FBQUEsTUFDSCxRQUFRO0FBQUEsUUFDSixRQUFRO0FBQUEsUUFDUixjQUFjO0FBQUEsTUFDbEI7QUFBQSxJQUNKO0FBQUEsRUFDSjtBQUFBLEVBQ0EsU0FBUztBQUFBLElBQ0wsVUFBVTtBQUFBLElBQ1YsS0FBSztBQUFBLElBQ0wsY0FBYztBQUFBLElBQ2QsWUFBWSxPQUFPLEVBQUUsUUFBUSxZQUFZLENBQUM7QUFBQSxJQUMxQyxZQUFZLE9BQU8sRUFBRSxRQUFRLE9BQU8sQ0FBQztBQUFBLEVBQ3pDO0FBQUEsRUFDQSxNQUFNO0FBQUEsSUFDRixhQUFhO0FBQUEsSUFDYixZQUFZO0FBQUEsRUFDaEI7QUFBQSxFQUNBLFNBQVM7QUFBQSxJQUNMLE9BQU87QUFBQSxNQUNIO0FBQUEsUUFDSSxNQUFNO0FBQUEsUUFDTixhQUFhO0FBQUEsVUFDVCxJQUFJLElBQUksbUJBQW1CLHdDQUFlO0FBQUEsUUFDOUM7QUFBQSxNQUNKO0FBQUEsSUFDSjtBQUFBLElBQ0EsUUFBUSxDQUFDLGdCQUFnQjtBQUFBLEVBQzdCO0FBQ0osQ0FBQzsiLAogICJuYW1lcyI6IFtdCn0K
