<script lang="ts">
import { type ActorConfig, type HttpAgentOptions } from "@dfinity/agent";
import { DelegationIdentity, Ed25519KeyIdentity } from "@dfinity/identity";
// import type { SiweIdentityContextType } from "./context.type";
// import { useAccount, useSignMessage } from "wagmi";
import { IDL } from "@dfinity/candid";
import type {
  LoginOkResponse,
  PrepareLoginOkResponse,
  SIWE_IDENTITY_SERVICE,
  SignedDelegation as ServiceSignedDelegation,
} from "./service.interface";
import { clearIdentity, loadIdentity, saveIdentity } from "./local-storage";
import {
  callGetDelegation,
  callLogin,
  callPrepareLogin,
  createAnonymousActor,
} from "./siwe-provider";
import type { State } from "./state.type";
// import type { SignMessageErrorType } from "wagmi/actions";
import { type SignMessageErrorType } from "@wagmi/core";
import { createDelegationChain } from "./delegation";
import { normalizeError } from "./error";

import { setContext } from "svelte";
import { onMount } from "svelte";
// import { wagmiConfig } from "$lib/wagmi/wagmiStores";
import { appkitModal, wagmiConfig } from "$lib/appKit.svelte";
let { children, idlFactory, canisterId, httpAgentOptions, actorOptions }: { children: any, idlFactory: IDL.InterfaceFactory, canisterId: string, httpAgentOptions?: HttpAgentOptions, actorOptions?: ActorConfig} = $props();

//   /** Configuration options for the HTTP agent used to communicate with the Internet Computer network. */
//   httpAgentOptions?: HttpAgentOptions;

//   /** Configuration options for the actor. These options are passed to the actor upon its creation. */
//   actorOptions?: ActorConfig;

//   /** The Interface Description Language (IDL) factory for the canister. This factory is used to create an actor interface for the canister. */
//   idlFactory: IDL.InterfaceFactory;

//   /** The unique identifier of the canister on the Internet Computer network. This ID is used to establish a connection to the canister. */
//   canisterId: string;

//   /** The child components that the SiweIdentityProvider will wrap. This allows any child component to access the authentication context provided by the SiweIdentityProvider. */
//   children: ReactNode;
// }) {
//   const { address: connectedEthAddress } = useAccount();


import { signMessage } from "@wagmi/core";

let prepareLoginOkResponse = $state<PrepareLoginOkResponse | undefined>(
  undefined
);

//TODO DELET
// const [state, setState] = useState<State>({
//   isInitializing: true,
//   prepareLoginStatus: "idle",
//   loginStatus: "idle",
// });

let contextState = $state<State>({
  isInitializing: true,
  prepareLoginStatus: "idle",
  loginStatus: "idle",
});

const updateState = (newState: Partial<State>) => {
  contextState = { ...contextState, ...newState };
};

// Keep track of the promise handlers for the login method during the async login process.
//TODO DELET
// const loginPromiseHandlers = useRef<{
//   resolve: (
//     value: DelegationIdentity | PromiseLike<DelegationIdentity>
//   ) => void;
//   reject: (error: Error) => void;
// } | null>(null);

let loginPromiseHandlers = $state<{
  resolve: (
    value: DelegationIdentity | PromiseLike<DelegationIdentity>
  ) => void;
  reject: (error: Error) => void;
} | null>(null);



/**
 * Load a SIWE message from the provider, to be used for login. Calling prepareLogin
 * is optional, as it will be called automatically on login if not called manually.
 */
 async function prepareLogin(): Promise<PrepareLoginOkResponse | undefined> {
  if (!contextState.anonymousActor) {
    throw new Error(
      "Hook not initialized properly. Make sure to supply all required props to the SiweIdentityProvider."
    );
  }
  if (!$appkitModal.getAddress()) {
    throw new Error(
      "No Ethereum address available. Call prepareLogin after the user has connected their wallet."
    );
  }

  updateState({
    prepareLoginStatus: "preparing",
    prepareLoginError: undefined,
  });

  try {
    const response = await callPrepareLogin(
      contextState.anonymousActor,
      $appkitModal.getAddress() ? ($appkitModal.getAddress() as `0x${string}`) : undefined
    );
    updateState({
      prepareLoginOkResponse: response,
      prepareLoginStatus: "success",
    });
    prepareLoginOkResponse = response;
    return response;
  } catch (e) {
    const error = normalizeError(e);
    console.error(error);
    updateState({
      prepareLoginStatus: "error",
      prepareLoginError: error,
    });
  }
}

