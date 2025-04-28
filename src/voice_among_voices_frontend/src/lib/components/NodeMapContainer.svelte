<script lang="ts">
    import NodeMapPhysics from "./NodeMapPhysics.svelte";
    import NewAngleSelector from "./NewAngleSelector.svelte";

    import { onMount } from "svelte";

    import { identityAgent } from "$lib/canisters";
    import type { VoiceNodeIngress } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import { blobToUint8Array } from "$lib/utils/convUtils";
    import {
        voiceNodes,
        simulationParameters,
        backendSimulationResult,
        walletAddress,
        myTokens,
        selectedAngle,
        hoveredAngle,
        currentVoiceBlob,
        playheadPosition,
        externalPlaybackPosition,
        angle,
        fileLoaded,
        applicationState,
        applicationStates,
        toastMessage,
        sampleLength,
        nodeWidthPx,
        nodeWidthLogical,
    } from "$lib/state/uxState";
    import { blur } from "svelte/transition";
    import { fetchTokens } from "$lib/evm/evmInteractions.svelte";
    import { untrack } from "svelte";
    import {
        getSimulationParameters,
        getVoiceNodes,
        updateVoiceNode,
    } from "$lib/icInteractions";

    let {
        class: classes,
    }: {
        class?: string;
    } = $props();

    let nodeMap = $state<NodeMapPhysics>();

    onMount(async () => {
        $applicationState = applicationStates.loadingNodes;
        $voiceNodes = await getVoiceNodes();

        $applicationState = $identityAgent
            ? applicationStates.loggedInIdle
            : applicationStates.loggedOut;
        $simulationParameters = await getSimulationParameters();
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
            $applicationState = applicationStates.loadingTokens;
        }

        $myTokens = await fetchTokens();
        $applicationState = applicationStates.loggedInIdle;
    }

    const handleDropNewNode = async (voiceNode: VoiceNodeIngress) => {
        try {
            const sample = await blobToUint8Array($currentVoiceBlob!);
            const { x, y, id } = voiceNode;

            let backend_simulation_result = await updateVoiceNode({
                id,
                x,
                y,
                sample,
            });

            if (!backend_simulation_result) {
                $toastMessage = "Failed to update backend";
                return;
            }

            if ("Ok" in backend_simulation_result) {
                $backendSimulationResult = backend_simulation_result.Ok;
                $applicationState = applicationStates.loggedInSimulating;
            } else {
                $applicationState = applicationStates.loggedInIdle;
                $toastMessage = "Failed to drop new node";
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
            $voiceNodes = await getVoiceNodes();
            $currentVoiceBlob = null;
            $sampleLength = 0;
            $nodeWidthPx = 0;
            $nodeWidthLogical = 0;
        } catch (e) {
            $toastMessage = "Failed to drop new node";
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
