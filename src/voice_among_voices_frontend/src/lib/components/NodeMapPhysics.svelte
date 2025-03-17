<script lang="ts">
    import {onMount} from 'svelte';
    import {mapRange, clamp} from '$lib/utils/mathUtils';
    import RAPIER from '@dimforge/rapier2d-compat';

    import {browser} from '$app/environment';
    import {backend} from '$lib/canisters';
    import type {
        ColliderCoordinate,
        VoiceNodeEgress,
        SimulationParameters,
        VoiceNodeIngress,
    } from '../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did';
    import {
        convertColliderCoordinatesToFloat32Array,
        canvasToLogical,
    } from '$lib/utils/convUtils';
    import {
        canvasWidth as standardCanvasWidth,
        canvasHeight as standardCanvasHeight,
        usableCanvasWidth as standardUsableCanvasWidth,
        usableCanvasHeight as standardUsableCanvasHeight,
        worldStepInterval,
    } from '$lib/config/nodeMap';
    import { simulationParameters, selectedAngle, hoveredAngle } from '$lib/state/uxState.svelte';
    import { isDarkMode } from "$lib/utils/uxUtils";
    let { nodes, backendNodes, dragging, showPlayHead = true, playHeadPosition = 0, playHeadAngle = 0, dropNewNode, movePlayHead, class: classes = "" }: {
        nodes: VoiceNodeEgress[];
        backendNodes: VoiceNodeEgress[];
        dragging: boolean;
        showPlayHead: boolean;
        playHeadPosition: number;
        playHeadAngle: number;
        dropNewNode: (voiceNode: VoiceNodeIngress) => void;
        movePlayHead: (normalizedPosition: number) => void;
        class?: string;
    } = $props();
    

    let localNodes: VoiceNodeEgress[] = [];

    let container: HTMLDivElement | null = null;
    let canvas: HTMLCanvasElement | null = null;
    let context: CanvasRenderingContext2D | null = null;

    let canvasDiameter = $state(800);
    let marginDivisor = 12; // total margin is 50 per default
    let canvasMargin = $derived(canvasDiameter / marginDivisor);
    let usableCanvasDiameter = $derived(canvasDiameter - 2 * canvasMargin);

    let colliderCoordinates: ColliderCoordinate[] = $state([]);
    const drawCollider = false;
    const bodyLineWidth = 0.3;

    let max_distance: number = $state(0);
    let force_strength: number = $state(0);
    let force_cutoff: number = $state(0);
    let linear_damping: number = $state(0);
    let logical_radius: number = $state(0);
    let friction: number = $state(0);

    let physicsActive = $state(false);
    let rendering = $state(false);
    let resetting = $state(false);
    let moving = $state(false);
    let scaled = $state(false);
    let fastForward = $state(false);

    type PhysicsBody = {
        collider: RAPIER.Collider;
        rigidBody: RAPIER.RigidBody;
        voiceNode: VoiceNodeEgress;
    };

    let canvasRatio: number = $state(1);

    onMount(() => {
        if (canvas) {
            context = canvas.getContext('2d');
            canvasRatio = 1; // TODO
            // canvasRatio = window.devicePixelRatio || 1;
        }
    });

    $effect(() => {
        if (!scaled && context && logical_radius && canvasRatio) {
            scaled = true;

            setupContext();
        }
    });

    function setupContext() {
        const translateX = canvasRatio * canvasDiameter / 2;
        const translateY = canvasRatio * canvasDiameter / 2;
        context?.translate(translateX, translateY);

        context?.scale(
            canvasRatio * (usableCanvasDiameter / (2 * logical_radius)),
            -canvasRatio * (usableCanvasDiameter / (2 * logical_radius))
        );
    }

    onMount(async () => {
        $simulationParameters = await backend.get_simulation_parameters();
        colliderCoordinates = await backend.get_collider_coordinates();
        console.log($simulationParameters);
        if (!$simulationParameters) return;
        if (colliderCoordinates.length === 0) return;

        max_distance = $simulationParameters!.max_distance;
        force_strength = $simulationParameters!.force_strength;
        force_cutoff = $simulationParameters!.force_cutoff;
        linear_damping = $simulationParameters!.linear_damping;
        logical_radius = $simulationParameters!.logical_radius;
        friction = $simulationParameters!.friction;
    });

    $effect(() => {if (
        browser &&
        context &&
        localNodes.length >= 1 &&
        !rendering &&
        !resetting &&
        $simulationParameters
    ) {
        rendering = true;
        setupAndRender();
    }});


    $effect(() => {if (browser && !!localNodes && $simulationParameters) {
        resetPhysics();
    }});

    $effect(() => {if (localNodes.length === 0 && nodes.length > 0) {
        resetNodes();
    }});

    $effect(() => {if (dragging) {
        resetNodes();
    }});

    function resetNodes() {
        localNodes = [...nodes];
    }

    ///////////PHYSICS//////////

    const setupAndRender = () => {
        RAPIER.init().then(() => {
            if (!$simulationParameters) return;
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
                    // const resultantAbs = magneticForces.reduce(
                    //     (acc, current) => {
                    //         return (
                    //             acc + Math.sqrt(current.x ** 2 + current.y ** 2)
                    //         );
                    //     },
                    //     0
                    // );

                    // if (resultantAbs < force_cutoff) {
                    //     body.rigidBody.sleep();
                    //     return;
                    // }

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
            physicsBodies = localNodes.map((node) => {
                let rigidBodyDesc = RAPIER.RigidBodyDesc.dynamic()
                    .lockRotations()
                    .setLinearDamping(linear_damping)
                    .setTranslation(Number(node.x), Number(node.y))
                    .setUserData(node);
                let colliderDesc = RAPIER.ColliderDesc.ball(node.radius) //ts glitching here lol
                    .setDensity(2.0)
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

    /////////RENDERING///////////

    // Draw the nodes on the canvas

    let then: number | undefined;
    let now: number | undefined;

    function checkIfStillMoving(bodies: PhysicsBody[]) {
        moving = false;
        bodies.forEach((body) => {
            if (body.rigidBody.isMoving()) {
                moving = true;
            }
        });
    }

    // function stopSlowNodes(bodies: PhysicsBody[], velocityCutoff: number) {
    //     bodies.forEach((body) => {
    //         const linVel = body.rigidBody.linvel();
    //         const absVel = Math.sqrt(linVel.x ** 2 + linVel.y ** 2);

    //         if (absVel < velocityCutoff) {
    //             body.rigidBody.setLinvel({x: 0, y: 0}, true);
    //         }
    //     });
    // }

    function drawPlayHead(context: CanvasRenderingContext2D) {
        if (!showPlayHead || !context) return;

        // Convert angle to radians
        const angleRadians = playHeadAngle * (Math.PI / 180);

        // Get start and end points on the circle (logical_radius)
        const startX = Math.sin(angleRadians) * logical_radius;
        const startY = Math.cos(angleRadians) * logical_radius;
        const endX = Math.sin(angleRadians) * -logical_radius;
        const endY = Math.cos(angleRadians) * -logical_radius;

        // Interpolate between start and end based on playHeadPosition
        const playHeadX = startX + (endX - startX) * playHeadPosition;
        const playHeadY = startY + (endY - startY) * playHeadPosition;

        // Calculate the tangent vector (rotate radial vector by 90 degrees counterclockwise)
        const radialVectorX = playHeadX;
        const radialVectorY = playHeadY;

        // The tangent vector perpendicular to the radial vector
        const tangentX = -radialVectorY; // Negate the Y for 90 degree rotation
        const tangentY = radialVectorX; // X remains the same

        // Normalize the tangent vector to ensure it has a unit length
        const tangentLength = Math.sqrt(
            tangentX * tangentX + tangentY * tangentY
        );
        const normalizedTangentX = tangentX / tangentLength;
        const normalizedTangentY = tangentY / tangentLength;

        // Line half-length (so it extends equally on both sides)
        const lineLength = 2 * logical_radius; // Adjust if needed.

        // Calculate the points for the perpendicular line (centered at playHeadX, playHeadY)
        // Find the intersection points of the tangent line with the circle
        const distanceFromCenter = Math.sqrt(playHeadX * playHeadX + playHeadY * playHeadY);
        
        // Calculate the maximum length the line can have at this point to stay within the circle
        const maxLineLength = 2 * Math.sqrt(logical_radius * logical_radius - distanceFromCenter * distanceFromCenter);
        
        // Use the smaller of the two: either the calculated max length or the original line length
        const actualLineLength = Math.min(maxLineLength, lineLength);
        
        const lineStartX = playHeadX + normalizedTangentX * (actualLineLength / 2);
        const lineStartY = playHeadY + normalizedTangentY * (actualLineLength / 2);
        const lineEndX = playHeadX - normalizedTangentX * (actualLineLength / 2);
        const lineEndY = playHeadY - normalizedTangentY * (actualLineLength / 2);

        // Draw the perpendicular playhead line
        context.beginPath();
        context.moveTo(lineStartX, lineStartY);
        context.lineTo(lineEndX, lineEndY);
        context.lineWidth = 0.2; // Line width

        context.strokeStyle = isDarkMode() ? `hsl(0, 0%, 100%)` : `hsl(0, 0%, 0%)`;  // Color of the playhead line

        context.stroke();
        context.closePath();
    }

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
        requestAnimationFrame((timestamp) => {
            if (then === undefined) {
                then = timestamp;
            }

            now = timestamp;
            const elapsed = now - then;
            const interval = fastForward ? 0.1 : worldStepInterval;

            if (elapsed > interval) {
                then = now - (elapsed % interval);

                if (world && physicsActive) {
                    magnetismFunction();
                    world.step();
                    checkIfStillMoving(bodies);
                }

                if (context) {
                    // Clear the canvas
                    context.clearRect(
                        -canvasDiameter / 2,
                        -canvasDiameter / 2,
                        canvasDiameter,
                        canvasDiameter
                    );

                    // context!.rect(-500, -500, 1000, 1000); //TODO remove this
                    //     context!.fillStyle = `hsl(0 100% 50%)`;
                    //     context!.fill();



                    // Draw the collider
                    if (drawCollider && colliderCoords?.length > 0) {
                        context.beginPath();
                        context.moveTo(
                            colliderCoords[0].x,
                            colliderCoords[0].y
                        );
                        colliderCoords.forEach((coordinate) => {
                            context?.lineTo(
                                coordinate.x,
                                coordinate.y
                            );
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
                        const colorVel = clamp(
                            mapRange(absVel, 0, 2, 0, 100),
                            0,
                            100
                        );
                        const canvasX = bodyPos.x;
                        const canvasY = bodyPos.y;
    
                        const isTouchedByPlayhead = isNodeTouchedByPlayhead(
                            canvasX, 
                            canvasY, 
                            body.voiceNode.radius * 10
                        );

                        context!.beginPath();
                        context!.ellipse(
                            canvasX,
                            canvasY,
                            body.voiceNode.radius,
                            body.voiceNode.radius,
                            0,
                            0,
                            Math.PI * 2
                        );
                        if(isTouchedByPlayhead) {
                            context!.fillStyle = `hsl(${body.voiceNode.id}, 100%, 50%)`;
                        } else if($selectedAngle && body.voiceNode.id === BigInt($selectedAngle)) {
                            context!.fillStyle = `hsl(${$selectedAngle}, 100%, 50%)`;
                        } else if ($hoveredAngle && body.voiceNode.id === BigInt($hoveredAngle)) {
                            context!.fillStyle = `hsl(${$hoveredAngle}, 100%, 80%)`;
                        } else {
                            context!.fillStyle = `hsla(${30 + colorVel / 10}, 80%, ${colorVel}%, ${colorVel}%)`;
                        }
                        context!.fill();
                        context!.lineWidth = bodyLineWidth;
                        context!.strokeStyle = `hsl(${body.voiceNode.id} 100% 50% )`
                        context!.stroke();
                        context!.closePath();
                    });

                    backendBodies?.forEach((body) => {
                        context!.beginPath();
                        context!.ellipse(
                            body.x,
                            body.y,
                            body.radius + 1,
                            body.radius + 1,
                            0,
                            0,
                            Math.PI * 2
                        );

                        context!.strokeStyle = isDarkMode() ? `hsl(0, 0%, 100%)` : `hsl(0, 0%, 0%)`;  // Color of the playhead line

                        context!.lineWidth = 0.5;
                        context!.stroke();
                        context!.closePath();
                    });
                    if (showPlayHead) {
                        drawPlayHead(context);
                    }
                }
            }
            if (rendering) {
                render(
                    bodies,
                    colliderCoordinates,
                    world,
                    magnetismFunction,
                    backendNodes
                );
            }
        });
    }

    function handleClick(e: MouseEvent) {
        const rect = canvas!.getBoundingClientRect();
        const canvasX = (e.clientX - rect.left) * canvasRatio;
        const canvasY = (e.clientY - rect.top) * canvasRatio;


        //canvasX needs to add canvaswidth-usablecanvaswitdh/2

        const {logicalX, logicalY} = canvasToLogical(
            canvasX,
            canvasY,
            usableCanvasDiameter,
            canvasMargin,
            logical_radius
        );

        console.log(logicalX, logicalY);

        if (showPlayHead) {
            // Calculate the position along the playHead
            const angleRadians = playHeadAngle * (Math.PI / 180);

            // Tangent points (start and end of the line)
            const startX = Math.sin(angleRadians) * logical_radius;
            const startY = Math.cos(angleRadians) * logical_radius;
            const endX = Math.sin(angleRadians) * -logical_radius;
            const endY = Math.cos(angleRadians) * -logical_radius;

            // Project the click position onto the line (start -> end)
            const lineLength = Math.sqrt(
                (endX - startX) ** 2 + (endY - startY) ** 2
            );
            const dotProduct =
                ((logicalX - startX) * (endX - startX) +
                    (logicalY - startY) * (endY - startY)) /
                lineLength;

            // Normalize the dotProduct to get playHeadPosition between 0 and 1
            const normalizedPosition = clamp(dotProduct / lineLength, 0, 1);

            // Dispatch the movePlayHead event with the normalized position
            movePlayHead(normalizedPosition);
        }
    }

    function handleFastForward() {
        resetNodes();
        backendNodes = [];
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
        const rect = canvas!.getBoundingClientRect();
        const canvasX = (e.clientX - rect.left) * canvasRatio;
        const canvasY = (e.clientY - rect.top) * canvasRatio;


        const {logicalX, logicalY} = canvasToLogical(
            canvasX,
            canvasY,
            usableCanvasDiameter,
            canvasMargin,
            logical_radius
        );

        const nodeRadius = parseFloat(
            e.dataTransfer?.getData('nodeRadius') || '5'
        );

        const distanceFromCenter = Math.sqrt(logicalX ** 2 + logicalY ** 2);
        const maxDistance = logical_radius - nodeRadius;

        if (distanceFromCenter > maxDistance) {
            console.log('out');
            return;
        }

        const voiceNode: VoiceNodeIngress = {
            id: BigInt($selectedAngle!),
            x: logicalX,
            y: logicalY,
            sample: [],
        };
        dropNewNode(voiceNode);
        let updatableNode = localNodes.find(
            (node) => node.id === BigInt($selectedAngle!)
        );

        if (updatableNode) {
            updatableNode.x = logicalX;
            updatableNode.y = logicalY;
            updatableNode.radius = nodeRadius;
        } else {
            localNodes.push({
                x: logicalX,
                y: logicalY,
                radius: nodeRadius,
                id: BigInt($selectedAngle!),
            });
        }

        resetPhysics();
        physicsActive = true;
        rendering = false;
    }

        // Update canvas size when container size changes
        function updateCanvasSize() {
            if (!container) return;
            
            const rect = container.getBoundingClientRect();
            canvasDiameter = Math.min(rect.width, rect.height);
            
            if (canvas) {
                canvas.width = canvasDiameter * canvasRatio;
                canvas.height = canvasDiameter * canvasRatio;
                
                // Reset context and scaling when size changes
                if (context && canvasRatio && logical_radius && canvasDiameter > 0 && canvasDiameter > 0) {
                    context.setTransform(1, 0, 0, 1, 0, 0); // Reset transform
                    const translateX = canvasRatio * canvasDiameter / 2;
                    const translateY = canvasRatio * canvasDiameter / 2;
                    context.translate(translateX, translateY);

                    context.scale(
                        canvasRatio * canvasDiameter / logical_radius / 2 * (usableCanvasDiameter / canvasDiameter),
                        -canvasRatio * canvasDiameter / logical_radius / 2 * (usableCanvasDiameter / canvasDiameter)
                    );
                } else {
                    setTimeout(updateCanvasSize, 10);
                }
            }
        }

        function isNodeTouchedByPlayhead(nodeX: number, nodeY: number, nodeRadius: number): boolean { //TODO: not working yet for some reason
            // if (!showPlayHead) return false;
    
            // // Convert angle to radians
            // const angleRadians = playHeadAngle * (Math.PI / 180);
            
            // // Get start and end points on the circle
            // const startX = Math.sin(angleRadians) * logical_radius;
            // const startY = Math.cos(angleRadians) * logical_radius;
            // const endX = Math.sin(angleRadians) * -logical_radius;
            // const endY = Math.cos(angleRadians) * -logical_radius;
            
            // // Interpolate between start and end based on playHeadPosition to get actual playhead point
            // const playHeadX = startX + (endX - startX) * playHeadPosition;
            // const playHeadY = startY + (endY - startY) * playHeadPosition;
            
            // // Calculate distance from node center to playhead point
            // const distance = Math.sqrt((nodeX - playHeadX) ** 2 + (nodeY - playHeadY) ** 2);
            // console.log(distance, nodeRadius);
            // // Node is touched if distance is less than or equal to node radius
            // return distance <= nodeRadius;
            return false
        }

        onMount(() => {
            if (canvas) {
                context = canvas.getContext('2d');
                canvasRatio = 1;
                
                // Set up resize observer
                const resizeObserver = new ResizeObserver(() => {
                    updateCanvasSize();
                });
                
                resizeObserver.observe(container!);
                updateCanvasSize(); // Initial size
                
                return () => resizeObserver.disconnect();
            }
        });

        $effect(() => {
            if(logical_radius) updateCanvasSize();
        });
</script>

<div
    bind:this={container} 
    class={`relative ${classes} w-full flex items-center justify-center`}
>
    <canvas
        bind:this={canvas}
        width={canvasDiameter * canvasRatio}
        height={canvasDiameter * canvasRatio}
        onclick={handleClick}
        ondragover={handleDragOver}
        ondrop={handleDrop}
        class={`w-[${canvasDiameter}px] h-[${canvasDiameter}px]`}
    ></canvas>
</div>
<!-- TODO handle these -->
<!-- <label for="node-id">node id</label>
<input
    id="node-id"
    type="number"
    min="0"
    max="359"
    bind:value={nodeId}
/>
{#if physicsActive && backendNodes.length > 0}
    <button
        class="hover:shadow-lg rounded-full bg-slate-500 px-5"
        onpointerdown={handleFastForward}
    >
        >>
    </button>
{/if} -->
