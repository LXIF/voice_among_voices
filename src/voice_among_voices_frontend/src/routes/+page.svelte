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
    import {
        blobToUint8Array,
        handleBackendAudioData,
    } from '$lib/utils/convUtils';
    import AngleFileBox from '$lib/components/AngleFileBox.svelte';

    let voiceNodes: VoiceNodeEgress[] = [];
    let backendSimulationResult: VoiceNodeEgress[] = [];
    let simulationParameters: SimulationParameters;
    let audioParameters: AudioParameters;
    let sampleLength = 0;
    let nodeWidthPx = 0;
    let nodeWidthLogical = 0;
    let currentVoiceBlob: Blob;
    let myVoice;

    let myCurrentSampleAudioElement: HTMLAudioElement;

    let dragging = false;

    let playheadPosition = 0;
    let externalPlaybackPosition = 0;
    let angle = 0;
    let fileLoaded = false;

    onMount(async () => {
        voiceNodes = await backend.get_voice_nodes();
        simulationParameters = await backend.get_simulation_parameters();
        audioParameters = await backend.get_audio_parameters();
        myVoice = await backend.get_my_voice();

        if (myVoice.length > 0) {
            const audioURL = await handleBackendAudioData(myVoice[0].sample);
            myCurrentSampleAudioElement.src = audioURL;
        }
    });

    const handleDropNewNode = async (event: CustomEvent) => {
        const sample = await blobToUint8Array(currentVoiceBlob);
        const {x, y} = event.detail;
        let backend_simulation_result = await backend.add_voice_node({
            x,
            y,
            sample,
        });
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
            simulationParameters.logical_radius * 2
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
        showPlayHead={fileLoaded}
        playHeadAngle={angle}
        playHeadPosition={playheadPosition}
        on:movePlayHead={(e) => {
            externalPlaybackPosition = e.detail;
        }}
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
        {audioParameters}
    />
    <h1>Current first voice:</h1>
    <p>TODO: replace with own voice</p>
    <audio
        controls
        bind:this={myCurrentSampleAudioElement}
    ></audio>
    <AngleFileBox
        {externalPlaybackPosition}
        on:playbackPosition={(e) => (playheadPosition = e.detail)}
        on:fileAngle={(e) => (angle = e.detail)}
        on:fileLoaded={(e) => (fileLoaded = e.detail)}
    />
</main>
