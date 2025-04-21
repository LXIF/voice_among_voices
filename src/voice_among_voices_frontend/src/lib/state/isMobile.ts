// stores/device.ts
import { writable } from "svelte/store";

function createDeviceStore() {
  const { subscribe, set } = writable(false);

  function checkDevice() {
    if (typeof window === "undefined") return false;
    const isTouch =
      matchMedia("(pointer:coarse)").matches;
    set(isTouch);
  }

  if (typeof window !== "undefined") {
    // Initial check
    checkDevice();
    // Update on resize
    window.addEventListener("resize", checkDevice);
  }

  return {
    subscribe,
  };
}

export const isTouch = createDeviceStore();
