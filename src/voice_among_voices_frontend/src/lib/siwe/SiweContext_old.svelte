<script lang="ts">

// //TODO: TRANSLATE TO SVELTEKIT

// /* eslint-disable react-refresh/only-export-components */
// // import {
// //   createContext,
// //   useContext,
// //   type ReactNode,
// //   useEffect,
// //   useState,
// //   useRef,
// //   useCallback,
// // } from "react";
// import { type ActorConfig, type HttpAgentOptions } from "@dfinity/agent";
// import { DelegationIdentity, Ed25519KeyIdentity } from "@dfinity/identity";
// // import type { SiweIdentityContextType } from "./context.type";
// // import { useAccount, useSignMessage } from "wagmi";
// import { IDL } from "@dfinity/candid";
// import type {
//   LoginOkResponse,
//   PrepareLoginOkResponse,
//   SIWE_IDENTITY_SERVICE,
//   SignedDelegation as ServiceSignedDelegation,
// } from "./service.interface";
// import { clearIdentity, loadIdentity, saveIdentity } from "./local-storage";
// import {
//   callGetDelegation,
//   callLogin,
//   callPrepareLogin,
//   createAnonymousActor,
// } from "./siwe-provider";
// import type { State } from "./state.type";
// // import type { SignMessageErrorType } from "wagmi/actions";
// import type { SignMessageErrorType } from "@wagmi/core";
// import { createDelegationChain } from "./delegation";
// import { normalizeError } from "./error";

// import { connected, signerAddress } from "svelte-wagmi";
// import { writable, get } from "svelte/store";
// import { setContext } from "svelte";

// /**
//  * Re-export types
//  */
// // export * from "./context.type";
// export * from "./service.interface";
// export * from "./storage.type";

// // TODO DELET
// /**
//  * React context for managing SIWE (Sign-In with Ethereum) identity.
//  */
// // export const SiweIdentityContext = createContext<
// //   SiweIdentityContextType | undefined
// // >(undefined);

// // TODO DELET
// /**
//  * Hook to access the SiweIdentityContext.
//  */
// // export const useSiweIdentity = (): SiweIdentityContextType => {
// //   const context = useContext(SiweIdentityContext);
// //   if (!context) {
// //     throw new Error(
// //       "useSiweIdentity must be used within an SiweIdentityProvider"
// //     );
// //   }
// //   return context;
// // };

// //TODO get the props somehow (maybe this is a derived store?)
// /**
//  * Provider component for the SIWE identity context. Manages identity state and provides authentication-related functionalities.
//  *
//  * @prop {IDL.InterfaceFactory} idlFactory - Required. The Interface Description Language (IDL) factory for the canister. This factory is used to create an actor interface for the canister.
//  * @prop {string} canisterId - Required. The unique identifier of the canister on the Internet Computer network. This ID is used to establish a connection to the canister.
//  * @prop {HttpAgentOptions} httpAgentOptions - Optional. Configuration options for the HTTP agent used to communicate with the Internet Computer network.
//  * @prop {ActorConfig} actorOptions - Optional. Configuration options for the actor. These options are passed to the actor upon its creation.
//  * @prop {ReactNode} children - Required. The child components that the SiweIdentityProvider will wrap. This allows any child component to access the authentication context provided by the SiweIdentityProvider.
//  *
//  * @example
//  * ```tsx
//  * import { SiweIdentityProvider } from 'ic-use-siwe-identity';
//  * import {canisterId, idlFactory} from "path-to/siwe-enabled-canister/index";
//  * import { _SERVICE } from "path-to/siwe-enabled-canister.did";
//  *
//  * function App() {
//  *   return (
//  *     <SiweIdentityProvider<_SERVICE>
//  *       idlFactory={idlFactory}
//  *       canisterId={canisterId}
//  *       // ...other props
//  *     >
//  *       {... your app components}
//  *     </App>
//  *   );
//  * }
//  *
//  * import { SiweIdentityProvider } from "ic-use-siwe-identity";
//  *```
//  */
// // // eslint-disable-next-line @typescript-eslint/no-unused-vars
// // export function SiweIdentityProvider<T extends SIWE_IDENTITY_SERVICE>({
// //   httpAgentOptions,
// //   actorOptions,
// //   idlFactory,
// //   canisterId,
// //   children,
// // }: {
// //   /** Configuration options for the HTTP agent used to communicate with the Internet Computer network. */
// //   httpAgentOptions?: HttpAgentOptions;

