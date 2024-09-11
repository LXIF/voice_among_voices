// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface Platform {}
  }

  type VoiceNodeIngress = import('../../declarations/voice_among_voices_backend/voice_among_voices_backend.did').VoiceNodeIngress;
  type VoiceNodeEgress = import('../../declarations/voice_among_voices_backend/voice_among_voices_backend.did').VoiceNodeEgress;
  type SimulationParameters = import('../../declarations/voice_among_voices_backend/voice_among_voices_backend.did').SimulationParameters;
}

export {};
