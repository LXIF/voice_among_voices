import { createAppKit } from "@reown/appkit";
import { mainnet, polygonAmoy, polygon, sepolia } from "@reown/appkit/networks";
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";
import { derived, get, readable } from "svelte/store";
import { type Config } from "wagmi";

// 1. Get a project ID at https://cloud.reown.com
// const projectId = process.env.PUBLIC_REOWN_ID!;
const projectId = "da65f4e00cada14e87d84160b45060f5"; //TODO: store in env

export const networks = [mainnet, polygon, polygonAmoy, sepolia];

// 2. Set up Wagmi adapter
const wagmiAdapter = readable<WagmiAdapter>(
  new WagmiAdapter({
    projectId,
    networks,
  })
);

// 3. Configure the metadata
const metadata = {
  name: "Voice among Voices",
  description: "Big Cool Sound Map Wee",
  url: "localhost", // origin must match your domain & subdomain //TODO
  icons: ["https://avatars.githubusercontent.com/u/179229932"],
};

// 3. Create the modal
export const appkitModal = readable(
  createAppKit({
    adapters: [get(wagmiAdapter)],
    networks: [sepolia], //TODO: change for deployment
    metadata,
    projectId,
    features: {
      analytics: true, // Optional - defaults to your Cloud configuration
    },
  })
);

export const wagmiConfig = derived<typeof wagmiAdapter, Config>(
  wagmiAdapter,
  (adapter) => adapter.wagmiConfig
);
