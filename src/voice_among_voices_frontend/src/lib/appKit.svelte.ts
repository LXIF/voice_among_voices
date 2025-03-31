import { createAppKit } from "@reown/appkit";
import { mainnet, polygonAmoy, polygon, sepolia } from "@reown/appkit/networks";
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";
import { derived, get, readable } from "svelte/store";
import { type Config } from "wagmi";
import { browser } from "$app/environment";

// 1. Get a project ID at https://cloud.reown.com
// const projectId = process.env.PUBLIC_REOWN_ID!;
const projectId = "da65f4e00cada14e87d84160b45060f5"; //TODO: store in env

export const networks = [mainnet, polygon, polygonAmoy, sepolia];

// Initialize adapter only in browser context
const createWagmiAdapter = () => {
  if (!browser) return null;

  console.log("[Debug] Creating WagmiAdapter");
  return new WagmiAdapter({
    projectId,
    networks,
  });
};

// Get deployment-aware URL
const getAppUrl = () => {
  if (!browser) return "";
  const host = window.location.host;
  return host.includes("localhost")
    ? "http://localhost:4943"
    : `https://${host}`;
};

// Create stores only when in browser context
export const wagmiAdapter = readable<WagmiAdapter | null>(null, (set) => {
  if (browser) {
    const adapter = createWagmiAdapter();
    console.log("[Debug] WagmiAdapter created:", !!adapter);
    set(adapter);
  }
});

// 3. Configure the metadata
const metadata = {
  name: "Voice among Voices",
  description: "Big Cool Sound Map Wee",
  url: getAppUrl(),
  icons: ["https://avatars.githubusercontent.com/u/179229932"],
};

// Create AppKit instance only in browser context
export const appkitModal = readable(
  browser
    ? createAppKit({
        adapters: [get(wagmiAdapter)].filter(Boolean) as WagmiAdapter[],
        networks: [sepolia], // TODO: change for deployment
        metadata,
        projectId,
        features: {
          analytics: true,
        },
      })
    : null
);

export const wagmiConfig = derived<typeof wagmiAdapter, Config | null>(
  wagmiAdapter,
  ($adapter) => {
    if (!$adapter) {
      console.log("[Debug] No WagmiAdapter available");
      return null;
    }
    console.log("[Debug] Creating Wagmi config");
    return $adapter.wagmiConfig;
  }
);

// Debug logging
if (browser) {
  console.log("[Debug] AppKit initialization:", {
    url: getAppUrl(),
    networks: networks.map((n) => n.name),
    hasAdapter: !!get(wagmiAdapter),
  });
}
