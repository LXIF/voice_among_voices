import { canisterId } from "../../../../declarations/ic_siwe_provider";
import { SiweManager, siweStateStore } from "ic-siwe-js";
import { writable } from "svelte/store";
import { browser } from "$app/environment";

// Initialize stores with null for SSR
export const siwe = writable<SiweManager | null>(null);
export const prepareLoginStatus = writable("");
export const prepareLoginError = writable<string | null>(null);
export const loginStatus = writable("");
export const loginError = writable<string | null>(null);
export const signMessageStatus = writable("");

// Only initialize SIWE in browser
if (browser) {
    // Initialize SIWE manager
    siwe.set(new SiweManager(canisterId));

    // Set up subscription
    siweStateStore.subscribe((snapshot) => {
        const {
            prepareLoginStatus: prepLoginStatus,
            prepareLoginError: prepLoginError,
            loginStatus: logStatus,
            loginError: logError,
            signMessageStatus: signMsgStatus,
        } = snapshot.context;

        prepareLoginStatus.set(prepLoginStatus);
        prepareLoginError.set(prepLoginError?.message || null);
        loginStatus.set(logStatus);
        loginError.set(logError?.message || null);
        signMessageStatus.set(signMsgStatus);
    });
}
