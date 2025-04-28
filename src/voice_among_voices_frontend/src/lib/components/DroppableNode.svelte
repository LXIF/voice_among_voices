<script lang="ts">
    import {
        applicationState,
        applicationStates,
        voiceNodes,
    } from "$lib/state/uxState";
    import { scale } from "svelte/transition";
    import { elasticOut } from "svelte/easing";
    import { getVoiceNodes } from "$lib/icInteractions";

    let {
        ondropnode,
        nodeWidthPx,
        nodeWidthLogical,
        nodeId,
        class: classes,
        showHandle = true,
    }: {
        ondropnode: ({ x, y }: { x: number; y: number }) => void;
        nodeWidthPx: number;
        nodeWidthLogical: number;
        nodeId: number;
        class?: string;
        showHandle?: boolean;
    } = $props();

    let draggableElement = $state<HTMLDivElement>();

    let scaledNodeWidthPx = $derived(
        (document.querySelector("canvas")?.width! / 600) * nodeWidthPx,
    );
    const handleWidth = 50;
    const pointOffset = 7;
    let nodeX = $derived(handleWidth / 2);
    let nodeY = $derived(scaledNodeWidthPx / 2);
    let handleY = $derived(nodeY + 70);

    async function handlePointerDown(e: PointerEvent) {
        e.preventDefault();
        if (!draggableElement) throw "should be unreachable";
        if (!$applicationState.droppingActive) return;
        draggableElement.style.left = `${e.clientX - (nodeWidthPx / 2 + 25)}px`;
        draggableElement.style.top = `${e.clientY - (nodeWidthPx / 2 + 70)}px`;
        startDrag();
        $voiceNodes = await getVoiceNodes();
    }

    function pointerMove(e: PointerEvent) {
        e.preventDefault();
        if (!draggableElement) throw "should be unreachable";
        draggableElement.style.left = `${e.clientX - (nodeWidthPx / 2 + 25)}px`;
        draggableElement.style.top = `${e.clientY - (nodeWidthPx / 2 + 70)}px`;
    }

    function startDrag() {
        if (!draggableElement) throw "should be unreachable";
        $applicationState = applicationStates.draggingVoice;
        draggableElement.classList.add("fixed");
        const body = document.querySelector("body");
        if (!body) throw "Help! I have no body!";
        body.onpointermove = pointerMove;
    }

    function handlePointerUp(e: PointerEvent) {
        ondropnode({ x: e.clientX, y: e.clientY });
        setTimeout(() => {
            if (!draggableElement) throw "should be unreachable";
            draggableElement.classList.remove("fixed");
            draggableElement.style.left = "";
            draggableElement.style.top = "";
            const body = document.querySelector("body");
            if (!body) throw "Help! I have no body!";
            body.onpointermove = null;
        }, 100);
    }
</script>

<!-- <div class={`flex justify-center items-center h-10 w-full ${classes}`}> -->
<!-- <div
    draggable="true"
    role="button"
    tabindex="0"
    aria-roledescription="drag this onto the map to place your node"
    ondragstart={handleDragStart}
    ondragend={handleDragEnd}
    class="flex justify-center items-center rounded-full cursor-pointer z-10"
    class:opacity-0={dragging}
    style={`height: ${nodeWidthPx}px; width: ${nodeWidthPx}px; background-color: hsl(${nodeId % 360}, 100%, 50%);`}
>
    <div class="relative top-14 bg-black dark:bg-white text-white dark:text-black min-w-10 min-h-10 rounded-full text-center flex justify-center items-center text-3xl">drag</div>
</div> -->
<!-- </div> -->
<div class="flex w-20 items-center justify-center">
    {#if nodeWidthPx > 0 && $applicationState.showDraggableNode}
        <div
            role="button"
            tabindex="0"
            aria-roledescription="drag this onto the map to place your node"
            onpointerdown={handlePointerDown}
            onpointerup={handlePointerUp}
            class="pointer-events-auto z-50 cursor-pointer touch-none select-none"
            bind:this={draggableElement}
            transition:scale={{
                duration: 500,
                easing: elasticOut,
            }}
        >
            <svg
                width={handleWidth}
                height={scaledNodeWidthPx + 2 * handleWidth}
            >
                <!-- Drag handle -->
                <circle
                    cx={handleWidth / 2}
                    cy={handleY}
                    r={25}
                    class="fill-slate-950 dark:fill-slate-50"
                />

                <!-- Drag handle line -->
                <line
                    x1={nodeX}
                    y1={handleY}
                    x2={nodeX}
                    y2={nodeY}
                    stroke-width="2"
                    class="stroke-slate-950 dark:stroke-slate-50"
                />

                <!-- Drag handle dots -->
                <circle
                    cx={nodeX - pointOffset}
                    cy={handleY - pointOffset / 2}
                    r={2}
                    class="fill-slate-200 dark:fill-slate-500"
                />
                <circle
                    cx={nodeX}
                    cy={handleY - pointOffset / 2}
                    r={2}
                    class="fill-slate-200 dark:fill-slate-500"
                />
                <circle
                    cx={nodeX + pointOffset}
                    cy={handleY - pointOffset / 2}
                    r={2}
                    class="fill-slate-200 dark:fill-slate-500"
                />
                <circle
                    cx={nodeX - pointOffset}
                    cy={handleY + pointOffset / 2}
                    r={2}
                    class="fill-slate-200 dark:fill-slate-500"
                />
                <circle
                    cx={nodeX}
                    cy={handleY + pointOffset / 2}
                    r={2}
                    class="fill-slate-200 dark:fill-slate-500"
                />
                <circle
                    cx={nodeX + pointOffset}
                    cy={handleY + pointOffset / 2}
                    r={2}
                    class="fill-slate-200 dark:fill-slate-500"
                />

                <!-- Recorded Node -->
                <circle
                    cx={nodeX}
                    cy={nodeY}
                    r={scaledNodeWidthPx / 2}
                    fill={`hsl(${nodeId % 360}, 100%, 50%)`}
                />
            </svg>
        </div>
    {/if}
</div>
