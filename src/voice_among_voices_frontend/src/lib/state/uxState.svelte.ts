import type {
  SimulationParameters,
  VoiceNodeEgress,
  AudioParameters,
} from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
import { writable } from "svelte/store";
import { Tween } from "svelte/motion";
import { elasticOut } from "svelte/easing";

export const selectedAngle = writable<number | null>(null);
export const hoveredAngle = writable<number | null>(null);
export const currentVoiceBlob = writable<Blob | undefined>(undefined);
export const dragging = writable<boolean>(false);
export const playheadPosition = writable<number>(0);
export const externalPlaybackPosition = writable<number>(0);
export const angle = writable<number>(0);
export const fileLoaded = writable<boolean>(false);
export const voiceNodes = writable<VoiceNodeEgress[]>([]);
export const backendSimulationResult = writable<VoiceNodeEgress[]>([]);
export const simulationParameters = writable<SimulationParameters | undefined>(
  undefined
);
export const audioParameters = writable<AudioParameters | undefined>(undefined);
export const myAddress = writable<string>("");
export const myTokens = writable<number[]>([]);
export const loadingTokens = writable<boolean>(false);
export const mapRotation = new Tween(0, {
  easing: elasticOut,
  duration: 800,
});
export let walletAddress = writable<string>("");
