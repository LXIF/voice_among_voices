// @ts-ignore
import { createActor, canisterId } from 'declarations/voice_among_voices_backend';
import { canisterId as IIcanisterId } from '../../../declarations/internet_identity';
import { createActor as createNFTActor, canisterId as NFTcanisterId } from 'declarations/voice_among_voices_nft';
import { HttpAgent } from '@dfinity/agent';
import {AuthClient} from '@dfinity/auth-client';
// @ts-ignore
import { building } from '$app/environment';

import { writable, type Writable, get } from 'svelte/store';
import type { Actor } from '@dfinity/agent';

function dummyActor() {
    return new Proxy({}, { get() { throw new Error("Canister invoked while building"); } });
}

// @ts-ignore
const buildingOrTesting = building || process.env.NODE_ENV === "test";

const identityAgent: Writable<HttpAgent | undefined> = writable();

let backend: Actor | {} = dummyActor();

identityAgent.subscribe(agent => {
    if(buildingOrTesting) {
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
    // @ts-ignore
    if (process.env.DFX_NETWORK === 'local') {
        iiUrl = `http://${IIcanisterId}.localhost:4943/`;
        // @ts-ignore
    } else if (process.env.DFX_NETWORK === 'ic') {
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

    identityAgent.set(await HttpAgent.create({identity}));
};