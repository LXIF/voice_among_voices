<script lang="ts">
    import { audioParameters, simulationParameters, currentVoiceBlob, dragging, playheadPosition, externalPlaybackPosition, angle, fileLoaded, hoveredAngle, selectedAngle } from "$lib/state/uxState.svelte";
    import DroppableNode from "./DroppableNode.svelte";
    import VoiceRecorder from "./VoiceRecorder.svelte";
    import AngleFileBox from "./AngleFileBox.svelte";
    import { usableCanvasWidth } from '$lib/config/nodeMap';
    import { onMount } from "svelte";
    import { backend } from "$lib/canisters";
    import { identityAgent } from "$lib/canisters";

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
        logicalWidth: number
    ) => {
        const pixelPerMs = canvasWidth / totalLength;
        const logicalPerMs = logicalWidth / totalLength;

        return {
            nodeWidthPx: sampleLength * pixelPerMs,
            nodeWidthLogical: sampleLength * logicalPerMs,
        };
    };

    const handleRecordingLength = (length: number) => {
        if($audioParameters === undefined || $simulationParameters === undefined) throw "invalid params";

            sampleLength = length;

            if(sampleLength > 0) {
                const nodeWidths = calculateNodeWidth(
                    length,
                    usableCanvasWidth,
                    $audioParameters!.total_length_ms,
                    $simulationParameters!.logical_radius * 2
                );
        
                nodeWidthPx = nodeWidths.nodeWidthPx;
                nodeWidthLogical = nodeWidths.nodeWidthLogical;
            }
    };

    const handleVoiceRecorded = (blob: Blob) => {
        $currentVoiceBlob = blob;
    };
</script>


<div>
    {#if $selectedAngle && $identityAgent}
        <DroppableNode
            {nodeWidthPx}
            {nodeWidthLogical}
            ondragstart={() => ($dragging = true)}
            ondragend={() => ($dragging = false)}
            nodeId={$selectedAngle}
            class="z-10"
        />
        <VoiceRecorder
            recordingLength={handleRecordingLength}
            voiceRecorded={handleVoiceRecorded}
            audioParameters={$audioParameters}
        />
        <AngleFileBox
            externalPlaybackPosition={$externalPlaybackPosition}
            onPlaybackPosition={(position) => ($playheadPosition = position)}
            onFileAngle={(newAngle) => ($angle = newAngle)}
            onFileLoaded={(loaded) => ($fileLoaded = loaded)}
            angle={$selectedAngle!}
        />
    {:else}
    <AngleFileBox
        externalPlaybackPosition={$externalPlaybackPosition}
        onPlaybackPosition={(position) => ($playheadPosition = position)}
        onFileAngle={(newAngle) => ($angle = newAngle)}
        onFileLoaded={(loaded) => ($fileLoaded = loaded)}
        angle={0}
    />
    {/if}
    {#if $hoveredAngle || $selectedAngle}
        <p>{$hoveredAngle ? $hoveredAngle : $selectedAngle}</p>
    {/if}
</div>