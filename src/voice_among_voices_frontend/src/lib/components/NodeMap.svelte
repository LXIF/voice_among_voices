<script lang="ts">
    import {onMount} from 'svelte';

    export let nodes: VoiceNode[] = [];

    let canvas: HTMLCanvasElement;
    let context: CanvasRenderingContext2D | null;

    /////////RENDERING///////////
    // Canvas dimensions
    const canvasWidth = 500;
    const canvasHeight = 500;

    // Logical coordinates for internal mapping
    const logicalWidth = 100;
    const logicalHeight = 100;

    function mapToCanvasX(logicalX: number) {
        return (logicalX / logicalWidth) * canvasWidth;
    }

    function mapToCanvasY(logicalY: number) {
        return (logicalY / logicalHeight) * canvasHeight;
    }

    // Draw the nodes on the canvas
    function drawNodes() {
        if (context) {
            // Clear the canvas
            context.clearRect(0, 0, canvasWidth, canvasHeight);

            // Draw each VoiceNode as a circle using the mapped coordinates
            nodes.forEach((node) => {
                const canvasX = mapToCanvasX(Number(node.x));
                const canvasY = mapToCanvasY(Number(node.y));

                context!.beginPath();
                context!.arc(canvasX, canvasY, 10, 0, Math.PI * 2);
                context!.fillStyle = 'blue';
                context!.fill();
                context!.stroke();
                context!.closePath();
            });
        }
    }

    // Handle the click event and map it back to logical coordinates
    function handleClick(event: MouseEvent) {
        const rect = canvas.getBoundingClientRect();
        const canvasX = event.clientX - rect.left;
        const canvasY = event.clientY - rect.top;

        // Convert canvas coordinates to logical coordinates
        const logicalX = (canvasX / canvasWidth) * logicalWidth;
        const logicalY = (canvasY / canvasHeight) * logicalHeight;

        console.log('Clicked coordinates (logical):', {logicalX, logicalY});
    }

    onMount(() => {
        if (canvas) {
            context = canvas.getContext('2d');
            drawNodes();
        }
    });

    $: if (context && nodes) {
        drawNodes();
    }
</script>

<main>
    <canvas
        bind:this={canvas}
        width={canvasWidth}
        height={canvasHeight}
        on:click={handleClick}
        style="border: 1px solid black;"
    ></canvas>
</main>

<style>
    main {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 100vh;
    }
</style>
