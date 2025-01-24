import {
  createActor,
  canisterId,
} from "../../../declarations/voice_among_voices_backend";
import { canisterId as IIcanisterId } from "../../../declarations/internet_identity";
import { HttpAgent, SignIdentity } from "@dfinity/agent";
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

export const setIdentityAgent = async (newIdentity: SignIdentity) => {
  identityAgent.set(await HttpAgent.create({ identity: newIdentity }));
};
