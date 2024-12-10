import { createAppKit } from "@reown/appkit";
import {
  mainnet,
  arbitrum,
  polygonAmoy,
  polygon,
} from "@reown/appkit/networks";
import { WagmiAdapter } from "@reown/appkit-adapter-wagmi";

// TODO: turn into store

// 1. Get a project ID at https://cloud.reown.com
const projectId = process.env.PUBLIC_REOWN_ID;

export const networks = [mainnet, polygon, polygonAmoy];

// 2. Set up Wagmi adapter
const wagmiAdapter = new WagmiAdapter({
  projectId,
  networks,
});

// 3. Configure the metadata
const metadata = {
  name: "Voice among Voices",
  description: "Big Cool Sound Map Wee",
  url: "https://example.com", // origin must match your domain & subdomain
  icons: ["https://avatars.githubusercontent.com/u/179229932"],
};

// 3. Create the modal
const modal = createAppKit({
  adapters: [wagmiAdapter],
  networks: [mainnet, arbitrum],
  metadata,
  projectId,
  features: {
    analytics: true, // Optional - defaults to your Cloud configuration
  },
});

// 4. Trigger modal programaticaly
const openConnectModalBtn = document.getElementById("open-connect-modal");
const openNetworkModalBtn = document.getElementById("open-network-modal");

openConnectModalBtn.addEventListener("click", () => modal.open());
openNetworkModalBtn.addEventListener("click", () =>
  modal.open({ view: "Networks" })
);

// 5. Alternatively use w3m component buttons within the index.html file
