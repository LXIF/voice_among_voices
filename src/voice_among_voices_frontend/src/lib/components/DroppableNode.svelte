<script lang="ts">
    import {createEventDispatcher} from 'svelte';

    export let nodeWidthPx = 0;
    export let nodeWidthLogical = 0;

    const dispatch = createEventDispatcher();
    let dragging = false;

    function handleDragStart(e: DragEvent) {
        dispatch('dragstart');
        // clone and move offscreen
        let dragImageElement = (e.target as HTMLDivElement).cloneNode(
            true
        ) as HTMLDivElement;
        dragImageElement.style.position = 'absolute';
        dragImageElement.style.top = '-1000px';
        dragImageElement.style.left = '-1000px';
        document.body.appendChild(dragImageElement);

        const rect = dragImageElement.getBoundingClientRect();

        dragging = true;
        e.dataTransfer?.setDragImage(
            dragImageElement,
            rect.width / 2,
            rect.height / 2
        );
        e.dataTransfer?.setData('nodeRadius', String(nodeWidthLogical / 2));
    }

    function handleDragEnd() {
        dragging = false;
        dispatch('dragend');
    }
</script>

<div class="flex justify-center items-center h-20 w-full">
    <div
        draggable="true"
        role="button"
        tabindex="0"
        aria-roledescription="drag this onto the map to place your node"
        on:dragstart={handleDragStart}
        on:dragend={handleDragEnd}
        class="flex justify-center items-center rounded-full cursor-pointer bg-red-600"
        class:opacity-0={dragging}
        style={`height: ${nodeWidthPx}px; width: ${nodeWidthPx}px;`}
    ></div>
</div>