async function rejectLoginWithError(error: Error | unknown, message?: string) {
  const e = normalizeError(error);
  const errorMessage = message || e.message;

  console.error(e);

  updateState({
    prepareLoginOkResponse: undefined,
    loginStatus: "error",
    loginError: new Error(errorMessage),
  });

  loginPromiseHandlers?.reject(new Error(errorMessage));
}


/**
 * This function is called when the signMessage hook has settled, that is, when the
 * user has signed the message or canceled the signing process.
 */
 async function onLoginSignatureSettled(
  loginSignature: `0x${string}` | undefined,
  error: SignMessageErrorType | null
) {
  if (error) {
    rejectLoginWithError(
      error,
      "An error occurred while signing the login message."
    );
    return;
  }
  if (!loginSignature) {
    rejectLoginWithError(new Error("Sign message returned no data."));
    return;
  }

  // Important for security! A random session identity is created on each login.
  const sessionIdentity = Ed25519KeyIdentity.generate();
  const sessionPublicKey = sessionIdentity.getPublicKey().toDer();

  if (!contextState.anonymousActor || !$appkitModal.getAddress()) {
    rejectLoginWithError(new Error("Invalid actor or address."));
    return;
  }

  if (!prepareLoginOkResponse) {
    rejectLoginWithError(new Error("Prepare login not called."));
    return;
  }

  // Logging in is a two-step process. First, the signed SIWE message is sent to the backend.
  // Then, the backend's siwe_get_delegation method is called to get the delegation.

  let loginOkResponse: LoginOkResponse;
  try {
    loginOkResponse = await callLogin(
      contextState.anonymousActor,
      loginSignature,
      $appkitModal.getAddress() ? ($appkitModal.getAddress() as `0x${string}`) : undefined,
      sessionPublicKey,
      prepareLoginOkResponse.nonce
    );
  } catch (e) {
    rejectLoginWithError(e, "Unable to login.");
    return;
  }

  // Call the backend's siwe_get_delegation method to get the delegation.
  let signedDelegation: ServiceSignedDelegation;
  try {
    signedDelegation = await callGetDelegation(
      contextState.anonymousActor,
      $appkitModal.getAddress() ? ($appkitModal.getAddress() as `0x${string}`) : undefined,
      sessionPublicKey,
      loginOkResponse.expiration
    );
  } catch (e) {
    rejectLoginWithError(e, "Unable to get identity.");
    return;
  }

  // Create a new delegation chain from the delegation.
  const delegationChain = createDelegationChain(
    signedDelegation,
    loginOkResponse.user_canister_pubkey
  );

  // Create a new delegation identity from the session identity and the
  // delegation chain.
  const identity = DelegationIdentity.fromDelegation(
    sessionIdentity,
    delegationChain
  );

  // Save the identity to local storage.
  saveIdentity(
    $appkitModal.getAddress() as `0x${string}`,
    sessionIdentity,
    delegationChain
  );

  // Set the identity in state.
  updateState({
    loginStatus: "success",
    identityAddress: $appkitModal.getAddress() as `0x${string}`,
    identity,
    delegationChain,
  });

  loginPromiseHandlers?.resolve(identity);

  // The signMessage hook is reset so that it can be used again.
  //TODO: figure this out
  // reset();
}


