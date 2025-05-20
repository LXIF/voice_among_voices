<script lang="ts">
    import {
        showCensorModal,
        selectedManagementNode,
    } from "$lib/state/uxState";
    import Button from "./Button.svelte";
    import Dialog from "./Dialog.svelte";
    import { backend } from "$lib/canisters";
    import { onMount } from "svelte";

    let audioElement: HTMLAudioElement;
    let currentPlayingId: number | null = null;
    let censorSuccess = $state(false);

    async function playVoice(id: number) {
        if (currentPlayingId === id) {
            audioElement.pause();
            currentPlayingId = null;
            return;
        }
        try {
            const response = await backend.get_voice(BigInt(id));
            console.log(response);
            if ("Ok" in response) {
                const audioBlob = new Blob(
                    [new Uint8Array(response.Ok.sample)],
                    {
                        type: "audio/wav",
                    },
                );
                const audioUrl = URL.createObjectURL(audioBlob);
                audioElement.src = audioUrl;
                currentPlayingId = id;
                await audioElement.play();
            } else {
                throw response.Err;
            }
        } catch (error) {
            console.error("Error playing voice:", error);
        }
    }

    async function censorVoice(id: number) {
        try {
            const response = await backend.censor(BigInt(id));
            if ("Ok" in response) {
                censorSuccess = true;
                setTimeout(() => (censorSuccess = false), 2000);
            }
        } catch (error) {
            console.error("Error censoring voice:", error);
        }
    }

    // Clean up audio URL when component is destroyed
    onMount(() => {
        return () => {
            if (audioElement?.src) {
                URL.revokeObjectURL(audioElement.src);
            }
        };
    });
</script>

<Dialog
    class="max-h-[50vh] text-slate-950 dark:text-white"
    onClose={() => ($showCensorModal = false)}
>
    <div class="p-4">
        <h2 class="mb-4 text-xl font-bold">Voice Management</h2>
        <h3 class="text-l mb-4 font-bold">
            Select a node on the map to manage it
        </h3>
        {#if $selectedManagementNode}
            <div class="max-h-[40vh] space-y-2 overflow-y-auto">
                <div
                    class="flex items-center justify-between rounded bg-slate-100 p-2 dark:bg-slate-800"
                >
                    <span class="font-medium"
                        ><span
                            style={`color: hsl(${$selectedManagementNode}, 100%, 50%)`}
                            >●</span
                        >
                        Voice #{$selectedManagementNode}</span
                    >
                    <div class="space-x-2">
                        <Button
                            onclick={() => playVoice($selectedManagementNode!)}
                            class={currentPlayingId === $selectedManagementNode
                                ? "bg-green-500"
                                : ""}
                        >
                            {currentPlayingId === $selectedManagementNode
                                ? "Playing..."
                                : "Play"}
                        </Button>
                        <Button
                            onclick={() =>
                                censorVoice($selectedManagementNode!)}
                            variant="danger"
                        >
                            {censorSuccess ? "Success!" : "Censor"}
                        </Button>
                    </div>
                </div>
            </div>
        {/if}
    </div>
</Dialog>

<audio
    bind:this={audioElement}
    on:ended={() => (currentPlayingId = null)}
    class="hidden"
/>
