<script lang="ts">
    import {onMount} from 'svelte';
    import {mapRange, clamp} from '$lib/utils/mathUtils';
    import DroppableNode from './DroppableNode.svelte';
    import {createEventDispatcher} from 'svelte';

    export let nodes: VoiceNodeEgress[] = [];

    let canvas: HTMLCanvasElement;
    let context: CanvasRenderingContext2D | null;
    let physicsActive = false;
    let rendering = false;
    let resetting = false;
    let moving = false;
    const velocityCutoff = 0.2; // TODO: get from canister
    const forceCutoff = 100; // TODO: get from can
    const dispatch = createEventDispatcher();

    type PhysicsBody = {
        collider: any;
        rigidBody: any;
        voiceNode: VoiceNodeEgress;
    };

    onMount(() => {
        if (canvas) {
            context = canvas.getContext('2d');
        }
    });

    $: if (context && nodes.length >= 1 && !rendering && !resetting) {
        rendering = true;
        setupAndRender();
    }

    $: if (!!nodes) {
        console.log('bargle');
        resetPhysics();
    }

    ///////////PHYSICS//////////

    const setupAndRender = () => {
        import('@dimforge/rapier2d').then((RAPIER) => {
            let world: any;
            let physicsBodies: PhysicsBody[] = [];
            let gravity = {x: 0, y: 0};

            world = new RAPIER.World(gravity);

            // Magnetism parameters in logical coords
            const maxDistance = 20; // TODO: get these params from canister
            const forceStrength = 3000; // TODO: get this from sample length too
            const linearDamping = 10;

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
                                localBody?.userData as VoiceNodeEgress;
                            // if (localBodyUserData.id === body.voiceNode.id)
                            //     return true;
                            bodiesWithinReach.push(localBody);
                            return true;
                        },
                        null,
                        null,
                        null,
                        body.rigidBody
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

                    // reset the forces
                    body.rigidBody.resetForces(true);
                    // check if resultant is above threshold
                    const resultant = magneticForces.reduce((acc, current) => {
                        return acc + Math.sqrt(current.x ** 2 + current.y ** 2);
                    }, 0);

                    if (resultant < forceCutoff) return;

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

    // Converts canvas pixel coordinates to logical coordinates
    function canvasToLogical(x: number, y: number) {
        const logicalX = (x / canvasWidth) * logicalWidth;
        const logicalY = (y / canvasHeight) * logicalHeight;
        return {logicalX, logicalY};
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
            checkIfStillMoving(bodies);
        }

        if (context && bodies && bodies.length > 1) {
            // Clear the canvas
            context.clearRect(0, 0, canvasWidth, canvasHeight);

            // Draw each VoiceNode as a circle using the mapped coordinates
            bodies.forEach((body) => {
                const bodyPos = body.rigidBody.translation();
                const linVel = body.rigidBody.linvel();
                const absVel = Math.sqrt(linVel.x ** 2 + linVel.y ** 2);
                const colorVel = clamp(mapRange(absVel, 0, 2, 0, 255), 0, 255);
                const canvasX = mapToCanvasX(Number(bodyPos.x));
                const canvasY = mapToCanvasY(Number(bodyPos.y));

                context!.beginPath();
                context!.ellipse(canvasX, canvasY, 10, 10, 0, 0, Math.PI * 2);
                context!.fillStyle = `rgb(${colorVel},${colorVel},0)`;
                context!.fill();
                context!.stroke();
                context!.closePath();
            });
        }

        function checkIfStillMoving(bodies: PhysicsBody[]) {
            moving = false;
            bodies.forEach((body) => {
                if (body.rigidBody.isMoving()) {
                    moving = true;
                }
            });
        }

        function stopSlowNodes(bodies: PhysicsBody[]) {
            bodies.forEach((body) => {
                const linVel = body.rigidBody.linvel();
                const absVel = Math.sqrt(linVel.x ** 2 + linVel.y ** 2);

                if (absVel < velocityCutoff) {
                    body.rigidBody.setLinvel({x: 0, y: 0}, true);
                }
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

    /////for drag and drop/////

    function handleDragOver(e: DragEvent) {
        e.preventDefault();
    }

    async function handleDrop(e: DragEvent) {
        e.preventDefault();
        const rect = canvas.getBoundingClientRect();
        const canvasX = e.clientX - rect.left;
        const canvasY = e.clientY - rect.top;

        const {logicalX, logicalY} = canvasToLogical(canvasX, canvasY);

        console.log('dropped at logical coordinates:', {logicalX, logicalY});

        const voiceNode = {
            x: logicalX,
            y: logicalY,
            sample: 'test',
        };

        dispatch('dropNewNode', voiceNode as VoiceNodeIngress);
        rendering = false;
    }
</script>

<main>
    <canvas
        bind:this={canvas}
        width={canvasWidth}
        height={canvasHeight}
        on:click={handleClick}
        on:dragover={handleDragOver}
        on:drop={handleDrop}
        style="border: 1px solid black;"
    ></canvas>
    <button
        class="bg-slate-500 rounded-full"
        on:click={togglePhysics}>toggle physics</button
    >
    <button
        class="bg-slate-500 rounded-full"
        on:click={resetPhysics}>reset physics</button
    >
    <div>
        moving: {moving}
    </div>
    <DroppableNode />
</main>