/**
 * Initiates the login process. If a SIWE message is not already available, it will be
 * generated by calling prepareLogin.
 *
 * @returns {void} Login does not return anything. If an error occurs, the error is available in
 * the loginError property.
 */

 async function login() {
  const promise = new Promise<DelegationIdentity>((resolve, reject) => {
    loginPromiseHandlers = { resolve, reject };
  });
  // Set the promise handlers immediately to ensure they are available for error handling.

  if (!contextState.anonymousActor) {
    rejectLoginWithError(
      new Error(
        "Hook not initialized properly. Make sure to supply all required props to the SiweIdentityProvider."
      )
    );
    return promise;
  }
  if (!$appkitModal.getAddress()) {
    rejectLoginWithError(
      new Error(
        "No Ethereum address available. Call login after the user has connected their wallet."
      )
    );
    return promise;
  }
  if (contextState.prepareLoginStatus === "preparing") {
    rejectLoginWithError(
      new Error("Don't call login while prepareLogin is running.")
    );
    return promise;
  }

  updateState({
    loginStatus: "logging-in",
    loginError: undefined,
  });

  try {
    // The SIWE message can be prepared in advance, or it can be generated as part of the login process.
    let prepareLoginOkResponse = contextState.prepareLoginOkResponse;
    if (!prepareLoginOkResponse) {
      prepareLoginOkResponse = await prepareLogin();
      if (!prepareLoginOkResponse) {
        throw new Error("Prepare login failed did not return a SIWE message.");
      }
    }

    console.log($wagmiConfig);

    const signature = await signMessage( // TODO: maybe handle user error better
        $wagmiConfig,
        {
            message: prepareLoginOkResponse.siwe_message,
            connector: $wagmiConfig.connectors[0] //TODO
        }
    );
    onLoginSignatureSettled(signature, null);
  } catch (e) {
    rejectLoginWithError(e);
  }

  return promise;
}

/**
 * Clears the state and local storage. Effectively "logs the user out".
 */
function clear() {
  updateState({
    isInitializing: false,
    prepareLoginStatus: "idle",
    prepareLoginError: undefined,
    prepareLoginOkResponse: undefined,
    loginStatus: "idle",
    loginError: undefined,
    identity: undefined,
    identityAddress: undefined,
    delegationChain: undefined,
  });
  prepareLoginOkResponse = undefined;
  clearIdentity();
}


/**
 * Load the identity from local storage on mount.
 */
//TODO DELET
// useEffect(() => {
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
// }, [updateState]);

onMount(() => {
  try {
    const [a, i, d] = loadIdentity();
    updateState({
      identityAddress: a,
      identity: i,
      delegationChain: d,
      isInitializing: false,
    });
  } catch (e) {
    if (e instanceof Error) {
      console.log("Could not load identity from local storage: ", e.message);
    }
    updateState({
      isInitializing: false,
    });
  }
});

/**
 * On address change, reset the state. Action is conditional on state.isInitializing
 * being false.
 */
//TODO DELET
// useEffect(() => {
//   if (state.isInitializing) return;
//   clear();
//   // eslint-disable-next-line react-hooks/exhaustive-deps
// }, [connectedEthAddress]);

$effect(() => {
  if($appkitModal.getAddress()) {
    if (contextState.isInitializing) return;
    clear();
  }
});

/**
 * Create an anonymous actor on mount. This actor is used during the login
 * process.
 */
//TODO DELET
// useEffect(() => {
//   const a = createAnonymousActor({
//     idlFactory,
//     canisterId,
//     httpAgentOptions,
//     actorOptions,
//   });
//   updateState({
//     anonymousActor: a,
//   });
// }, [idlFactory, canisterId, httpAgentOptions, actorOptions, updateState]);

onMount(() => {
  const a = createAnonymousActor({
    idlFactory,
    canisterId,
    httpAgentOptions,
    actorOptions,
  });
  updateState({
    anonymousActor: a,
  });
});

let isPreparingLogin = $derived(contextState.prepareLoginStatus === "preparing");
let isPrepareLoginError = $derived(contextState.prepareLoginStatus === "error");
let isPrepareLoginSuccess = $derived(contextState.prepareLoginStatus === "success");
let isPrepareLoginIdle = $derived(contextState.prepareLoginStatus === "idle");

let isLoggingIn = $derived(contextState.loginStatus === "logging-in");
let isLoginError = $derived(contextState.loginStatus === "error");
let isLoginSuccess = $derived(contextState.loginStatus === "success");
let isLoginIdle = $derived(contextState.loginStatus === "idle");


function setSiweContext() {
  setContext('siwe', 
          {
            ...contextState,
            prepareLogin,
            isPreparingLogin,
            isPrepareLoginError,
            isPrepareLoginSuccess,
            isPrepareLoginIdle,
            login,
            isLoggingIn,
            isLoginError,
            isLoginSuccess,
            isLoginIdle,
            signMessageStatus: "todo", //TODO
            signMessageError: "todo",
            clear,
          }
    );
}

setSiweContext();
</script>


{@render children?.()}