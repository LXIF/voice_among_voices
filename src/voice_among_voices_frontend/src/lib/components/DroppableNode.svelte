<script lang="ts">
    let {
        ondragstart,
        ondragend,
        nodeWidthPx,
        nodeWidthLogical,
        nodeId,
        class: classes,
        showHandle = true,
    }: {
        ondragstart: () => void;
        ondragend: () => void;
        nodeWidthPx: number;
        nodeWidthLogical: number;
        nodeId: number;
        class?: string;
        showHandle?: boolean;
    } = $props();

    let dragging = $state(false);

    function handleDragStart(e: DragEvent) {
        ondragstart();
        // clone and move offscreen
        let dragImageElement = (e.target as HTMLDivElement).cloneNode(
            true,
        ) as HTMLDivElement;
        dragImageElement.style.position = "absolute";
        dragImageElement.style.top = "-10000px";
        dragImageElement.style.left = "-10000px";
        document.body.appendChild(dragImageElement);

        const rect = dragImageElement.getBoundingClientRect();

        dragging = true;
        e.dataTransfer?.setDragImage(
            dragImageElement,
            rect.width / 2,
            rect.height / 2,
        );
        e.dataTransfer?.setData("nodeRadius", String(nodeWidthLogical / 2));
    }

    function handleDragEnd() {
        dragging = false;
        ondragend();
    }
</script>

<!-- <div class={`flex justify-center items-center h-10 w-full ${classes}`}> -->
<div
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
    <!-- <div class="relative top-14 bg-black dark:bg-white text-white dark:text-black min-w-10 min-h-10 rounded-full text-center flex justify-center items-center text-3xl">drag</div> -->
</div>
<!-- </div> -->
