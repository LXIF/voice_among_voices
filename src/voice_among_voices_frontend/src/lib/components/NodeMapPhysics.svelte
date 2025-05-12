<script lang="ts">
    import { onMount, untrack } from "svelte";
    import { mapRange, clamp } from "$lib/utils/mathUtils";
    import RAPIER from "@dimforge/rapier2d-compat";
    // import RAPIER from "@dimforge/rapier2d-deterministic";

    import { browser } from "$app/environment";
    import type {
        ColliderCoordinate,
        VoiceNodeEgress,
        VoiceNodeIngress,
    } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import {
        convertColliderCoordinatesToFloat32Array,
        canvasToLogical,
    } from "$lib/utils/convUtils";
    import {
        canvasWidth as standardCanvasWidth,
        canvasHeight as standardCanvasHeight,
        usableCanvasWidth as standardUsableCanvasWidth,
        usableCanvasHeight as standardUsableCanvasHeight,
        worldStepInterval,
    } from "$lib/config/nodeMap";
    import {
        simulationParameters,
        selectedAngle,
        hoveredAngle,
        mapRotation,
        applicationState,
        applicationStates,
        voiceNodes,
        toastMessage,
    } from "$lib/state/uxState";
    import { isDarkMode } from "$lib/utils/uxUtils";
    import {
        getColliderCoordinates,
        getSimulationParameters,
        getVoiceNodes,
    } from "$lib/icInteractions";

    let {
        nodes,
        backendNodes,
        showPlayHead = true,
        playHeadPosition = 0,
        playHeadAngle = 0,
        dropNewNode,
        movePlayHead,
        onResetNodes,
        class: classes = "",
    }: {
        nodes: VoiceNodeEgress[];
        backendNodes: VoiceNodeEgress[];
        showPlayHead: boolean;
        playHeadPosition: number;
        playHeadAngle: number;
        dropNewNode: (voiceNode: VoiceNodeIngress) => void;
        movePlayHead: (normalizedPosition: number) => void;
        onResetNodes?: () => void;
        class?: string;
    } = $props();

    let localNodes: VoiceNodeEgress[] = $state([]);

    let container: HTMLDivElement | null = null;
    let svgElement: SVGSVGElement | null = null;

    let canvasDiameter = $state(800);
    let marginDivisor = 12; // margin is 50 per default
    let canvasMargin = $derived(canvasDiameter / marginDivisor);
    let usableCanvasDiameter = $derived(canvasDiameter - 2 * canvasMargin);

    let colliderCoordinates: ColliderCoordinate[] = $state([]);
    const drawCollider = false;
    const bodyLineWidth = 0.3;

    let max_distance: number = 0;
    let force_strength: number = 0;
    let linear_damping: number = 0;
    let logical_radius: number = 0;
    let friction: number = 0;
    let density: number = 0;
    let max_steps: number = 0;

    let steps: number = 0;

    // let physicsActive = $state(false);
    let resetting = $state(false);
    let moving = $state(false);
    let fastForward = $state(false);

    // let contentRect = $state<DOMRectReadOnly>();

    // let lastAppliedRotation = 180;

    type PhysicsBody = {
        collider: RAPIER.Collider;
        rigidBody: RAPIER.RigidBody;
        voiceNode: VoiceNodeEgress;
    };

    let canvasRatio: number = $state(1);

    // onMount(() => {
    //     if (canvas) {
    //         context = canvas.getContext("2d");
    //         canvasRatio = 1; // TODO
    //         // canvasRatio = window.devicePixelRatio || 1;
    //     }
    // });

    function transformColliderCoordinates(
        coords: { x: number; y: number }[],
    ): string {
        let outputString = "";
        coords.forEach((coord) => {
            outputString = `${outputString} ${coord.x},${coord.y}`;
        });
        return outputString;
    }

    function rotatedCoords(
        x: number,
        y: number,
        rotation: number,
    ): { rotatedX: number; rotatedY: number } {
        // Convert rotation from degrees to radians
        const angleRad = (rotation * Math.PI) / 180;

        // Calculate the rotated coordinates using rotation matrix
        const rotatedX = x * Math.cos(angleRad) - y * Math.sin(angleRad);
        const rotatedY = x * Math.sin(angleRad) + y * Math.cos(angleRad);

        return { rotatedX, rotatedY };
    }

    function rotatedX(node: VoiceNodeEgress, rotation: number) {
        // Convert rotation from degrees to radians
        const angleRad = (rotation * Math.PI) / 180;

        return node.x * Math.cos(angleRad) - node.y * Math.sin(angleRad);
    }

    function rotatedY(node: VoiceNodeEgress, rotation: number) {
        // Convert rotation from degrees to radians
        const angleRad = (rotation * Math.PI) / 180;

        return node.x * Math.sin(angleRad) + node.y * Math.cos(angleRad);
    }

    onMount(async () => {
        $simulationParameters = await getSimulationParameters();
        colliderCoordinates = await getColliderCoordinates();

        if (!$simulationParameters) return;
        if (colliderCoordinates.length === 0) return;

        max_distance = $simulationParameters!.max_distance;
        force_strength = $simulationParameters!.force_strength;
        linear_damping = $simulationParameters!.linear_damping;
        logical_radius = $simulationParameters!.logical_radius;
        friction = $simulationParameters!.friction;
        density = $simulationParameters!.density;
        max_steps = Number($simulationParameters.max_steps);
    });

    // $effect(() => {
    //     if (
    //         browser &&
    //         svgElement &&
    //         localNodes.length >= 1 &&
    //         !rendering &&
    //         !resetting &&
    //         $simulationParameters
    //         //   nonNullish(colliderCoordinates)
    //     ) {
    //         rendering = true;
    //         setupAndSimulate();
    //     }
    // });

    $effect(() => {
        if (
            browser &&
            $simulationParameters &&
            colliderCoordinates.length > 0
        ) {
            untrack(() => resetPhysics());
        }
    });

    $effect(() => {
        if (localNodes.length === 0 && nodes.length > 0) {
            resetNodes();
        }
    });

    function resetNodes() {
        // if (backendNodes.length > 0) {
        //     console.log("resetting with backend");
        //     localNodes = [...backendNodes];
        //     onResetNodes?.();
        //     $voiceNodes = await getVoiceNodes();
        //     return;
        // }
        console.log("Resetting nodes...");
        localNodes = [...nodes];
    }

    ///////////PHYSICS//////////

    const setupAndSimulate = () => {
        console.log("Setting up physics engine and rendering...");
        RAPIER.init()
            .then(() => {
                if (!$simulationParameters) {
                    console.error("no simulation parameters");
                    $toastMessage = "No simulation parameters";
                    return;
                }
                let world: RAPIER.World;
                let physicsBodies: PhysicsBody[] = [];
                let gravity = { x: 0, y: 0 };

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
                            body.rigidBody,
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
                                    (bodyWithinReachPos.y - bodyPos.y) ** 2,
                            );
                            const magneticForceScalar =
                                (1 / distanceBetweenBodies ** 2) *
                                force_strength;
                            const vectorBetweenBodies = new RAPIER.Vector2(
                                bodyPos.x - bodyWithinReachPos.x,
                                bodyPos.y - bodyWithinReachPos.y,
                            );
                            // add vector to the magnetic forces
                            magneticForces.push(
                                new RAPIER.Vector2(
                                    vectorBetweenBodies.x * magneticForceScalar,
                                    vectorBetweenBodies.y * magneticForceScalar,
                                ),
                            );
                        });

                        // reset the forces
                        body.rigidBody.resetForces(true);

                        magneticForces.forEach((force) =>
                            body.rigidBody.addForce(force, true),
                        );
                    });
                }

                // create world collider

                const roundInnerColliderDesc = RAPIER.ColliderDesc.polyline(
                    convertColliderCoordinatesToFloat32Array(
                        colliderCoordinates,
                    ),
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
                        .setDensity(density)
                        .setFriction(friction);

                    const rigidBody = world.createRigidBody(rigidBodyDesc);
                    const collider = world.createCollider(
                        colliderDesc,
                        rigidBody,
                    );

                    return {
                        rigidBody,
                        collider,
                        voiceNode: node,
                    };
                });

                simulate(physicsBodies, world, applyMagnetismForces);
            })
            .then(() => {
                resetNodes();
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

    // function drawPlayHead(context: CanvasRenderingContext2D) {
    //     if (!showPlayHead || !context) return;

    //     // Convert angle to radians
    //     const angleRadians = playHeadAngle * (Math.PI / 180);

    //     // Get start and end points on the circle (logical_radius)
    //     const startX = Math.sin(angleRadians) * logical_radius;
    //     const startY = Math.cos(angleRadians) * logical_radius;
    //     const endX = Math.sin(angleRadians) * -logical_radius;
    //     const endY = Math.cos(angleRadians) * -logical_radius;

    //     // Interpolate between start and end based on playHeadPosition
    //     const playHeadX = startX + (endX - startX) * playHeadPosition;
    //     const playHeadY = startY + (endY - startY) * playHeadPosition;

    //     // Calculate the tangent vector (rotate radial vector by 90 degrees counterclockwise)
    //     const radialVectorX = playHeadX;
    //     const radialVectorY = playHeadY;

    //     // The tangent vector perpendicular to the radial vector
    //     const tangentX = -radialVectorY; // Negate the Y for 90 degree rotation
    //     const tangentY = radialVectorX; // X remains the same

    //     // Normalize the tangent vector to ensure it has a unit length
    //     const tangentLength = Math.sqrt(
    //         tangentX * tangentX + tangentY * tangentY,
    //     );
    //     const normalizedTangentX = tangentX / tangentLength;
    //     const normalizedTangentY = tangentY / tangentLength;

    //     // Line half-length (so it extends equally on both sides)
    //     const lineLength = 2 * logical_radius; // Adjust if needed.

    //     // Calculate the points for the perpendicular line (centered at playHeadX, playHeadY)
    //     // Find the intersection points of the tangent line with the circle
    //     const distanceFromCenter = Math.sqrt(
    //         playHeadX * playHeadX + playHeadY * playHeadY,
    //     );

    //     // Calculate the maximum length the line can have at this point to stay within the circle
    //     const maxLineLength =
    //         2 *
    //         Math.sqrt(
    //             logical_radius * logical_radius -
    //                 distanceFromCenter * distanceFromCenter,
    //         );

    //     // Use the smaller of the two: either the calculated max length or the original line length
    //     const actualLineLength = Math.min(maxLineLength, lineLength);

    //     const lineStartX =
    //         playHeadX + normalizedTangentX * (actualLineLength / 2);
    //     const lineStartY =
    //         playHeadY + normalizedTangentY * (actualLineLength / 2);
    //     const lineEndX =
    //         playHeadX - normalizedTangentX * (actualLineLength / 2);
    //     const lineEndY =
    //         playHeadY - normalizedTangentY * (actualLineLength / 2);

    //     // Draw the perpendicular playhead line
    //     context.beginPath();
    //     context.moveTo(lineStartX, lineStartY);
    //     context.lineTo(lineEndX, lineEndY);
    //     context.lineWidth = 0.2; // Line width

    //     context.strokeStyle = isDarkMode()
    //         ? `hsl(0, 0%, 100%)`
    //         : `hsl(0, 0%, 0%)`; // Color of the playhead line

    //     context.stroke();
    //     context.closePath();
    // }

    function simulate(
        bodies: PhysicsBody[],
        world: any,
        magnetismFunction: Function,
    ) {
        requestAnimationFrame((timestamp) => {
            if (then === undefined) {
                then = timestamp;
            }

            now = timestamp;
            const elapsed = now - then;
            const interval = fastForward ? 0.1 : worldStepInterval;

            if (elapsed > interval) {
                then = now - (elapsed % interval);

                if (world && $applicationState.physicsActive) {
                    steps++;

                    if (steps > max_steps) {
                        // $applicationState = applicationStates.loggedInIdle;
                        // TODO: stop simulating
                        $applicationState.physicsActive = false;
                    }
                    magnetismFunction();
                    world.step();
                    checkIfStillMoving(bodies);
                    // update localNodes based off bodies

                    localNodes = bodies.map((body) => {
                        const { x, y } = body.rigidBody.translation();
                        return {
                            x,
                            y,
                            id: body.voiceNode.id,
                            radius: body.voiceNode.radius,
                        };
                    });
                    simulate(bodies, world, magnetismFunction);
                }
            }
        });
    }

    function handleClick(e: MouseEvent) {
        const rect = svgElement!.getBoundingClientRect();
        const mapY = e.clientY - rect.top;

        if (showPlayHead) {
            const normalizedPosition = 1 - mapY / rect.height;
            // Dispatch the movePlayHead event with the normalized position
            movePlayHead(normalizedPosition);
        }
    }

    function handleFastForward() {
        resetNodes();
        backendNodes = [];
    }

    function resetPhysics() {
        console.log("Resetting physics...");
        steps = 0;
        setTimeout(() => {
            //let last frame play out
            setupAndSimulate();
        }, 50);
    }

    /////for drag and drop/////

    function handleDragOver(e: DragEvent) {
        e.preventDefault();
    }

    export async function handleDrop({
        nodeX,
        nodeY,
        nodeRadius,
    }: {
        nodeX: number;
        nodeY: number;
        nodeRadius: number;
    }) {
        resetNodes();
        const rect = svgElement!.getBoundingClientRect();

        const mapX = nodeX - rect.left;
        const mapY = nodeY - rect.top;
        const normalizedX = mapX / rect.width;
        const normalizedY = mapY / rect.height;
        const logicalX = normalizedX * 100 - 50;
        const logicalY = normalizedY * 100 - 50;
        const { rotatedX, rotatedY } = rotatedCoords(
            logicalX,
            logicalY,
            -mapRotation.current + 180,
        );

        const distanceFromCenter = Math.sqrt(rotatedX ** 2 + rotatedY ** 2);
        const maxDistance = logical_radius - nodeRadius;

        if (distanceFromCenter > maxDistance) {
            console.log("out");
            return;
        }

        const voiceNode: VoiceNodeIngress = {
            id: BigInt($selectedAngle!),
            x: rotatedX,
            y: rotatedY,
            sample: [],
        };
        dropNewNode(voiceNode);
        let updatableNode = localNodes.find(
            (node) => node.id === BigInt($selectedAngle!),
        );

        if (updatableNode) {
            updatableNode.x = rotatedX;
            updatableNode.y = rotatedY;
            updatableNode.radius = nodeRadius;
        } else {
            localNodes.push({
                x: rotatedX,
                y: rotatedY,
                radius: nodeRadius,
                id: BigInt($selectedAngle!),
            });
        }

        $applicationState = applicationStates.loadingBackendResult;
        resetPhysics();
    }

    // Update canvas size when container size changes
    // function updateCanvasSize() {
    //     console.log("Updating canvas size...");
    //     if (!container) return;

    //     const rect = container.getBoundingClientRect();
    //     if (rect.height <= rect.width) {
    //         canvasDiameter = Math.min(
    //             Math.max(rect.width, rect.height),
    //             Math.min(innerHeight, innerWidth),
    //         );
    //     } else {
    //         canvasDiameter = rect.width;
    //     }

    //     if (canvas) {
    //         canvas.width = canvasDiameter * canvasRatio;
    //         canvas.height = canvasDiameter * canvasRatio;

    //         // Reset context and scaling when size changes
    //         if (
    //             context &&
    //             canvasRatio > 0 &&
    //             logical_radius &&
    //             canvasDiameter > 0
    //         ) {
    //             context.reset();
    //             context.setTransform(1, 0, 0, 1, 0, 0); // Reset transform
    //             const scaledCanvasDiameter = canvasRatio * canvasDiameter;
    //             const scaledCanvasRadius = scaledCanvasDiameter / 2;
    //             const canvasToLogicalRadiusRatio =
    //                 scaledCanvasRadius / logical_radius;
    //             const usableToTotalDiameterRatio =
    //                 usableCanvasDiameter / canvasDiameter;
    //             const translateX = scaledCanvasDiameter / 2;
    //             const translateY = translateX;
    //             context.translate(translateX, translateY);
    //             context.scale(
    //                 // invert x to align with stereo image
    //                 -canvasToLogicalRadiusRatio * usableToTotalDiameterRatio,
    //                 canvasToLogicalRadiusRatio * usableToTotalDiameterRatio,
    //             );
    //             context.rotate(
    //                 ((lastAppliedRotation - mapRotation.current) / 180) *
    //                     Math.PI,
    //             );
    //         } else {
    //             setTimeout(updateCanvasSize, 10);
    //         }
    //     }
    // }

    function isNodeTouchedByPlayhead(
        nodeX: number,
        nodeY: number,
        nodeRadius: number,
    ): boolean {
        if (!showPlayHead) return false;

        // Get current rotation in radians
        const rotationRadians = (mapRotation.current * Math.PI) / 180;

        // Rotate the node position by the inverse of the canvas rotation
        // This gives us the node position in the non-rotated coordinate system
        const rotatedNodeX =
            nodeX * Math.cos(rotationRadians) +
            nodeY * Math.sin(rotationRadians);
        const rotatedNodeY =
            nodeX * Math.sin(rotationRadians) -
            nodeY * Math.cos(rotationRadians);

        // Convert playhead angle to radians
        const angleRadians = (playHeadAngle * Math.PI) / 180;

        // Get tangency points
        let tangentX = logical_radius * Math.sin(angleRadians);
        let tangentY = logical_radius * Math.cos(angleRadians);
        let distanceToTangent;

        if (playHeadAngle % 180 === 0) {
            distanceToTangent = Math.abs(rotatedNodeY - tangentY);
        } else if (playHeadAngle % 90 === 0) {
            distanceToTangent = Math.abs(rotatedNodeX - tangentX);
        } else {
            // Tangent slope
            let tangentSlope = -tangentX / tangentY;

            // Tangent line equation: y - tangentY = tangentSlope * (x - tangentX)
            // Rewriting it as Ax + By + C = 0
            let aTangent = tangentSlope;
            let bTangent = -1;
            let cTangent = -tangentSlope * tangentX + tangentY;

            // Calculate distance from rotated node to the tangent line
            distanceToTangent =
                Math.abs(
                    aTangent * rotatedNodeX +
                        bTangent * rotatedNodeY +
                        cTangent,
                ) / Math.sqrt(aTangent * aTangent + bTangent * bTangent);
        }

        // Map the playhead position to logical coordinates
        const mappedPlayHeadPosition = playHeadPosition * 2 * logical_radius;

        // Calculate distance from the node to the playhead
        const distanceFromPlayhead = Math.abs(
            distanceToTangent - mappedPlayHeadPosition,
        );

        return distanceFromPlayhead <= nodeRadius;
    }
</script>

<div
    bind:this={container}
    class={`relative ${classes} flex w-full items-center justify-center p-[8%]`}
>
    <svg
        bind:this={svgElement}
        viewBox="-50 -50 100 100"
        ondragover={handleDragOver}
        onclick={handleClick}
        role="application"
        id="node-map"
        class="min-w-full"
    >
        {#if drawCollider}
            <polygon
                points={transformColliderCoordinates(colliderCoordinates)}
                fill="green"
            />
        {/if}
        <g style={`transform: rotate(${mapRotation.current + 180}deg);`}>
            {#each localNodes as node}
                <circle
                    cx={node.x}
                    cy={node.y}
                    r={node.radius}
                    stroke="hsl({node.id}, 100%, 50%)"
                    stroke-width={bodyLineWidth}
                    fill="none"
                />
            {/each}
            {#each backendNodes as node}
                <circle
                    cx={node.x}
                    cy={node.y}
                    r={node.radius + 0.5}
                    stroke-width={bodyLineWidth}
                    fill="none"
                    class="stroke-slate-950 dark:stroke-white"
                />
            {/each}
        </g>
        {#if showPlayHead && playHeadPosition > 0}
            <line
                class="stroke-slate-950 dark:stroke-white"
                x1="-50"
                x2="50"
                y1={-(playHeadPosition * 100 - 50)}
                y2={-(playHeadPosition * 100 - 50)}
                stroke-width="0.2"
            />
        {/if}
    </svg>
</div>

<!-- TODO handle this -->
<!-- 
{#if physicsActive && backendNodes.length > 0}
    <button
        class="hover:shadow-lg rounded-full bg-slate-500 px-5"
        onpointerdown={handleFastForward}
    >
        >>
    </button>
{/if} -->
