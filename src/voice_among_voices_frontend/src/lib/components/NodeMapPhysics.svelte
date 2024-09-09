<script lang="ts">
    import RAPIER from '@dimforge/rapier2d';
    import {onMount} from 'svelte';

    export let nodes: VoiceNode[] = [];

    let canvas: HTMLCanvasElement;
    let context: CanvasRenderingContext2D | null;
    let world: RAPIER.World;
    let physicsBodies: PhysicsBody[] = [];

    type PhysicsBody = {
        collider: RAPIER.Collider;
        rigidBody: RAPIER.RigidBody;
    };

    /////////RENDERING///////////
    // Canvas dimensions
    const canvasWidth = 500;
    const canvasHeight = 500;

    // Logical coordinates for internal mapping
    const logicalWidth = 10;
    const logicalHeight = 10;

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

    ///////////PHYSICS//////////
    // Magnetism parameters in logical coords
    const maxDistance = 2;
    const forceStrength = 5; // TODO: get this from sample length too
    const linearDamping = 0.5;

    function applyMagnetismForces() {
        physicsBodies.forEach((body, i) => {
            // Create array of all bodies within reach
            const cutoffPosition = body.rigidBody.translation();
            const cutoffRotation = body.rigidBody.rotation();
            const cutoffShape = new RAPIER.Ball(maxDistance);

            let resultantForce = new RAPIER.Vector2(0, 0);
            let bodiesWithinReach: RAPIER.RigidBody[] = [];

            world.intersectionsWithShape(
                cutoffPosition,
                cutoffRotation,
                cutoffShape,
                (collider) => {
                    const body = collider.parent();
                    if (!body) return true;
                    bodiesWithinReach.push(body);
                    return true;
                }
            );

            // Create resultant force out of all their vectors
            bodiesWithinReach.forEach((bodyWithinReach) => {
                // magnetism is 1/r^2
                // const distance =
            });

            // Apply the force
        });
    }

    import('@dimforge/rapier2d').then((RAPIER) => {
        let gravity = {x: 0, y: 0};
        world = new RAPIER.World(gravity);

        // create rigid bodies
        physicsBodies = nodes.map((node) => {
            let rigidBodyDesc = RAPIER.RigidBodyDesc.dynamic()
                .lockRotations()
                .setLinearDamping(linearDamping)
                .setTranslation(Number(node.x), Number(node.y));
            let colliderDesc = RAPIER.ColliderDesc.ball(0.5).setDensity(2.0); // TODO: get this from node size / sample length.

            const rigidBody = world.createRigidBody(rigidBodyDesc);
            const collider = world.createCollider(colliderDesc, rigidBody);

            return {
                rigidBody,
                collider,
            };
        });
    });

    ////////////////////////////

    onMount(() => {
        if (canvas) {
            context = canvas.getContext('2d');
            drawNodes();
        }
    });

    $: if (context) {
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
