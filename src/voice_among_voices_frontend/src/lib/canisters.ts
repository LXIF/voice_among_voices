import {
    createActor,
    canisterId,
} from "../../../declarations/voice_among_voices_backend";
import { HttpAgent, SignIdentity } from "@dfinity/agent";
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
        },
    ) as ActorType;
}

const buildingOrTesting = building || process.env.NODE_ENV === "test";

export const identityAgent: Writable<HttpAgent | undefined> = writable();

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

export const setIdentityAgent = async (
    newIdentity: SignIdentity | undefined,
) => {
    if (newIdentity) {
        identityAgent.set(await HttpAgent.create({ identity: newIdentity }));
    } else {
        identityAgent.set(newIdentity);
    }
};
