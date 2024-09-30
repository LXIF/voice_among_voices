<script lang="ts">
    import {backend} from '$lib/canisters'; // complains but works
    import {onMount} from 'svelte';
    import DroppableNode from '$lib/components/DroppableNode.svelte';
    import NodeMapPhysics from '$lib/components/NodeMapPhysics.svelte';
    import VoiceRecorder from '$lib/components/VoiceRecorder.svelte';
    import type {
        VoiceNodeEgress,
        SimulationParameters,
        AudioParameters,
    } from '../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did';
    import {usableCanvasWidth} from '$lib/config/nodeMap';
    import {blobToUint8Array} from '$lib/utils/convUtils';

    let voiceNodes: VoiceNodeEgress[] = [];
    let backendSimulationResult: VoiceNodeEgress[] = [];
    let simulationParameters: SimulationParameters;
    let audioParameters: AudioParameters;
    let sampleLength = 0;
    let nodeWidthPx = 0;
    let nodeWidthLogical = 0;
    let currentVoiceBlob: Blob;

    let dragging = false;

    onMount(async () => {
        voiceNodes = await backend.get_voice_nodes();
        simulationParameters = await backend.get_simulation_parameters();
        audioParameters = await backend.get_audio_parameters();
    });

    const handleDropNewNode = async (event: CustomEvent) => {
        const sample = await blobToUint8Array(currentVoiceBlob);
        const {x, y} = event.detail;
        let backend_simulation_result = await backend.add_voice_node({
            x,
            y,
            sample,
        }); // TODO: setup backend for receiving blob, then do this
        // let backend_simulation_result = await backend.add_voice_node({
        //     ...event.detail,
        //     sample: currentVoiceBlob,
        // });
        if (backend_simulation_result.Ok) {
            backendSimulationResult = backend_simulation_result.Ok;
        } else {
            // TODO: potentially provide user feedback
            console.log(backend_simulation_result.Err.NotValidAudioFileError);
        }
        voiceNodes = await backend.get_voice_nodes();
    };

    const handleRecordingLength = (e: CustomEvent) => {
        sampleLength = e.detail;

        const nodeWidths = calculateNodeWidth(
            sampleLength,
            usableCanvasWidth,
            audioParameters.total_length_ms,
            simulationParameters.logical_width
        );

        nodeWidthPx = nodeWidths.nodeWidthPx;
        nodeWidthLogical = nodeWidths.nodeWidthLogical;
    };

    const handleVoiceRecorded = (e: CustomEvent) => {
        currentVoiceBlob = e.detail;
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

<main class="flex justify-center items-center flex-col h-[100vh]">
    <NodeMapPhysics
        nodes={voiceNodes}
        backendNodes={backendSimulationResult}
        on:dropNewNode={handleDropNewNode}
        {dragging}
    />
    <DroppableNode
        {nodeWidthPx}
        {nodeWidthLogical}
        on:dragstart={() => (dragging = true)}
        on:dragend={() => (dragging = false)}
    />
    <VoiceRecorder
        on:recordingLength={handleRecordingLength}
        on:voiceRecorded={handleVoiceRecorded}
    />
</main>
