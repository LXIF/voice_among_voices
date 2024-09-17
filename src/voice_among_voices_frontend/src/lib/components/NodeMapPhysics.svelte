<script lang="ts">
    import {onMount} from 'svelte';
    import {mapRange, clamp} from '$lib/utils/mathUtils';
    import DroppableNode from './DroppableNode.svelte';
    import {createEventDispatcher} from 'svelte';
    import RAPIER from '@dimforge/rapier2d-compat';
    import {browser} from '$app/environment'; // ts keeps motzing but it works
    import {backend} from '$lib/canisters'; // motzes but works
    import type {
        ColliderCoordinate,
        VoiceNodeEgress,
        SimulationParameters,
        VoiceNodeIngress,
    } from '../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did';
    import {
        convertColliderCoordinatesToFloat32Array,
        mapToCanvasX,
        mapToCanvasY,
        canvasToLogical,
    } from '$lib/utils/convUtils';

    export let nodes: VoiceNodeEgress[] = [];
    export let backendNodes: VoiceNodeEgress[] = [];

    let canvas: HTMLCanvasElement;
    let context: CanvasRenderingContext2D | null;

    let simulationParameters: SimulationParameters | null;
    let colliderCoordinates: ColliderCoordinate[] = [];

    let max_distance: number,
        force_strength: number,
        force_cutoff: number,
        linear_damping: number,
        logical_height: number,
        logical_width: number,
        friction: number;

    let physicsActive = false;
    let rendering = false;
    let resetting = false;
    let moving = false;
    let scaled = false;
    const dispatch = createEventDispatcher();

    type PhysicsBody = {
        collider: RAPIER.Collider;
        rigidBody: RAPIER.RigidBody;
        voiceNode: VoiceNodeEgress;
    };

    // Canvas dimensions
    const canvasWidth = 600;
    const canvasHeight = 600;
    const usableCanvasWidth = 500;
    const usableCanvasHeight = 500;
    let canvasRatio: number | undefined = undefined;

    onMount(() => {
        if (canvas) {
            context = canvas.getContext('2d');
            canvasRatio = window.devicePixelRatio;
        }
    });

    $: {
        if (
            !scaled &&
            context &&
            logical_height &&
            logical_width &&
            canvasRatio
        ) {
            scaled = true;

            const translateX = (canvasWidth - usableCanvasWidth) / 2;
            const translateY = (canvasHeight - usableCanvasHeight) / 2;
            context?.translate(translateX, translateY);

            context?.scale(
                canvasRatio * (usableCanvasWidth / logical_width),
                canvasRatio * (usableCanvasHeight / logical_height)
            );
        }
    }

    onMount(async () => {
        simulationParameters = await backend.get_simulation_parameters();
        colliderCoordinates = await backend.get_collider_coordinates();

        if (!simulationParameters) return;
        if (colliderCoordinates.length === 0) return;

        max_distance = simulationParameters?.max_distance;
        force_strength = simulationParameters?.force_strength;
        force_cutoff = simulationParameters?.force_cutoff;
        linear_damping = simulationParameters?.linear_damping;
        logical_height = simulationParameters?.logical_height;
        logical_width = simulationParameters?.logical_width;
        friction = simulationParameters?.friction;
    });

    $: if (
        browser &&
        context &&
        nodes.length >= 1 &&
        !rendering &&
        !resetting &&
        simulationParameters
    ) {
        rendering = true;
        setupAndRender();
    }

    $: if (browser && !!nodes && simulationParameters) {
        resetPhysics();
    }

    ///////////PHYSICS//////////

    const setupAndRender = () => {
        RAPIER.init().then(() => {
            if (!simulationParameters) return;
            let world: RAPIER.World;
            let physicsBodies: PhysicsBody[] = [];
            let gravity = {x: 0, y: 0};

            world = new RAPIER.World(gravity);

            // Magnetism parameters in logical coords

            function applyMagnetismForces() {
                physicsBodies.forEach((body, i) => {
                    // Create array of all bodies within reach
                    const cutoffPosition = body.rigidBody.translation();
                    const cutoffRotation = body.rigidBody.rotation();
                    const cutoffShape = new RAPIER.Ball(max_distance);

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
                        undefined,
                        undefined,
                        undefined,
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
                            (1 / distanceBetweenBodies ** 2) * force_strength;
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
                    body.rigidBody.resetForces(true); // TODO: check what happens if you reset after the below check
                    // check if resultant is above threshold
                    const resultantAbs = magneticForces.reduce(
                        (acc, current) => {
                            return (
                                acc + Math.sqrt(current.x ** 2 + current.y ** 2)
                            );
                        },
                        0
                    );

                    if (resultantAbs < force_cutoff) {
                        body.rigidBody.sleep();
                        return;
                    }

                    magneticForces.forEach((force) =>
                        body.rigidBody.addForce(force, true)
                    );
                });
            }

            // create world collider

            const roundInnerColliderDesc = RAPIER.ColliderDesc.polyline(
                convertColliderCoordinatesToFloat32Array(colliderCoordinates)
            ).setFriction(friction);
            world.createCollider(roundInnerColliderDesc);

            // create rigid bodies
            physicsBodies = nodes.map((node) => {
                let rigidBodyDesc = RAPIER.RigidBodyDesc.dynamic()
                    .lockRotations()
                    .setLinearDamping(linear_damping)
                    .setTranslation(Number(node.x), Number(node.y))
                    .setUserData(node);
                let colliderDesc = RAPIER.ColliderDesc.ball(2)
                    .setDensity(2.0) // TODO: get this from node size / sample length.
                    .setFriction(friction);

                const rigidBody = world.createRigidBody(rigidBodyDesc);
                const collider = world.createCollider(colliderDesc, rigidBody);

                return {
                    rigidBody,
                    collider,
                    voiceNode: node,
                };
            });

            render(
                physicsBodies,
                colliderCoordinates,
                world,
                applyMagnetismForces,
                backendNodes
            );
        });
    };

    ////////////////////////////

    /////////RENDERING///////////

    // function mapToCanvasX(logicalX: number) {
    //     return (logicalX / logical_width) * canvasWidth;
    // }

    // function mapToCanvasY(logicalY: number) {
    //     return (logicalY / logical_height) * canvasHeight;
    // }

    // // Converts canvas pixel coordinates to logical coordinates
    // function canvasToLogical(x: number, y: number) {
    //     const logicalX = (x / canvasWidth) * logical_width;
    //     const logicalY = (y / canvasHeight) * logical_height;
    //     return {logicalX, logicalY};
    // }

    // Draw the nodes on the canvas
    function render(
        bodies: PhysicsBody[],
        colliderCoords: ColliderCoordinate[],
        world: any,
        magnetismFunction: Function,
        backendBodies: VoiceNodeEgress[]
    ) {
        // const nodesToDraw = bodies.map((body) => {
        //     return body.voiceNode;
        // });
        if (world && physicsActive) {
            magnetismFunction();
            world.step();
            checkIfStillMoving(bodies);
        }

        if (context) {
            // Clear the canvas
            context.clearRect(0, 0, canvasWidth, canvasHeight);

            // Draw the collider
            if (colliderCoords?.length > 0) {
                context.beginPath();
                context.moveTo(colliderCoords[0].x, colliderCoords[0].y);
                colliderCoords.forEach((coordinate) => {
                    context?.lineTo(coordinate.x, coordinate.y);
                });
                // context.strokeStyle = 'black';
                // context.lineWidth = 0.5;
                // context.stroke();
                context!.fillStyle = `hsl(200 50% 50%)`;
                context!.fill();
                context.closePath();
            }

            // Draw each VoiceNode as a circle using the mapped coordinates
            bodies.forEach((body) => {
                const bodyPos = body.rigidBody.translation();
                const linVel = body.rigidBody.linvel();
                const absVel = Math.sqrt(linVel.x ** 2 + linVel.y ** 2);
                const colorVel = clamp(mapRange(absVel, 0, 2, 0, 100), 0, 100);
                const canvasX = bodyPos.x;
                const canvasY = bodyPos.y;

                context!.beginPath();
                context!.ellipse(canvasX, canvasY, 2, 2, 0, 0, Math.PI * 2);
                context!.fillStyle = `hsl(${30 + colorVel / 10} 80% ${colorVel}% )`;
                context!.fill();
                // context!.lineWidth = 0.5;
                // context!.stroke();
                context!.closePath();
            });

            backendBodies?.forEach((body) => {
                context!.beginPath();
                context!.ellipse(body.x, body.y, 2, 2, 0, 0, Math.PI * 2);
                // context!.fillStyle = `hsl(0 100% 50% )`;
                // context!.fill();
                context!.strokeStyle = `hsl(0 100% 50% )`;
                context!.lineWidth = 0.5;
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

        function stopSlowNodes(bodies: PhysicsBody[], velocityCutoff: number) {
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
                render(
                    bodies,
                    colliderCoordinates,
                    world,
                    magnetismFunction,
                    backendNodes
                );
            }
        }, 16);
    }

    // Handle the click event and map it back to logical coordinates
    function handleClick(e: MouseEvent) {
        const rect = canvas.getBoundingClientRect();
        const canvasX =
            e.clientX - rect.left - (canvasWidth - usableCanvasWidth) / 2;
        const canvasY =
            e.clientY - rect.top - (canvasHeight - usableCanvasHeight) / 2;

        const {logicalX, logicalY} = canvasToLogical(
            canvasX,
            canvasY,
            usableCanvasWidth,
            usableCanvasHeight,
            logical_width,
            logical_height
        );

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
        const canvasX =
            e.clientX - rect.left - (canvasWidth - usableCanvasWidth) / 2;
        const canvasY =
            e.clientY - rect.top - (canvasHeight - usableCanvasHeight) / 2;

        const {logicalX, logicalY} = canvasToLogical(
            canvasX,
            canvasY,
            usableCanvasWidth,
            usableCanvasHeight,
            logical_width,
            logical_height
        );

        console.log('dropped at logical coordinates:', {logicalX, logicalY});

        const nodeRadius = parseFloat(
            e.dataTransfer?.getData('nodeRadius') || '5'
        );

        const distanceFromCenter = Math.sqrt(
            (logicalX - logical_width / 2) ** 2 +
                (logicalY - logical_height / 2) ** 2
        );
        const maxDistance = logical_width / 2 - nodeRadius;

        if (distanceFromCenter > maxDistance) {
            console.log('oiut');
            return;
        }

        const voiceNode = {
            x: logicalX,
            y: logicalY,
            sample: 'todo',
        };

        dispatch('dropNewNode', voiceNode as VoiceNodeIngress);
        rendering = false;
    }
</script>

<main>
    <canvas
        bind:this={canvas}
        width={canvasWidth * (canvasRatio || 1)}
        height={canvasHeight * (canvasRatio || 1)}
        on:click={handleClick}
        on:dragover={handleDragOver}
        on:drop={handleDrop}
        class={`w-[${canvasWidth}px] h-[${canvasHeight}px]`}
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
    <!-- TODO: update nodeWidth with sample length -->
    <DroppableNode nodeWidth={4} />
</main>
