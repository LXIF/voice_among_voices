import type {
  SimulationParameters,
  VoiceNodeEgress,
  AudioParameters,
} from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
import { writable } from "svelte/store";
import { Tween } from "svelte/motion";
import { elasticOut, cubicOut, cubicInOut, sineInOut } from "svelte/easing";
import { browser } from "$app/environment";

if (browser) {
  console.log("[Debug] uxState.svelte.ts module initialization");
}

export const selectedAngle = writable<number>(0);
export const hoveredAngle = writable<number | null>(null);
export const currentVoiceBlob = writable<Blob | undefined>(undefined);
export const dragging = writable<boolean>(false);
export const playheadPosition = new Tween(0, {
  easing: sineInOut,
  duration: 50,
});
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
  easing: cubicOut,
  duration: 800,
});
export const walletAddress = writable<string>("");
export const loadingFile = writable(false);
export const loadingProgress = new Tween(0, {
  easing: cubicInOut,
  duration: 500,
});
export const loadingVoices = writable<boolean>(false);

export const resetUxState = () => {
  selectedAngle.set(0);
  hoveredAngle.set(null);
  currentVoiceBlob.set(undefined);
  dragging.set(false);
  playheadPosition.set(0);
  externalPlaybackPosition.set(0);
  angle.set(0);
  fileLoaded.set(false);
  backendSimulationResult.set([]);
  myAddress.set("");
  myTokens.set([]);
  mapRotation.set(0);
  walletAddress.set("");
  loadingFile.set(false);
  loadingProgress.set(0);
  loadingVoices.set(false);
};
