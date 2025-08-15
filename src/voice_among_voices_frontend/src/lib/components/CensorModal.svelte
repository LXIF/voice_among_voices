<script lang="ts">
    import {
        showCensorModal,
        selectedManagementNode,
        voiceNodes,
    } from "$lib/state/uxState";
    import Button from "./Button.svelte";
    import Dialog from "./Dialog.svelte";
    import { backend } from "$lib/canisters";
    import { onMount } from "svelte";
    import { getVoiceNodes } from "$lib/icInteractions";

    let audioElement: HTMLAudioElement;
    let currentPlayingId: number | null = $state(null);
    let censorState = $state<"idle" | "censoring" | "success" | "failure">(
        "idle",
    );
    let sampleLoading = $state(false);

    let from = $state(0);
    let step = $state(0);

    function incrementManagementNode() {
        const currentId = $selectedManagementNode;
        const availableIds = $voiceNodes.map((node) => Number(node.id));
        const nextId =
            availableIds.find((id) => id > currentId) ??
            Math.min(...availableIds);
        $selectedManagementNode = nextId;
    }
    function decrementManagementNode() {
        const currentId = $selectedManagementNode;
        const availableIds = $voiceNodes.map((node) => Number(node.id));
        const nextId =
            availableIds.findLast((id) => id < currentId) ??
            Math.max(...availableIds);
        $selectedManagementNode = nextId;
    }

    async function playVoice(id: number) {
        if (currentPlayingId === id) {
            audioElement.pause();
            currentPlayingId = null;
            return;
        }
        try {
            sampleLoading = true;
            const response = await backend.get_voice(BigInt(id));
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
                sampleLoading = false;
                await audioElement.play();
            } else {
                sampleLoading = false;
                throw response.Err;
            }
        } catch (error) {
            console.error("Error playing voice:", error);
        }
    }

    async function censorVoice(id: number) {
        censorState = "censoring";
        try {
            const response = await backend.censor(BigInt(id));
            if ("Ok" in response) {
                censorState = "success";
                setTimeout(() => (censorState = "idle"), 2000);
            }
        } catch (error) {
            console.error("Error censoring voice:", error);
            censorState = "failure";
            setTimeout(() => (censorState = "idle"), 2000);
        }
    }

    // Clean up audio URL when component is destroyed
    onMount(() => {
        (async () => {
            $voiceNodes = await getVoiceNodes();
            $selectedManagementNode = Math.min(
                $voiceNodes
                    .map((node) => Number(node.id))
                    .sort((a, b) => a - b)[0],
            );
        })();
        return () => {
            if (audioElement?.src) {
                URL.revokeObjectURL(audioElement.src);
            }
        };
    });

    const handlePopulateAll = async () => {
        console.log("populating all!");
        const res = await backend.populate_with_demo_content(
            BigInt(from),
            BigInt(step),
        );
        console.log(res);
    };
</script>

<Dialog
    class="max-h-[50vh] text-slate-950 dark:text-white"
    onClose={() => ($showCensorModal = false)}
    alwaysOnBottom
>
    <div class="p-4">
        <h2 class="mb-4 text-xl font-bold">Voice Censoring</h2>
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
                        <Button
                            onclick={decrementManagementNode}
                            class="touch-manipulation px-2 py-2">←</Button
                        >
                        <div class="inline-block min-w-16">
                            Voice {$selectedManagementNode}
                        </div>
                        <Button
                            onclick={incrementManagementNode}
                            class="touch-manipulation px-2 py-2">→</Button
                        ></span
                    >
                    <div class="space-x-2">
                        <Button
                            onclick={() => playVoice($selectedManagementNode!)}
                            class={currentPlayingId === $selectedManagementNode
                                ? "bg-green-500"
                                : ""}
                        >
                            {sampleLoading
                                ? "Loading..."
                                : currentPlayingId === $selectedManagementNode
                                  ? "Playing..."
                                  : "Play"}
                        </Button>
                        <Button
                            onclick={() =>
                                censorVoice($selectedManagementNode!)}
                            variant="danger"
                        >
                            {censorState === "idle"
                                ? "Censor"
                                : censorState === "success"
                                  ? "Success!"
                                  : censorState === "censoring"
                                    ? "Censoring..."
                                    : censorState === "failure"
                                      ? "Error"
                                      : ""}
                        </Button>
                    </div>
                </div>
            </div>
        {/if}
    </div>
    <!-- <input type="number" bind:value={from} class="mb-2 text-black" />
    <input type="number" bind:value={step} class="mb-2 text-black" />
    <Button onclick={handlePopulateAll}>Populate all</Button> -->
</Dialog>

<audio
    bind:this={audioElement}
    on:ended={() => (currentPlayingId = null)}
    class="hidden"
/>
