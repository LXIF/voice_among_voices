<script lang="ts">
    import {onMount} from 'svelte';

    export let nodes: VoiceNode[] = [];

    let canvas: HTMLCanvasElement;
    let context: CanvasRenderingContext2D | null;
    let physicsActive = false;
    let rendering = false;
    let resetting = false;

    type PhysicsBody = {
        collider: any;
        rigidBody: any;
        voiceNode: VoiceNode;
    };

    onMount(() => {
        if (canvas) {
            context = canvas.getContext('2d');
        }
    });

    $: if (context && nodes.length > 1 && !rendering && !resetting) {
        rendering = true;
        setupAndRender();
    }

    ///////////PHYSICS//////////

    const setupAndRender = () => {
        import('@dimforge/rapier2d').then((RAPIER) => {
            let world: any;
            let physicsBodies: PhysicsBody[] = [];
            let gravity = {x: 0, y: 0};

            world = new RAPIER.World(gravity);

            // Magnetism parameters in logical coords
            const maxDistance = 20;
            const forceStrength = 1000; // TODO: get this from sample length too
            const linearDamping = 2;

            function applyMagnetismForces() {
                physicsBodies.forEach((body, i) => {
                    // Create array of all bodies within reach
                    const cutoffPosition = body.rigidBody.translation();
                    const cutoffRotation = body.rigidBody.rotation();
                    const cutoffShape = new RAPIER.Ball(maxDistance);

                    let bodiesWithinReach: any[] = [];
                    let magneticForces: any[] = [];

                    world.intersectionsWithShape(
                        cutoffPosition,
                        cutoffRotation,
                        cutoffShape,
                        (collider: any) => {
                            const localBody = collider.parent();
                            if (!localBody) return true;
                            const localBodyUserData =
                                localBody?.userData as VoiceNode;
                            if (localBodyUserData.id === body.voiceNode.id)
                                return true;
                            bodiesWithinReach.push(localBody);
                            return true;
                        }
                    );

                    // Create resultant force out of all their vectors
                    bodiesWithinReach.forEach((bodyWithinReach) => {
                        // magnetism is 1/r^2
                        // find vector between body and bodyWithinReach
                        // scale it by 1/r^2
                        const bodyPos = body.rigidBody.translation();
                        const bodyWithinReachPos =
                            bodyWithinReach.translation();

                        const distanceBetweenBodies = Math.sqrt(
                            (bodyWithinReachPos.x - bodyPos.x) ** 2 +
                                (bodyWithinReachPos.y - bodyPos.y) ** 2
                        );
                        const magneticForceScalar =
                            (1 / distanceBetweenBodies ** 2) * forceStrength;
                        const vectorBetweenBodies = new RAPIER.Vector2(
                            bodyPos.x - bodyWithinReachPos.x,
                            bodyPos.y - bodyWithinReachPos.y
                        );
                        // add vector to the magnetic forces
                        magneticForces.push(
                            new RAPIER.Vector2(
                                vectorBetweenBodies.x * magneticForceScalar,
                                vectorBetweenBodies.y * magneticForceScalar
                            )
                        );
                    });

                    // Apply the force
                    body.rigidBody.resetForces(true);
                    magneticForces.forEach((force) =>
                        body.rigidBody.addForce(force, true)
                    );
                });
            }

            // create world colliders

            const topCollider = RAPIER.ColliderDesc.cuboid(
                logicalWidth / 2,
                0.1
            ).setTranslation(logicalWidth / 2, 0);
            const leftCollider = RAPIER.ColliderDesc.cuboid(
                0.1,
                logicalHeight / 2
            ).setTranslation(0, logicalHeight / 2);
            const bottomCollider = RAPIER.ColliderDesc.cuboid(
                logicalWidth / 2,
                0.1
            ).setTranslation(logicalWidth / 2, logicalHeight);
            const rightCollider = RAPIER.ColliderDesc.cuboid(
                0.1,
                logicalHeight / 2
            ).setTranslation(logicalWidth, logicalHeight / 2);

            world.createCollider(topCollider);
            world.createCollider(leftCollider);
            world.createCollider(bottomCollider);
            world.createCollider(rightCollider);

            // create rigid bodies
            physicsBodies = nodes.map((node) => {
                let rigidBodyDesc = RAPIER.RigidBodyDesc.dynamic()
                    .lockRotations()
                    .setLinearDamping(linearDamping)
                    .setTranslation(Number(node.x), Number(node.y))
                    .setUserData(node);
                let colliderDesc = RAPIER.ColliderDesc.ball(2).setDensity(2.0); // TODO: get this from node size / sample length.

                const rigidBody = world.createRigidBody(rigidBodyDesc);
                const collider = world.createCollider(colliderDesc, rigidBody);

                return {
                    rigidBody,
                    collider,
                    voiceNode: node,
                };
            });

            render(physicsBodies, world, applyMagnetismForces);
        });
    };

    ////////////////////////////

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
    function render(
        bodies: PhysicsBody[],
        world: any,
        magnetismFunction: Function
    ) {
        // const nodesToDraw = bodies.map((body) => {
        //     return body.voiceNode;
        // });
        if (world && physicsActive) {
            magnetismFunction();
            world.step();
        }

        if (context && bodies && bodies.length > 1) {
            // Clear the canvas
            context.clearRect(0, 0, canvasWidth, canvasHeight);

            // Draw each VoiceNode as a circle using the mapped coordinates
            bodies.forEach((body) => {
                const bodyPos = body.rigidBody.translation();
                const canvasX = mapToCanvasX(Number(bodyPos.x));
                const canvasY = mapToCanvasY(Number(bodyPos.y));

                context!.beginPath();
                context!.arc(canvasX, canvasY, 10, 0, Math.PI * 2);
                context!.fillStyle = 'blue';
                context!.fill();
                context!.stroke();
                context!.closePath();
            });
        }

        setTimeout(() => {
            if (rendering) {
                render(bodies, world, magnetismFunction);
            }
        }, 16);
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

    function togglePhysics() {
        physicsActive = !physicsActive;
        console.log(physicsActive);
    }

    function resetPhysics() {
        resetting = true;
        rendering = false;

        setTimeout(() => {
            //let last frame play out
            rendering = true;
            resetting = false;
            setupAndRender();
        }, 20);
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
    <button on:click={togglePhysics}>toggle physics</button>
    <button on:click={resetPhysics}>reset physics</button>
</main>

<style>
    main {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 100vh;
    }
</style>
