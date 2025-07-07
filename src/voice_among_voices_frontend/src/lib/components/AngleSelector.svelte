<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { angleToRadians } from "$lib/utils/convUtils";
    import type { VoiceNodeEgress } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import {
        canvasWidth,
        canvasHeight,
        usableCanvasWidth,
        usableCanvasHeight,
        worldStepInterval,
    } from "$lib/config/nodeMap";

    let {
        availableAngles,
        nodes,
    }: { availableAngles: number[]; nodes: VoiceNodeEgress[] } = $props();

    let hoveredAngle: number | null = $state(null);

    const handleSelectAngle = (angle: number) => {};

    // Adjusted scaling parameters for SVG
    const centerX = 100;
    const centerY = 100;
    const radius = 90; // Radius of the outer circle
    const lineStopFactor = 0.875; // Factor to stop lines closer to the center (7/8th of the way)

    // Function to adjust angles: 0 is north, and positive angles go clockwise
    function adjustedAngleToRadians(angle: number) {
        return angleToRadians(90 - angle); // Subtracting angle from 90 to shift 0 degrees to "north"
    }
</script>

<svg
    width={usableCanvasWidth}
    height={usableCanvasHeight}
    viewBox="0 0 200 200"
    xmlns="http://www.w3.org/2000/svg"
>
    <!-- Draw circle -->
    <circle cx={centerX} cy={centerY} r={radius} stroke="black" fill="none" />

    <!-- Draw lines for available angles -->
    {#each availableAngles as angle}
        <line
            tabindex="0"
            role="button"
            x1={centerX +
                Math.cos(adjustedAngleToRadians(angle)) *
                    radius *
                    lineStopFactor}
            y1={centerY -
                Math.sin(adjustedAngleToRadians(angle)) *
                    radius *
                    lineStopFactor}
            x2={centerX + Math.cos(adjustedAngleToRadians(angle)) * radius}
            y2={centerY - Math.sin(adjustedAngleToRadians(angle)) * radius}
            class={hoveredAngle === angle ? "highlight" : ""}
            onmouseover={() => (hoveredAngle = angle)}
            onmouseleave={() => (hoveredAngle = null)}
            onclick={() => handleSelectAngle(angle)}
            onfocus={() => (hoveredAngle = angle)}
            onblur={() => (hoveredAngle = null)}
            onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                    handleSelectAngle(angle);
                }
            }}
            stroke="black"
            stroke-width="1"
        />
    {/each}

    <!-- Draw voice nodes -->
    {#if nodes && nodes.length > 0}
        {#each nodes as node}
            <circle
                cx={centerX + node.x / 2}
                cy={centerY - node.y / 2}
                r={node.radius / 2}
                fill="blue"
            />
        {/each}
    {/if}
</svg>

{#if hoveredAngle}
    <h1>{hoveredAngle}</h1>
{/if}

<style>
    .highlight {
        stroke: red; /* Highlight color */
        stroke-width: 2;
    }
</style>