// //   /** Configuration options for the actor. These options are passed to the actor upon its creation. */
// //   actorOptions?: ActorConfig;

// //   /** The Interface Description Language (IDL) factory for the canister. This factory is used to create an actor interface for the canister. */
// //   idlFactory: IDL.InterfaceFactory;

// //   /** The unique identifier of the canister on the Internet Computer network. This ID is used to establish a connection to the canister. */
// //   canisterId: string;

// //   /** The child components that the SiweIdentityProvider will wrap. This allows any child component to access the authentication context provided by the SiweIdentityProvider. */
// //   children: ReactNode;
// // }) {
// //   const { address: connectedEthAddress } = useAccount();

// import { signMessage } from "@wagmi/core";

// let prepareLoginOkResponse = $state<PrepareLoginOkResponse | undefined>(
//   undefined
// );

// //TODO DELET
// // const [state, setState] = useState<State>({
// //   isInitializing: true,
// //   prepareLoginStatus: "idle",
// //   loginStatus: "idle",
// // });

// let contextState = $state<State>({
//   isInitializing: true,
//   prepareLoginStatus: "idle",
//   loginStatus: "idle",
// });

// const updateState = (newState: Partial<State>) => {
//   contextState = { ...contextState, ...newState };
// };

// // Keep track of the promise handlers for the login method during the async login process.
// //TODO DELET
// // const loginPromiseHandlers = useRef<{
// //   resolve: (
// //     value: DelegationIdentity | PromiseLike<DelegationIdentity>
// //   ) => void;
// //   reject: (error: Error) => void;
// // } | null>(null);

// let loginPromiseHandlers = $state<{
//   resolve: (
//     value: DelegationIdentity | PromiseLike<DelegationIdentity>
//   ) => void;
//   reject: (error: Error) => void;
// } | null>(null);

// /**
//  * Load a SIWE message from the provider, to be used for login. Calling prepareLogin
//  * is optional, as it will be called automatically on login if not called manually.
//  */
// async function prepareLogin(): Promise<PrepareLoginOkResponse | undefined> {
//   if (!contextState.anonymousActor) {
//     throw new Error(
//       "Hook not initialized properly. Make sure to supply all required props to the SiweIdentityProvider."
//     );
//   }
//   if (!get(signerAddress)) {
//     throw new Error(
//       "No Ethereum address available. Call prepareLogin after the user has connected their wallet."
//     );
//   }

//   updateState({
//     prepareLoginStatus: "preparing",
//     prepareLoginError: undefined,
//   });

//   try {
//     const response = await callPrepareLogin(
//       contextState.anonymousActor,
//       get(signerAddress) ? (get(signerAddress) as `0x${string}`) : undefined
//     );
//     updateState({
//       prepareLoginOkResponse: response,
//       prepareLoginStatus: "success",
//     });
//     prepareLoginOkResponse = response;
//     return response;
//   } catch (e) {
//     const error = normalizeError(e);
//     console.error(error);
//     updateState({
//       prepareLoginStatus: "error",
//       prepareLoginError: error,
//     });
//   }
// }

// async function rejectLoginWithError(error: Error | unknown, message?: string) {
//   const e = normalizeError(error);
//   const errorMessage = message || e.message;

//   console.error(e);

//   updateState({
//     prepareLoginOkResponse: undefined,
//     loginStatus: "error",
//     loginError: new Error(errorMessage),
//   });

//   loginPromiseHandlers?.reject(new Error(errorMessage));
// }

// /**
//  * This function is called when the signMessage hook has settled, that is, when the
//  * user has signed the message or canceled the signing process.
//  */
// async function onLoginSignatureSettled(
//   loginSignature: `0x${string}` | undefined,
//   error: SignMessageErrorType | null
// ) {
//   if (error) {
//     rejectLoginWithError(
//       error,
//       "An error occurred while signing the login message."
//     );
//     return;
//   }
//   if (!loginSignature) {
//     rejectLoginWithError(new Error("Sign message returned no data."));
//     return;
//   }

//   // Important for security! A random session identity is created on each login.
//   const sessionIdentity = Ed25519KeyIdentity.generate();
//   const sessionPublicKey = sessionIdentity.getPublicKey().toDer();

