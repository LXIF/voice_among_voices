import { type AppKit, createAppKit } from "@reown/appkit";
import { base } from "@reown/appkit/networks";
import {
    WagmiAdapter,
    type WagmiAdapter as WagmiAdapterType,
} from "@reown/appkit-adapter-wagmi";
import { derived, get, readable } from "svelte/store";
import { browser } from "$app/environment";
import { toastMessage } from "./state/uxState";

// Force immediate execution
const initializeStores = () => {
    const projectId = "da65f4e00cada14e87d84160b45060f5";
    const networks = [base];

    // Create stores with initial values
    const wagmiAdapter = readable<WagmiAdapterType | undefined>(
        undefined,
        (set) => {
            if (browser) {
                try {
                    const adapter = new WagmiAdapter({ projectId, networks });
                    set(adapter);
                } catch (error) {
                    toastMessage.set("Error creating WagmiAdapter");
                    console.error(
                        "[Debug] WagmiAdapter creation error:",
                        error,
                    );
                }
            }
        },
    );

    const appkitModal = readable<AppKit | undefined>(undefined, (set) => {
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
                    features: {
                        analytics: true,
                        email: true,
                        socials: ["google"],
                    },
                });
                set(modal);
            } catch (error) {
                toastMessage.set("Error creating AppKit");
                console.error("[Debug] AppKit creation error:", error);
            }
        }
    });

    const wagmiConfig = derived(wagmiAdapter, ($adapter) => {
        return $adapter?.wagmiConfig;
    });

    return { wagmiAdapter, appkitModal, wagmiConfig };
};

// Force immediate initialization
const { wagmiAdapter, appkitModal, wagmiConfig } = initializeStores();

export { wagmiAdapter, appkitModal, wagmiConfig };
