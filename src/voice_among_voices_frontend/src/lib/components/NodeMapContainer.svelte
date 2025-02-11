<script lang="ts">
    import NodeMapPhysics from "./NodeMapPhysics.svelte";
    import NewAngleSelector from "./NewAngleSelector.svelte";
    import DroppableNode from "./DroppableNode.svelte";

    import { onMount } from "svelte";

    import { backend } from "$lib/canisters";
    import type { VoiceNodeEgress, SimulationParameters, AudioParameters, VoiceNodeIngress } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";

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

    let myPrincipal: string | undefined = $state();

    onMount(async () => {
        voiceNodes = await backend.get_voice_nodes();
        simulationParameters = await backend.get_simulation_parameters();
        audioParameters = await backend.get_audio_parameters();
    });

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

<div class="relative">
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
    />
    <NewAngleSelector availableAngles={[10]} nodes={voiceNodes} class="fixed top-0 right-0" />
    <DroppableNode
        {nodeWidthPx}
        {nodeWidthLogical}
        ondragstart={() => (dragging = true)}
        ondragend={() => (dragging = false)}
    />
</div>