//   if (!contextState.anonymousActor || !get(signerAddress)) {
//     rejectLoginWithError(new Error("Invalid actor or address."));
//     return;
//   }

//   if (!prepareLoginOkResponse) {
//     rejectLoginWithError(new Error("Prepare login not called."));
//     return;
//   }

//   // Logging in is a two-step process. First, the signed SIWE message is sent to the backend.
//   // Then, the backend's siwe_get_delegation method is called to get the delegation.

//   let loginOkResponse: LoginOkResponse;
//   try {
//     loginOkResponse = await callLogin(
//       contextState.anonymousActor,
//       loginSignature,
//       get(signerAddress) ? (get(signerAddress) as `0x${string}`) : undefined,
//       sessionPublicKey,
//       prepareLoginOkResponse.nonce
//     );
//   } catch (e) {
//     rejectLoginWithError(e, "Unable to login.");
//     return;
//   }

//   // Call the backend's siwe_get_delegation method to get the delegation.
//   let signedDelegation: ServiceSignedDelegation;
//   try {
//     signedDelegation = await callGetDelegation(
//       contextState.anonymousActor,
//       get(signerAddress) ? (get(signerAddress) as `0x${string}`) : undefined,
//       sessionPublicKey,
//       loginOkResponse.expiration
//     );
//   } catch (e) {
//     rejectLoginWithError(e, "Unable to get identity.");
//     return;
//   }

//   // Create a new delegation chain from the delegation.
//   const delegationChain = createDelegationChain(
//     signedDelegation,
//     loginOkResponse.user_canister_pubkey
//   );

//   // Create a new delegation identity from the session identity and the
//   // delegation chain.
//   const identity = DelegationIdentity.fromDelegation(
//     sessionIdentity,
//     delegationChain
//   );

//   // Save the identity to local storage.
//   saveIdentity(
//     get(signerAddress) as `0x${string}`,
//     sessionIdentity,
//     delegationChain
//   );

//   // Set the identity in state.
//   updateState({
//     loginStatus: "success",
//     identityAddress: get(signerAddress) as `0x${string}`,
//     identity,
//     delegationChain,
//   });

//   loginPromiseHandlers?.resolve(identity);

//   // The signMessage hook is reset so that it can be used again.
//   //TODO: figure this out
//   //reset();
// }

// /**
//  * Initiates the login process. If a SIWE message is not already available, it will be
//  * generated by calling prepareLogin.
//  *
//  * @returns {void} Login does not return anything. If an error occurs, the error is available in
//  * the loginError property.
//  */

// async function login() {
//   const promise = new Promise<DelegationIdentity>((resolve, reject) => {
//     loginPromiseHandlers = { resolve, reject };
//   });
//   // Set the promise handlers immediately to ensure they are available for error handling.

//   if (!contextState.anonymousActor) {
//     rejectLoginWithError(
//       new Error(
//         "Hook not initialized properly. Make sure to supply all required props to the SiweIdentityProvider."
//       )
//     );
//     return promise;
//   }
//   if (!get(signerAddress)) {
//     rejectLoginWithError(
//       new Error(
//         "No Ethereum address available. Call login after the user has connected their wallet."
//       )
//     );
//     return promise;
//   }
//   if (contextState.prepareLoginStatus === "preparing") {
//     rejectLoginWithError(
//       new Error("Don't call login while prepareLogin is running.")
//     );
//     return promise;
//   }

//   updateState({
//     loginStatus: "logging-in",
//     loginError: undefined,
//   });

//   try {
//     // The SIWE message can be prepared in advance, or it can be generated as part of the login process.
//     let prepareLoginOkResponse = contextState.prepareLoginOkResponse;
//     if (!prepareLoginOkResponse) {
//       prepareLoginOkResponse = await prepareLogin();
//       if (!prepareLoginOkResponse) {
//         throw new Error("Prepare login failed did not return a SIWE message.");
//       }
//     }

//     signMessage(
//       { message: prepareLoginOkResponse.siwe_message },
//       {
//         onSettled: onLoginSignatureSettled,
//       }
//     );
//   } catch (e) {
//     rejectLoginWithError(e);
//   }

//   return promise;
// }

