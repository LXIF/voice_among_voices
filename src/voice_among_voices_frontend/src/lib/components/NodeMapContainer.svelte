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
    } from "$lib/state/uxState";
    import { blur } from "svelte/transition";
    import { fetchTokens } from "$lib/evm/evmInteractions.svelte";
    import { untrack } from "svelte";
    import { withRetry } from "$lib/utils/commsUtils";

    let { class: classes }: { class?: string } = $props();

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

    $effect(() => {
        if ($identityAgent) {
            $walletAddress = $appkitModal!.getAddress()!;
        }
    });

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
        const sample = await blobToUint8Array($currentVoiceBlob!);
        const { x, y, id } = voiceNode;

        let backend_simulation_result = await backend.update_voice_node({
            id,
            x,
            y,
            sample,
        });

        if ("Ok" in backend_simulation_result) {
            $backendSimulationResult = backend_simulation_result.Ok;
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
    };
</script>

<div class="w-screen flex items-center justify-center">
    <div
        class="max-w-[70svh] w-full max-h-[70svh] h-full flex justify-center items-center scale-100"
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
            class="w-full h-full lg:max-w-[1200px]"
        />
        <NewAngleSelector
            nodes={$voiceNodes}
            loading={$loadingTokens || $loadingVoices}
            loggedIn={!!$identityAgent}
            class="absolute top-0 w-full h-full lg:max-w-[1200px]"
            onSelectAngle={(angle) => {
                $selectedAngle = angle;
            }}
            onHoverAngle={(angle) => ($hoveredAngle = angle)}
        />
        {#if $hoveredAngle}
            <div
                class="absolute top-0 w-full h-full lg:max-w-[1200px] flex justify-center items-center pointer-events-none"
            >
                <h1
                    transition:blur={{ duration: 100 }}
                    class="backdrop-filter text-9xl"
                >
                    {$hoveredAngle}°
                </h1>
            </div>
        {/if}
    </div>
</div>
