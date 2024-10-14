// @ts-ignore
import { createActor, canisterId } from 'declarations/voice_among_voices_backend';
import { canisterId as IIcanisterId } from '../../../declarations/internet_identity';
import { HttpAgent } from '@dfinity/agent';
import {AuthClient} from '@dfinity/auth-client';
// @ts-ignore
import { building } from '$app/environment';

import { writable, type Writable, get } from 'svelte/store';
import type { Identity } from '@dfinity/agent';
import { Actor } from '@dfinity/agent';

function dummyActor() {
    return new Proxy({}, { get() { throw new Error("Canister invoked while building"); } });
}

const buildingOrTesting = building || process.env.NODE_ENV === "test";

const identityAgent: Writable<HttpAgent | undefined> = writable();

export const backend = buildingOrTesting
    ? dummyActor()
    : get(identityAgent) === undefined ? createActor(canisterId) : createActor(canisterId, {identity: get(identityAgent)});

export const loginWithInternetIdentity = async () => {
    let iiUrl;
    if (process.env.DFX_NETWORK === 'local') {
        iiUrl = `http://${IIcanisterId}.localhost:4943/`;
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

    const agent = await HttpAgent.create({identity});

    identityAgent.set(createActor(canisterId, {
        agent
    }));
};