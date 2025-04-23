<script lang="ts">
    import { dragging } from "$lib/state/uxState";
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

    const handleWidth = 50;
    const pointOffset = 7;
    let nodeX = $derived(nodeWidthPx / 2 + handleWidth / 2);
    let nodeY = $derived(nodeWidthPx / 2);
    let handleY = $derived(nodeY + 70);

    function handlePointerDownt(e: PointerEvent) {
        e.preventDefault();
        if (!draggableElement) throw "should be unreachable";
        draggableElement.style.left = `${e.clientX - (nodeWidthPx / 2 + 25)}px`;
        draggableElement.style.top = `${e.clientY - (nodeWidthPx / 2 + 70)}px`;
        startDrag();
    }

    function pointerMove(e: PointerEvent) {
        e.preventDefault();
        if (!draggableElement) throw "should be unreachable";
        draggableElement.style.left = `${e.clientX - (nodeWidthPx / 2 + 25)}px`;
        draggableElement.style.top = `${e.clientY - (nodeWidthPx / 2 + 70)}px`;
    }

    function startDrag() {
        if (!draggableElement) throw "should be unreachable";
        $dragging = true;
        draggableElement.classList.add("fixed");
        draggableElement.onpointermove = pointerMove;
    }

    function handlePointerUp(e: PointerEvent) {
        if (!draggableElement) throw "should be unreachable";
        ondropnode({ x: e.clientX, y: e.clientY });
        $dragging = false;
        draggableElement.classList.remove("fixed");
        draggableElement.style.left = "";
        draggableElement.style.top = "";
        draggableElement.onpointermove = null;
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
<!-- TODO: generalize -->
{#if nodeWidthPx > 0}
    <div
        role="button"
        tabindex="0"
        aria-roledescription="drag this onto the map to place your node"
        onpointerdown={handlePointerDownt}
        onpointerup={handlePointerUp}
        class="z-10 cursor-pointer touch-none select-none"
        bind:this={draggableElement}
    >
        <svg
            width={nodeWidthPx + handleWidth}
            height={nodeWidthPx + 2 * handleWidth}
        >
            <!-- Drag handle -->
            <circle
                cx={nodeWidthPx / 2 + 25}
                cy={nodeWidthPx / 2 + 70}
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
                r={nodeWidthPx / 2}
                fill={`hsl(${nodeId % 360}, 100%, 50%)`}
            />
        </svg>
    </div>
{/if}
