import { writable, type Writable } from "svelte/store";
import { FeatureFlag } from "$lib/utils/featureFlags";
import { isNullish } from "@dfinity/utils";

declare global {
    interface Window {
        __featureFlags: Record<string, FeatureFlag>;
    }
}

type FeatureFlagStore = Writable<boolean> & {
    getFeatureFlag: () => FeatureFlag | undefined;
};

const LOCALSTORAGE_FEATURE_FLAGS_PREFIX = "VAV-localstorage-feature-flags__";

const createFeatureFlagStore = (
    name: string,
    defaultValue: boolean,
): FeatureFlagStore => {
    const { subscribe, set, update } = writable(defaultValue);

    // We cannot use browser because this is also imported in our showcase
    if (isNullish(globalThis.window)) {
        return {
            subscribe,
            set,
            update,
            getFeatureFlag: () => undefined,
        };
    }

    // Initialize feature flag object with value from localstorage

    const initializedFeatureFlag: FeatureFlag = new FeatureFlag(
        window.localStorage,
        LOCALSTORAGE_FEATURE_FLAGS_PREFIX + name,
        defaultValue,
        { subscribe, set, update },
    );

    // Make feature flags configurable from browser console
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    if (typeof window.__featureFlags === "undefined") {
        window.__featureFlags = {};
    }
    window.__featureFlags[name] = initializedFeatureFlag;

    const getFeatureFlag = () => {
        return initializedFeatureFlag;
    };

    return {
        subscribe,
        set,
        update,
        getFeatureFlag,
    };
};

export const AFFINITY_MODE = createFeatureFlagStore("AFFINITY_MODE", false);

export default {
    AFFINITY_MODE,
} as Record<string, FeatureFlagStore>;
