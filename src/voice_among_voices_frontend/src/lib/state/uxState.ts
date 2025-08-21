import type {
    SimulationParameters,
    VoiceNodeEgress,
    AudioParameters,
} from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
import { derived, get, writable } from "svelte/store";
import { Tween, Spring } from "svelte/motion";
import { elasticOut, cubicOut, cubicInOut, sineInOut } from "svelte/easing";
import { browser } from "$app/environment";
import { getVoiceNodes } from "$lib/icInteractions";
import { backend } from "$lib/canisters";

export const selectedAngle = writable<number>(0);
export const hoveredAngle = writable<number | null>(null);
export const currentVoiceBlob = writable<Blob | null>(null);
export const playheadPosition = new Tween(0, {
    easing: sineInOut,
    duration: 50,
});
export const externalPlaybackPosition = writable<number>(0);
export const angle = writable<number>(0);
export const fileLoaded = writable<boolean>(false);
export const voiceNodes = writable<VoiceNodeEgress[]>([]);
export const backendSimulationResult = writable<VoiceNodeEgress[]>([]);
export const simulationParameters = writable<SimulationParameters | null>(null);
export const audioParameters = writable<AudioParameters | null>(null);
export const myAddress = writable<string>("");
export const myTokens = writable<number[]>([]);
export const adminTokenId = writable<number>(0);

export const isAdmin = derived([myTokens, adminTokenId], () => {
    if (get(myTokens).length === 0) return false;
    return get(myTokens).includes(get(adminTokenId));
});
export const selectedManagementNode = writable<number>(1);
export const mapRotation = new Spring(0, {
    stiffness: 0.3,
    damping: 0.4,
});
export const walletAddress = writable<string>("");
export const loadingProgress = new Tween(0, {
    easing: cubicInOut,
    duration: 500,
});
export const sampleLength = writable<number>(0);
export const nodeWidthPx = writable<number>(0);
export const nodeWidthLogical = writable<number>(0);
export const hasNoTokens = writable<boolean>(false);
export const showInfoModal = writable<boolean>(false);
export const showParticipationOverlay = writable<boolean>(false);
export const showCensorModal = writable<boolean>(false);
export const toastMessage = writable<string>("");

export let buyTag = writable<{ x: number; y: number; angle: number } | null>(
    null,
);

export const resetUxState = () => {
    selectedAngle.set(0);
    hoveredAngle.set(null);
    currentVoiceBlob.set(null);
    playheadPosition.set(0);
    externalPlaybackPosition.set(0);
    angle.set(0);
    fileLoaded.set(false);
    backendSimulationResult.set([]);
    myAddress.set("");
    myTokens.set([]);
    mapRotation.set(0);
    walletAddress.set("");
    loadingProgress.set(0);
    hasNoTokens.set(false);
};

export const applicationStates = {
    loggedOut: {
        state: "loggedOut" as const,
        physicsActive: false,
        wheelActive: false,
        recorderActive: false,
        recorderVisible: false,
        droppingActive: false,
        showDraggableNode: false,
        showLoadingAnimation: false,
        showBackendResult: false,
        showFileLoadingLine: false,
    },
    loadingNodes: {
        state: "loadingNodes" as const,
        physicsActive: false,
        wheelActive: false,
        recorderActive: false,
        recorderVisible: false,
        droppingActive: false,
        showDraggableNode: false,
        showLoadingAnimation: true,
        showBackendResult: false,
        showFileLoadingLine: false,
    },
    loadingTokens: {
        state: "loadingTokens" as const,
        physicsActive: false,
        wheelActive: false,
        recorderActive: false,
        recorderVisible: false,
        droppingActive: false,
        showDraggableNode: false,
        showLoadingAnimation: true,
        showBackendResult: false,
        showFileLoadingLine: false,
    },
    loadingFile: {
        state: "loadingFile" as const,
        physicsActive: false,
        wheelActive: false,
        recorderActive: false,
        recorderVisible: true,
        droppingActive: false,
        showDraggableNode: false,
        showLoadingAnimation: false,
        showBackendResult: false,
        showFileLoadingLine: true,
    },
    recordingVoice: {
        state: "recordingVoice" as const,
        physicsActive: false,
        wheelActive: false,
        recorderActive: true,
        recorderVisible: true,
        droppingActive: false,
        showDraggableNode: true,
        showLoadingAnimation: false,
        showBackendResult: false,
        showFileLoadingLine: false,
    },
    playingFile: {
        state: "playingFile" as const,
        physicsActive: false,
        wheelActive: true,
        recorderActive: false,
        recorderVisible: true,
        droppingActive: false,
        showDraggableNode: false,
        showLoadingAnimation: false,
        showBackendResult: false,
        showFileLoadingLine: false,
    },
    rotatingMap: {
        state: "rotatingMap" as const,
        physicsActive: false,
        wheelActive: false,
        recorderActive: false,
        recorderVisible: true,
        droppingActive: false,
        showDraggableNode: false,
        showLoadingAnimation: false,
        showBackendResult: false,
        showFileLoadingLine: false,
    },
    loggedInIdle: {
        state: "loggedInIdle" as const,
        physicsActive: true,
        wheelActive: true,
        recorderActive: true,
        recorderVisible: true,
        droppingActive: true,
        showDraggableNode: true,
        showLoadingAnimation: false,
        showBackendResult: true,
        showFileLoadingLine: false,
    },
    loggedInSimulating: {
        state: "loggedInSimulating" as const,
        physicsActive: true,
        wheelActive: false,
        recorderActive: false,
        recorderVisible: true,
        droppingActive: false,
        showDraggableNode: false,
        showLoadingAnimation: false,
        showBackendResult: true,
        showFileLoadingLine: false,
    },
    loadingBackendResult: {
        state: "loadingBackendResult" as const,
        physicsActive: true,
        wheelActive: false,
        recorderActive: false,
        recorderVisible: true,
        droppingActive: false,
        showDraggableNode: false,
        showLoadingAnimation: true,
        showBackendResult: true,
        showFileLoadingLine: false,
    },
    draggingVoice: {
        state: "draggingVoice" as const,
        physicsActive: false,
        wheelActive: false,
        recorderActive: false,
        recorderVisible: true,
        droppingActive: true,
        showDraggableNode: true,
        showLoadingAnimation: false,
        showBackendResult: false,
        showFileLoadingLine: false,
    },
};

const createApplicationState = () => {
    const { subscribe, set, update } = writable<ApplicationState>(
        applicationStates.loggedOut,
    );

    const customSet = (value: ApplicationState) => {
        set(value);

        if (value.state === "recordingVoice") {
            backendSimulationResult.set([]);
        }
    };

    return {
        subscribe,
        set: customSet,
        update,
    };
};

export type ApplicationState = {
    state:
        | "loggedOut"
        | "loadingNodes"
        | "loadingTokens"
        | "loadingFile"
        | "recordingVoice"
        | "playingFile"
        | "rotatingMap"
        | "loggedInIdle"
        | "loggedInSimulating"
        | "loadingBackendResult"
        | "draggingVoice";
    physicsActive: boolean;
    wheelActive: boolean;
    recorderActive: boolean;
    recorderVisible: boolean;
    droppingActive: boolean;
    showLoadingAnimation: boolean;
    showFileLoadingLine: boolean;
    showBackendResult: boolean;
    showDraggableNode: boolean;
};

export const applicationState = createApplicationState();

export async function fetchAdminTokenId() {
    adminTokenId.set(Number(await backend.get_admin_id()));
}
