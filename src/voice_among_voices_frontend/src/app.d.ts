// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface Platform {}
  }

  type VoiceNode = {
    id: bigint;
    x: bigint;
    y: bigint;
    sample: string;
  };
}

export {};
