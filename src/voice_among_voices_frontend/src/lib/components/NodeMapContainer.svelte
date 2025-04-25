<script lang="ts">
    import NodeMapPhysics from "./NodeMapPhysics.svelte";
    import NewAngleSelector from "./NewAngleSelector.svelte";

    import { onMount } from "svelte";

    import { backend, identityAgent } from "$lib/canisters";
    import type { VoiceNodeIngress } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import { appkitModal } from "$lib/appKit";
    import { blobToUint8Array } from "$lib/utils/convUtils";
    import {
        voiceNodes,
        simulationParameters,
        backendSimulationResult,
        walletAddress,
        myTokens,
        loadingTokens,
        selectedAngle,
        hoveredAngle,
        currentVoiceBlob,
        dragging,
        playheadPosition,
        externalPlaybackPosition,
        angle,
        fileLoaded,
        loadingVoices,
        justDropped,
    } from "$lib/state/uxState";
    import { blur } from "svelte/transition";
    import { fetchTokens } from "$lib/evm/evmInteractions.svelte";
    import { untrack } from "svelte";
    import { withRetry } from "$lib/utils/commsUtils";

    let {
        class: classes,
    }: {
        class?: string;
    } = $props();

    let nodeMap = $state<NodeMapPhysics>();

    onMount(async () => {
        $loadingVoices = true;
        $voiceNodes = await withRetry(() => backend.get_voice_nodes(), {
            maxRetries: 15,
            delayMs: 1000,
            validate: (nodes) => nodes.length > 0,
            onRetry: (attempt) =>
                console.log(`Retrying fetch nodes, attempt ${attempt}...`),
        });

        $loadingVoices = false;
        $simulationParameters = await backend.get_simulation_parameters();
    });

    // $effect(() => {
    //     if ($identityAgent) {
    //         $walletAddress = $appkitModal!.getAddress()!;
    //     }
    // });

    $effect(() => {
        if ($walletAddress !== "") {
            untrack(() => fetchOwnedTokens());
        }
    });

    async function fetchOwnedTokens() {
        let tokens = await $myTokens;
        if (tokens.length === 0) {
            $loadingTokens = true;
        }

        $myTokens = await fetchTokens();
        $loadingTokens = false;
    }

    const handleDropNewNode = async (voiceNode: VoiceNodeIngress) => {
        try {
            $justDropped = true;
            const sample = await blobToUint8Array($currentVoiceBlob!);
            const { x, y, id } = voiceNode;

            let backend_simulation_result = await backend.update_voice_node({
                id,
                x,
                y,
                sample,
            });

            if ("Ok" in backend_simulation_result) {
                if ($justDropped) {
                    $backendSimulationResult = backend_simulation_result.Ok;
                }
            } else {
                if ("NotValidAudioFileError" in backend_simulation_result.Err) {
                    console.log(
                        backend_simulation_result.Err.NotValidAudioFileError,
                    );
                } else if (
                    "NotWithinCircleError" in backend_simulation_result.Err
                ) {
                    console.log("Not within circle error");
                }
            }
            $voiceNodes = await backend.get_voice_nodes();
        } catch (e) {
            console.error("Failed to drop new node, got this: ", e);
        }
    };

    export async function handleDrop({
        nodeX,
        nodeY,
        nodeRadius,
    }: {
        nodeX: number;
        nodeY: number;
        nodeRadius: number;
    }) {
        if (nodeMap) {
            nodeMap.handleDrop({ nodeX, nodeY, nodeRadius });
        }
    }
</script>

<div class="flex w-screen items-center justify-center">
    <div
        class="flex h-full max-h-[70svh] w-full max-w-[70svh] scale-100 items-center justify-center"
    >
        <NodeMapPhysics
            nodes={$voiceNodes}
            backendNodes={$backendSimulationResult}
            dropNewNode={handleDropNewNode}
            dragging={$dragging}
            showPlayHead={$fileLoaded}
            playHeadAngle={$angle}
            playHeadPosition={playheadPosition.current}
            movePlayHead={(normalizedPosition) => {
                $externalPlaybackPosition = normalizedPosition;
            }}
            class="h-full w-full lg:max-w-[1200px]"
            bind:this={nodeMap}
        />
        <NewAngleSelector
            nodes={$voiceNodes}
            loading={$loadingTokens || $loadingVoices}
            loggedIn={!!$identityAgent}
            class="absolute top-0 h-full w-full lg:max-w-[1200px]"
            onSelectAngle={(angle) => {
                $selectedAngle = angle;
            }}
            onHoverAngle={(angle) => {
                $hoveredAngle = angle;
            }}
        />
        {#if $hoveredAngle}
            <div
                class="pointer-events-none absolute top-0 flex h-full w-full items-center justify-center lg:max-w-[1200px]"
            >
                <h1
                    transition:blur={{ duration: 100 }}
                    class="text-9xl backdrop-filter"
                >
                    {$hoveredAngle}°
                </h1>
            </div>
        {/if}
    </div>
</div>
