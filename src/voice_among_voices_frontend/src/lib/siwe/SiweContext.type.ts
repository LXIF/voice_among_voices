import type { DelegationChain, DelegationIdentity } from "@dfinity/identity";
import type { ActorSubclass } from "@dfinity/agent";
import type { SIWE_IDENTITY_SERVICE } from "./service.interface";
import type { PrepareLoginOkResponse } from "./service.interface";

export interface SiweContextInterface {
  // State properties
  isInitializing: boolean;
  prepareLoginStatus: "idle" | "preparing" | "error" | "success";
  prepareLoginError?: Error;
  prepareLoginOkResponse?: PrepareLoginOkResponse;
  loginStatus: "idle" | "logging-in" | "error" | "success";
  loginError?: Error;
  identity?: DelegationIdentity;
  identityAddress?: `0x${string}`;
  delegationChain?: DelegationChain;
  anonymousActor?: ActorSubclass<SIWE_IDENTITY_SERVICE>;

  // Methods
  prepareLogin: () => Promise<PrepareLoginOkResponse | undefined>;
  login: () => Promise<DelegationIdentity>;
  clear: () => void;

  // Computed status flags
  isPreparingLogin: boolean;
  isPrepareLoginError: boolean;
  isPrepareLoginSuccess: boolean;
  isPrepareLoginIdle: boolean;
  isLoggingIn: boolean;
  isLoginError: boolean;
  isLoginSuccess: boolean;
  isLoginIdle: boolean;

  // TODO: Replace these with proper types once implemented
  signMessageStatus: "todo";
  signMessageError: "todo";
}
