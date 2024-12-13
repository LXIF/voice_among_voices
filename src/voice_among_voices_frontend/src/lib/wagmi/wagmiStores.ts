import { createConfig, http, type Config, getAccount } from "@wagmi/core";
import { mainnet, polygon, polygonAmoy } from "viem/chains";
import { injected } from "@wagmi/connectors";
import { readable, derived, type Readable } from "svelte/store";
import type { Address } from "viem";

// Create wagmi config
export const wagmiConfig = readable<Config>(
  createConfig({
    chains: [mainnet, polygon, polygonAmoy],
    connectors: [injected()],
    transports: {
      [mainnet.id]: http(),
      [polygon.id]: http(),
      [polygonAmoy.id]: http(),
    },
  })
);

export const signerAddress: Readable<Address | undefined> = derived(
  wagmiConfig,
  (config) => getAccount(config).address
);
