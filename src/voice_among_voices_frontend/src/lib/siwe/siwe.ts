import { canisterId } from "../../../../declarations/ic_siwe_provider";
import { SiweManager, siweStateStore } from "ic-siwe-js";
import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { type Identity } from "@dfinity/agent";

// Initialize stores with null for SSR
export const siwe = writable<SiweManager | null>(null);
export const prepareLoginStatus = writable("");
export const prepareLoginError = writable<string | null>(null);
export const loginStatus = writable("");
export const loginError = writable<string | null>(null);
export const signMessageStatus = writable("");
export const siweIdentity = writable<Identity | undefined>();

// Only initialize SIWE in browser
if (browser) {
    // Set up subscription
    siweStateStore.subscribe((snapshot) => {
        const {
            prepareLoginStatus: prepLoginStatus,
            prepareLoginError: prepLoginError,
            loginStatus: logStatus,
            loginError: logError,
            signMessageStatus: signMsgStatus,
            identity
        } = snapshot.context;

        prepareLoginStatus.set(prepLoginStatus);
        prepareLoginError.set(prepLoginError?.message || null);
        loginStatus.set(logStatus);
        loginError.set(logError?.message || null);
        signMessageStatus.set(signMsgStatus);
        siweIdentity.set(identity);
    });

    // Initialize SIWE manager
    siwe.set(new SiweManager(canisterId));
}
