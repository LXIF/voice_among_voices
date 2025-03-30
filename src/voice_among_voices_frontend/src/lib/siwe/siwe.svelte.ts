import { canisterId } from "../../../../declarations/ic_siwe_provider";
import { SiweManager, siweStateStore } from "ic-siwe-js";
import { writable } from "svelte/store";

export const siwe = writable(new SiweManager(canisterId));
export const prepareLoginStatus = writable("");
export const prepareLoginError = writable<string | null>(null);
export const loginStatus = writable("");
export const loginError = writable<string | null>(null);
export const signMessageStatus = writable("");

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
