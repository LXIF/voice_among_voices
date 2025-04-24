<script lang="ts">
    import {
        audioParameters,
        simulationParameters,
        currentVoiceBlob,
        dragging,
        playheadPosition,
        externalPlaybackPosition,
        angle,
        fileLoaded,
        hoveredAngle,
        selectedAngle,
    } from "$lib/state/uxState";
    import DroppableNode from "./DroppableNode.svelte";
    import VoiceRecorder from "./VoiceRecorder.svelte";
    import AngleFileBox from "./AngleFileBox.svelte";
    import { usableCanvasWidth } from "$lib/config/nodeMap";
    import { onMount } from "svelte";
    import { backend } from "$lib/canisters";
    import { identityAgent } from "$lib/canisters";

    const {
        onDropNodeWithRadius,
    }: {
        onDropNodeWithRadius: ({
            nodeX,
            nodeY,
            nodeRadius,
        }: {
            nodeX: number;
            nodeY: number;
            nodeRadius: number;
        }) => void;
    } = $props();

    let sampleLength = $state(0);
    let nodeWidthPx = $state(0);
    let nodeWidthLogical = $state(0);

    onMount(async () => {
        $audioParameters = await backend.get_audio_parameters();
        $simulationParameters = await backend.get_simulation_parameters();
    });

    const calculateNodeWidth = (
        sampleLength: number,
        canvasWidth: number,
        totalLength: number,
        logicalWidth: number,
    ) => {
        const pixelPerMs = canvasWidth / totalLength;
        const logicalPerMs = logicalWidth / totalLength;

        return {
            nodeWidthPx: sampleLength * pixelPerMs,
            nodeWidthLogical: sampleLength * logicalPerMs,
        };
    };

    const handleRecordingLength = (length: number) => {
        if (
            $audioParameters === undefined ||
            $simulationParameters === undefined
        )
            throw "invalid params";

        sampleLength = length;

        if (sampleLength > 0) {
            const nodeWidths = calculateNodeWidth(
                //TODO: make sveltier
                length,
                usableCanvasWidth,
                $audioParameters!.total_length_ms,
                $simulationParameters!.logical_radius * 2,
            );
            nodeWidthPx = nodeWidths.nodeWidthPx;
            nodeWidthLogical = nodeWidths.nodeWidthLogical;
        }
    };

    const handleVoiceRecorded = (blob: Blob) => {
        $currentVoiceBlob = blob;
    };

    const handleDropNode = ({ x, y }: { x: number; y: number }) => {
        onDropNodeWithRadius({
            nodeX: x,
            nodeY: y,
            nodeRadius: nodeWidthLogical / 2,
        }); //TODO maybe other node width
    };

    //TODO: this would require detecting "out"
    // const resetRecording = () => {
    //     $currentVoiceBlob = undefined;
    //     sampleLength = 0;
    //     nodeWidthPx = 0;
    //     nodeWidthLogical = 0;
    // }
</script>

<div>
    {#if $selectedAngle && $identityAgent}
        <div
            class="pointer-events-none absolute bottom-12 flex min-h-32 w-screen items-center justify-between px-8 md:bottom-24 lg:bottom-24"
        >
            <VoiceRecorder
                recordingLength={handleRecordingLength}
                voiceRecorded={handleVoiceRecorded}
                audioParameters={$audioParameters}
            />
            <DroppableNode
                {nodeWidthPx}
                {nodeWidthLogical}
                nodeId={$selectedAngle}
                class="z-10"
                ondropnode={handleDropNode}
            />
        </div>
    {/if}
    <AngleFileBox
        onPlaybackPosition={(position) => {
            playheadPosition.target = position;
        }}
        onFileAngle={(newAngle) => ($angle = newAngle)}
        onFileLoaded={(loaded) => ($fileLoaded = loaded)}
    />
</div>