// /**
//  * Clears the state and local storage. Effectively "logs the user out".
//  */
// function clear() {
//   updateState({
//     isInitializing: false,
//     prepareLoginStatus: "idle",
//     prepareLoginError: undefined,
//     prepareLoginOkResponse: undefined,
//     loginStatus: "idle",
//     loginError: undefined,
//     identity: undefined,
//     identityAddress: undefined,
//     delegationChain: undefined,
//   });
//   prepareLoginOkResponse = undefined;
//   clearIdentity();
// }

// /**
//  * Load the identity from local storage on mount.
//  */
// //TODO DELET
// // useEffect(() => {
// //   try {
// //     const [a, i, d] = loadIdentity();
// //     updateState({
// //       identityAddress: a,
// //       identity: i,
// //       delegationChain: d,
// //       isInitializing: false,
// //     });
// //   } catch (e) {
// //     if (e instanceof Error) {
// //       console.log("Could not load identity from local storage: ", e.message);
// //     }
// //     updateState({
// //       isInitializing: false,
// //     });
// //   }
// // }, [updateState]);

// $effect(() => {
//   try {
//     const [a, i, d] = loadIdentity();
//     updateState({
//       identityAddress: a,
//       identity: i,
//       delegationChain: d,
//       isInitializing: false,
//     });
//   } catch (e) {
//     if (e instanceof Error) {
//       console.log("Could not load identity from local storage: ", e.message);
//     }
//     updateState({
//       isInitializing: false,
//     });
//   }
// });

// /**
//  * On address change, reset the state. Action is conditional on state.isInitializing
//  * being false.
//  */
// //TODO DELET
// // useEffect(() => {
// //   if (state.isInitializing) return;
// //   clear();
// //   // eslint-disable-next-line react-hooks/exhaustive-deps
// // }, [connectedEthAddress]);

// $effect(() => {
//   connectedEthAddress;
//   if (state.isInitializing) return;
//   clear();
// });

// /**
//  * Create an anonymous actor on mount. This actor is used during the login
//  * process.
//  */
// //TODO DELET
// // useEffect(() => {
// //   const a = createAnonymousActor({
// //     idlFactory,
// //     canisterId,
// //     httpAgentOptions,
// //     actorOptions,
// //   });
// //   updateState({
// //     anonymousActor: a,
// //   });
// // }, [idlFactory, canisterId, httpAgentOptions, actorOptions, updateState]);

// $effect(() => {
//   const a = createAnonymousActor({
//     idlFactory,
//     canisterId,
//     httpAgentOptions,
//     actorOptions,
//   });
//   updateState({
//     anonymousActor: a,
//   });
// });

// setContext('siwe', 
//       {
//         ...state,
//         prepareLogin,
//         isPreparingLogin: state.prepareLoginStatus === "preparing",
//         isPrepareLoginError: state.prepareLoginStatus === "error",
//         isPrepareLoginSuccess: state.prepareLoginStatus === "success",
//         isPrepareLoginIdle: state.prepareLoginStatus === "idle",
//         login,
//         isLoggingIn: state.loginStatus === "logging-in",
//         isLoginError: state.loginStatus === "error",
//         isLoginSuccess: state.loginStatus === "success",
//         isLoginIdle: state.loginStatus === "idle",
//         signMessageStatus,
//         signMessageError,
//         clear,
//       }
// );

// //   return (
// //     <SiweIdentityContext.Provider
// //       value={{
// //         ...state,
// //         prepareLogin,
// //         isPreparingLogin: state.prepareLoginStatus === "preparing",
// //         isPrepareLoginError: state.prepareLoginStatus === "error",
// //         isPrepareLoginSuccess: state.prepareLoginStatus === "success",
// //         isPrepareLoginIdle: state.prepareLoginStatus === "idle",
// //         login,
// //         isLoggingIn: state.loginStatus === "logging-in",
// //         isLoginError: state.loginStatus === "error",
// //         isLoginSuccess: state.loginStatus === "success",
// //         isLoginIdle: state.loginStatus === "idle",
// //         signMessageStatus,
// //         signMessageError,
// //         clear,
// //       }}
// //     >
// //       {children}
// //     </SiweIdentityContext.Provider>
// //   );
// // }

// let { children } = $props();
</script>
<!-- 
{@render children?.()} -->