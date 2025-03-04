<script lang="ts">
    import {createEventDispatcher} from 'svelte';
    import {angleToRadians} from '$lib/utils/convUtils';
    import type {VoiceNodeEgress} from '../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did';
    import {
        canvasWidth,
        canvasHeight,
        usableCanvasWidth,
        usableCanvasHeight,
        worldStepInterval,
    } from '$lib/config/nodeMap';
    import { onMount } from 'svelte';

    let { 
        availableAngles, 
        nodes,
        class: classes = '',
    }: { 
        availableAngles: number[], 
        nodes: VoiceNodeEgress[],
        class?: string 
    } = $props();

    let hoveredAngle: number | null = $state(null);

    const handleSelectAngle = (angle: number) => {
        console.log(angle);
    };

    // Adjusted scaling parameters for SVG
    const centerX = 100;
    const centerY = 100;
    const radius = 84; // Radius of the outer circle
    const lineStopFactor = 0.875; // Factor to stop lines closer to the center (7/8th of the way)

    // Function to adjust angles: 0 is north, and positive angles go clockwise
    function adjustedAngleToRadians(angle: number) {
        return angleToRadians(90 - angle); // Subtracting angle from 90 to shift 0 degrees to "north"
    }

    function hsvToRgb(h: number, s: number, v: number): string {
        s = s / 100;
        v = v / 100;
        const i = Math.floor(h / 60);
        const f = h / 60 - i;
        const p = v * (1 - s);
        const q = v * (1 - f * s);
        const t = v * (1 - (1 - f) * s);
        
        let r, g, b;
        switch (i % 6) {
            case 0: [r, g, b] = [v, t, p]; break;
            case 1: [r, g, b] = [q, v, p]; break;
            case 2: [r, g, b] = [p, v, t]; break;
            case 3: [r, g, b] = [p, q, v]; break;
            case 4: [r, g, b] = [t, p, v]; break;
            case 5: [r, g, b] = [v, p, q]; break;
            default: [r, g, b] = [0, 0, 0];
        }
        
        return `rgb(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`;
    }

    function isAngleAvailable(angle: number): boolean {
        return availableAngles.includes(angle);
    }
</script>

<svg
    width={usableCanvasWidth}
    height={usableCanvasHeight}
    viewBox="0 0 200 200"
    xmlns="http://www.w3.org/2000/svg"
    class={`${classes} pointer-events-none`}
>
    <!-- Draw circle -->
    <!-- <circle
        cx={centerX}
        cy={centerY}
        r={radius}
        stroke="black"
        fill="none"
    /> -->

    <!-- Draw lines for all angles -->
    {#each Array.from({length: 360}, (_, i) => i + 1) as angle}
        <line
            tabindex={isAngleAvailable(angle) ? 0 : -1}
            role="button"
            x1={centerX + Math.cos(adjustedAngleToRadians(angle)) * radius * (hoveredAngle === angle ? 0.875 : 1)}
            y1={centerY - Math.sin(adjustedAngleToRadians(angle)) * radius * (hoveredAngle === angle ? 0.875 : 1)}
            x2={centerX + Math.cos(adjustedAngleToRadians(angle)) * radius * (hoveredAngle === angle ? 1.25 : 1.1)}
            y2={centerY - Math.sin(adjustedAngleToRadians(angle)) * radius * (hoveredAngle === angle ? 1.25 : 1.1)}
            class="transition-all duration-200 ease-in-out pointer-events-auto outline-none"
            onmouseover={() => isAngleAvailable(angle) && (hoveredAngle = angle)}
            onmouseleave={() => (hoveredAngle = null)}
            onclick={() => isAngleAvailable(angle) && handleSelectAngle(angle)}
            onfocus={() => isAngleAvailable(angle) && (hoveredAngle = angle)}
            onblur={() => (hoveredAngle = null)}
            onkeydown={(e) => {
                if (isAngleAvailable(angle) && (e.key === 'Enter' || e.key === ' ')) {
                    handleSelectAngle(angle);
                }
            }}
            stroke={isAngleAvailable(angle) ? hsvToRgb(angle, 100, 100) : hsvToRgb(angle, 50, 100)}
            stroke-width="0.5"
            pointer-events={isAngleAvailable(angle) ? 'auto' : 'none'}
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

<!-- {#if hoveredAngle}
    <h1>{hoveredAngle}</h1>
{/if} -->

<style>
    .highlight {
        stroke: red; /* Highlight color */
        stroke-width: 2;
    }
</style>
