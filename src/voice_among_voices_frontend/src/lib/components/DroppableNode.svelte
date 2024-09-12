<script lang="ts">
    export let nodeWidth = 5; // TODO: get from sample length, adjust for logical/canvas

    let dragging = false;

    function handleDragStart(e: DragEvent) {
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
        e.dataTransfer?.setData('nodeRadius', String(nodeWidth / 2));
    }

    function handleDragEnd() {
        dragging = false;
    }
</script>

<div
    draggable="true"
    aria-roledescription="drag this onto the map to place your node"
    on:dragstart={handleDragStart}
    on:dragend={handleDragEnd}
    class="flex justify-center items-center rounded-full cursor-pointer bg-red-600 w-5 h-5"
    class:opacity-0={dragging}
>
    drag
</div>
