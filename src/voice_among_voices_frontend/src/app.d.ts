// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface Platform {}
  }

  type VoiceNodeIngress = {
    x: number;
    y: number;
    sample: string;
  };

  type VoiceNodeEgress = {
    id: bigint;
    x: number;
    y: number;
  };
}

export {};
