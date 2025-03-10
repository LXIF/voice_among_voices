<script lang="ts">
    import NodeMapPhysics from "./NodeMapPhysics.svelte";
    import NewAngleSelector from "./NewAngleSelector.svelte";
    import DroppableNode from "./DroppableNode.svelte";

    import { onMount } from "svelte";

    import { backend, identityAgent } from "$lib/canisters";
    import type { VoiceNodeEgress, SimulationParameters, AudioParameters, VoiceNodeIngress } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import { appkitModal } from "$lib/appKit.svelte";
    import { blobToUint8Array } from "$lib/utils/convUtils";
    import {usableCanvasWidth} from '$lib/config/nodeMap';

    let voiceNodes: VoiceNodeEgress[] = $state([]);
    let backendSimulationResult: VoiceNodeEgress[] = $state([]);
    let simulationParameters: SimulationParameters | undefined = $state();
    let audioParameters: AudioParameters | undefined = $state();
    let sampleLength = $state(0);
    let nodeWidthPx = $state(0);
    let nodeWidthLogical = $state(0);
    let currentVoiceBlob: Blob | undefined = $state();

    let dragging = $state(false);

    let playheadPosition = $state(0);
    let externalPlaybackPosition = $state(0);
    let angle = $state(0);
    let fileLoaded = $state(false);

    let myAddress = $state("");
    let myTokens: number[] = $state([])

    let loadingTokens = $state(false);

    let myPrincipal: string | undefined = $state();

    let selectedAngle: number | null = $state(null);
    let hoveredAngle: number | null = $state(null);

    let { class: classes } : { class?: string } = $props();

    onMount(async () => {
        voiceNodes = await backend.get_voice_nodes();
        simulationParameters = await backend.get_simulation_parameters();
        audioParameters = await backend.get_audio_parameters();
    });

    $effect(() => {if($identityAgent) {
        myAddress = $appkitModal.getAddress()!
    }});

    $effect(() => {
        if(myAddress !== "") {
            fetchOwnedTokens()
        }
    });

    async function getOwnedTokensWithRetry(attempt = 1, maxRetries = 3): Promise<number[]> {
        const response = await backend.get_owned_tokens();
        
        if ('Ok' in response) {
            return response.Ok.map((token) => Number(token));
        }

        console.log("error loading tokens, retrying: ", response.Err);
        
        if (attempt < maxRetries) {
            await new Promise(resolve => setTimeout(resolve, 1000));
            return getOwnedTokensWithRetry(attempt + 1, maxRetries);
        }
        
        console.error('Failed to fetch owned tokens after', maxRetries, 'attempts');
        return [];
    }

    async function fetchOwnedTokens() {
        if(myTokens.length === 0) {
            loadingTokens = true;
        }

        myTokens = await getOwnedTokensWithRetry();
        loadingTokens = false;
    }

    const handleDropNewNode = async (voiceNode: VoiceNodeIngress) => {
        const sample = await blobToUint8Array(currentVoiceBlob!);
        const {x, y, id} = voiceNode;

        let backend_simulation_result = await backend.update_voice_node({
            id,
            x,
            y,
            sample,
        });

        if ('Ok' in backend_simulation_result) {
            backendSimulationResult = backend_simulation_result.Ok;
        } else {
            if ('NotValidAudioFileError' in backend_simulation_result.Err) {
                console.log(backend_simulation_result.Err.NotValidAudioFileError);
            } else if ('NotWithinCircleError' in backend_simulation_result.Err) {
                console.log('Not within circle error');
            }
        }
        voiceNodes = await backend.get_voice_nodes();
    };

    const calculateNodeWidth = (
        sampleLength: number,
        canvasWidth: number,
        totalLength: number,
        logicalWidth: number
    ) => {
        const pixelPerMs = canvasWidth / totalLength;
        const logicalPerMs = logicalWidth / totalLength;

        return {
            nodeWidthPx: sampleLength * pixelPerMs,
            nodeWidthLogical: sampleLength * logicalPerMs,
        };
    };
</script>

<div class="absolute w-full h-full flex justify-center items-center">
    <NodeMapPhysics
        nodes={voiceNodes}
        backendNodes={backendSimulationResult}
        dropNewNode={handleDropNewNode}
        {dragging}
        showPlayHead={fileLoaded}
        playHeadAngle={angle}
        playHeadPosition={playheadPosition}
        movePlayHead={(normalizedPosition) => {
            externalPlaybackPosition = normalizedPosition;
        }}
        class="w-full h-full lg:max-w-[600px]"
    />
    <NewAngleSelector
        availableAngles={myTokens}
        nodes={voiceNodes}
        loading={loadingTokens}
        class="absolute top-0 w-full h-full lg:max-w-[600px]"
        onSelectAngle={(angle) => selectedAngle = angle}
        onHoverAngle={(angle) => hoveredAngle = angle}
    />
</div>
<div>
    {#if hoveredAngle || selectedAngle}
    <p>{hoveredAngle ? hoveredAngle : selectedAngle}</p>
    {/if}
    {#if selectedAngle}
    <DroppableNode
        {nodeWidthPx}
        {nodeWidthLogical}
        ondragstart={() => (dragging = true)}
        ondragend={() => (dragging = false)}
    />
    {/if}
</div>