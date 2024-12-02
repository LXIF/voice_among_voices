import {
  createActor,
  canisterId,
} from "../../../declarations/voice_among_voices_backend";
import { canisterId as IIcanisterId } from "../../../declarations/internet_identity";
import { HttpAgent } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";
import { building } from "$app/environment";

import { writable, type Writable } from "svelte/store";

type ActorType = ReturnType<typeof createActor>;

function dummyActor(): ActorType {
  return new Proxy(
    {},
    {
      get() {
        throw new Error("Canister invoked while building");
      },
    }
  ) as ActorType;
}

const buildingOrTesting = building || process.env.NODE_ENV === "test";

const identityAgent: Writable<HttpAgent | undefined> = writable();

let backend: ActorType = dummyActor();

identityAgent.subscribe((agent) => {
  if (buildingOrTesting) {
    backend = dummyActor();
  } else if (agent === undefined) {
    backend = createActor(canisterId);
  } else {
    backend = createActor(canisterId, { agent });
  }
});

export { backend };

export const loginWithInternetIdentity = async () => {
  let iiUrl;
  if (process.env.DFX_NETWORK === "local") {
    iiUrl = `http://${IIcanisterId}.localhost:4943/`;
  } else if (process.env.DFX_NETWORK === "ic") {
    iiUrl = `https://${IIcanisterId}.ic0.app`;
  } else {
    iiUrl = `https://${IIcanisterId}.dfinity.network`;
  }

  const authClient = await AuthClient.create();

  await new Promise((resolve) => {
    authClient.login({
      identityProvider: iiUrl,
      onSuccess: resolve,
      onError: (e) => console.log(e),
    });
  });

  const identity = authClient.getIdentity();

  identityAgent.set(await HttpAgent.create({ identity }));
};
