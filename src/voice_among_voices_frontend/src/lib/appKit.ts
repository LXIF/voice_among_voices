import { type AppKit, createAppKit } from "@reown/appkit";
import { base } from "@reown/appkit/networks";
import {
    WagmiAdapter,
    type WagmiAdapter as WagmiAdapterType,
} from "@reown/appkit-adapter-wagmi";
import { derived, get, readable } from "svelte/store";
import { browser } from "$app/environment";

console.log("[Debug] Module initialization start");

// Force immediate execution
const initializeStores = () => {
    console.log("[Debug] Initializing stores");

    const projectId = "da65f4e00cada14e87d84160b45060f5";
    const networks = [base];

    // Create stores with initial values
    const wagmiAdapter = readable<WagmiAdapterType | undefined>(
        undefined,
        (set) => {
            console.log("[Debug] Setting up wagmiAdapter");
            if (browser) {
                try {
                    const adapter = new WagmiAdapter({ projectId, networks });
                    set(adapter);
                } catch (error) {
                    console.error(
                        "[Debug] WagmiAdapter creation error:",
                        error,
                    );
                }
            }
        },
    );

    const appkitModal = readable<AppKit | undefined>(undefined, (set) => {
        console.log("[Debug] Setting up appkitModal");
        if (browser) {
            try {
                const modal = createAppKit({
                    adapters: [get(wagmiAdapter)].filter(
                        Boolean,
                    ) as WagmiAdapterType[],
                    networks: [base],
                    metadata: {
                        name: "Voice among Voices",
                        description: "Big Cool Sound Map Wee",
                        url: browser ? window.location.origin : "",
                        icons: [
                            "https://avatars.githubusercontent.com/u/179229932",
                        ],
                    },
                    projectId,
                    features: { analytics: true },
                });
                set(modal);
            } catch (error) {
                console.error("[Debug] AppKit creation error:", error);
            }
        }
    });

    const wagmiConfig = derived(wagmiAdapter, ($adapter) => {
        console.log("[Debug] Deriving wagmiConfig");
        return $adapter?.wagmiConfig;
    });

    return { wagmiAdapter, appkitModal, wagmiConfig };
};

// Force immediate initialization
const { wagmiAdapter, appkitModal, wagmiConfig } = initializeStores();

console.log("[Debug] Module initialization complete");

export { wagmiAdapter, appkitModal, wagmiConfig };
