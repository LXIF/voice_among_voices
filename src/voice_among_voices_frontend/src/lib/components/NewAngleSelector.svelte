<script lang="ts">
    import { angleToRadians } from "$lib/utils/convUtils";
    import type { VoiceNodeEgress } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import {
        mapRotation,
        selectedAngle,
        playheadPosition,
        loadingProgress,
        myTokens,
        applicationState,
        buyTag,
    } from "$lib/state/uxState";
    import { identityAgent } from "$lib/canisters";
    import { blur } from "svelte/transition";
    import { isDarkMode } from "$lib/utils/uxUtils";
    import { untrack } from "svelte";
    import { isTouch } from "$lib/state/isMobile";
    import { PUBLIC_OPENSEA_URL } from "$lib/config/public";

    let {
        class: classes = "",
        onSelectAngle,
        onHoverAngle,
        loggedIn,
    }: {
        nodes: VoiceNodeEgress[];
        class?: string;
        onSelectAngle: (angle: number) => void;
        onHoverAngle: (angle: number | null) => void;
        loggedIn: boolean;
    } = $props();

    let hoveredAngle: number | null = $state(null);

    $effect(() => {
        onHoverAngle(hoveredAngle);
    });

    $effect(() => {
        if ($identityAgent && $myTokens.length > 0) {
            untrack(() => {
                rotateToClosest([...$myTokens, 0]);
            });
        }
    });

    const handleSelectAngle = (angle: number) => {
        rotateTo(angle);
        onSelectAngle(angle);
    };

    const rotateToClosest = (availableAngles: number[]) => {
        // Find the nearest available angle based on the current rotation
        const currentRotation = -mapRotation.current; // Convert from map rotation to angle
        const normalizedRotation = ((currentRotation % 360) + 360) % 360; // Normalize to 0-359

        if (availableAngles.length > 0) {
            // Find the angle in availableAngles that is closest to the current rotation
            let { closestAngle, closestDistance } = findClosestAngle(
                normalizedRotation,
                availableAngles,
            );
            hoveredAngle = closestAngle;

            // Rotate to the nearest available angle and wait for animation to complete
            const rotationPromise = rotateTo(currentRotation + closestDistance);
            rotationPromise.then(() => {
                mapRotation.set(-closestAngle, {
                    instant: true,
                });
                // Set a timeout to clear hoveredAngle 500ms after animation completes
                setTimeout(() => {
                    hoveredAngle = null;
                }, 500);
            });

            onSelectAngle(closestAngle);
        }
    };

    const findClosestAngle = (angle: number, availableAngles: number[]) => {
        let closestAngle = Math.round(angle + (180 % 360));
        let closestDistance = 180;
        availableAngles.forEach((availableAngle) => {
            // closest angles don't cross 0
            let distanceWithoutCrossing = availableAngle - angle;
            // closest angles cross 0 forwards
            let distanceCrossingForwards = availableAngle - angle - 360;
            // closest angles cross 0 backwards
            let distanceCrossingBackwards = availableAngle - angle + 360;

            if (Math.abs(distanceWithoutCrossing) < Math.abs(closestDistance)) {
                closestAngle = availableAngle;
                closestDistance = distanceWithoutCrossing;
            }
            if (
                Math.abs(distanceCrossingForwards) < Math.abs(closestDistance)
            ) {
                closestAngle = availableAngle;
                closestDistance = distanceCrossingForwards;
            }
            if (
                Math.abs(distanceCrossingBackwards) < Math.abs(closestDistance)
            ) {
                closestAngle = availableAngle;
                closestDistance = distanceCrossingBackwards;
            }
        });

        return { closestAngle, closestDistance };
    };

    const rotateTo = (angle: number) => {
        // We need to use the current map rotation as our starting point
        // instead of selectedAngle, especially after dragging
        const currentMapRotation = -mapRotation.current; // Convert from negative map rotation
        playheadPosition.target = 0;

        // Calculate the adjusted target angle for continuous rotation
        let targetAngle = angle;

        // Calculate the standard difference within 0-360 range
        const standardDiff = Math.abs(currentMapRotation - angle);

        // If the standard difference is more than 180 degrees, take the shorter path
        if (standardDiff > 180) {
            // If going clockwise across the 0° boundary
            if (currentMapRotation > 180 && angle < 180) {
                targetAngle = angle + 360; // Add 360 to make it continue past 360
            }
            // If going counterclockwise across the 0° boundary
            else if (currentMapRotation < 180 && angle > 180) {
                targetAngle = angle - 360; // Subtract 360 to make it go below 0
            }
        }

        // Store the promise from the set operation and return it
        const rotationPromise = mapRotation.set(-targetAngle, {
            preserveMomentum: 1,
        });

        // Update the selected angle state
        selectedAngle.set(angle);

        return rotationPromise;
    };

    // Adjusted scaling parameters for SVG
    const centerX = 100;
    const centerY = 100;
    const radius = 86; // Radius of the outer circle

    // Function to adjust angles: 0 is north, and positive angles go clockwise
    function adjustedAngleToRadians(angle: number) {
        return angleToRadians(90 - angle); // Subtracting angle from 90 to shift 0 degrees to "north"
    }

    function isAngleAvailable(angle: number): boolean {
        return angle === 0 || $myTokens.includes(angle);
    }

    // Add a rotating offset for the loading animation
    let rotatingOffset = $state(0);
    let animationFrameId: number;

    // Add state for pulsing animation
    let pulseOffset = $state(0);

    // Update the animation
    $effect(() => {
        if ($applicationState.showLoadingAnimation) {
            let lastTime = performance.now();

            function animate(currentTime: number) {
                const deltaTime = currentTime - lastTime;
                // Rotate the colors
                rotatingOffset = (rotatingOffset + deltaTime * 0.167) % 360;
                // Pulse the radius (slower than the rotation)
                pulseOffset = (pulseOffset + deltaTime * 0.004) % (Math.PI * 2);
                lastTime = currentTime;
                animationFrameId = requestAnimationFrame(animate);
            }

            animationFrameId = requestAnimationFrame(animate);

            return () => {
                if (animationFrameId) {
                    cancelAnimationFrame(animationFrameId);
                }
            };
        }
    });

    // Helper function to get the current pulse scale
    function getPulseScale(): number {
        if (!$applicationState.showLoadingAnimation) return 1;
        // Create a heartbeat-like effect with two quick pulses
        const t = pulseOffset;
        const pulse = Math.sin(t) * 0.5 + Math.sin(2 * t) * 0.25;
        return 1 + pulse * 0.05; // Scale the pulse effect (15% variation)
    }

    function getLineColor(angle: number, isAvailable: boolean): string {
        if (!loggedIn) {
            return `hsl(${angle}, 100%, 49%)`;
        } else if ($applicationState.showLoadingAnimation) {
            // During loading, rotate the hue and create a wave effect for saturation
            const adjustedAngle = (angle + rotatingOffset) % 360;
            // Create a wave effect based on the angle's position relative to the rotating offset
            const angleDiff = ((angle - rotatingOffset + 360) % 360) / 360;
            const pulsingSaturation =
                50 + Math.cos(angleDiff * 2 * Math.PI) * 50;
            const pulsingBrightness =
                60 + Math.sin(angleDiff * 2 * Math.PI) * 40;
            return `hsla(${adjustedAngle}, ${pulsingSaturation}%, 49%, ${pulsingBrightness}%)`;
        }
        // Normal state
        return isAvailable
            ? `hsla(${angle}, 100%, 49%, 100%)`
            : `hsla(${angle}, 70%, 49%, 20%)`;
    }

    function handleCirclePointerDown(e: PointerEvent) {
        //prevent body scroll
        e.preventDefault();
        e.stopPropagation();

        // !!$identityAgent && $applicationState.wheelActive
        //         ? "auto"
        //         : "none"
        if (!$identityAgent) {
            openBuyPage(0);
            return;
        }

        if (!!$identityAgent && $applicationState.wheelActive) {
            document
                .querySelector("body")
                ?.classList.add(
                    "touch-none",
                    "overflow-hidden",
                    "overflow-x-hidden",
                    "overscroll-none",
                    "scrolling-auto",
                    "fixed",
                    "w-full",
                    "h-full",
                );
            document
                .querySelector("html")
                ?.classList.add(
                    "touch-none",
                    "overflow-hidden",
                    "overflow-x-hidden",
                    "overscroll-none",
                    "scrolling-auto",
                    "fixed",
                    "w-full",
                    "h-full",
                );

            // Capture the starting point of the interaction
            const startX = e.clientX;
            const startY = e.clientY;
            // let isDragging = false;
            let lastAngle = 0;

            // Calculate center of the circle in client coordinates
            const svgRect = (
                e.currentTarget as SVGElement
            ).ownerSVGElement?.getBoundingClientRect();
            if (!svgRect) return;

            const centerX = svgRect.left + svgRect.width / 2;
            const centerY = svgRect.top + svgRect.height / 2;

            // Calculate starting angle
            const initialAngle =
                Math.atan2(startY - centerY, startX - centerX) *
                (180 / Math.PI);

            // Handle pointer movement
            function handlePointerMove(moveEvent: PointerEvent) {
                moveEvent.preventDefault();
                moveEvent.stopPropagation();
                window.scrollTo(0, 0);
                // Calculate current angle
                const currentAngle =
                    Math.atan2(
                        moveEvent.clientY - centerY,
                        moveEvent.clientX - centerX,
                    ) *
                    (180 / Math.PI);

                // Calculate the angle difference
                let angleDiff = currentAngle - initialAngle;

                // Update the map rotation based on the angle difference
                mapRotation.set(mapRotation.current + angleDiff - lastAngle, {
                    instant: true,
                });
                lastAngle = angleDiff;

                // Update hoveredAngle to match current rotation
                const currentRotation = -mapRotation.current; // Convert from map rotation to angle
                const normalizedRotation =
                    Math.round(((currentRotation % 360) + 360) % 360) % 360; // Normalize to 0-359 and round
                hoveredAngle = normalizedRotation;
            }

            // Handle pointer up
            async function handlePointerUp(_upEvent: PointerEvent) {
                document
                    .querySelector("body")
                    ?.classList.remove(
                        "touch-none",
                        "overflow-hidden",
                        "overflow-x-hidden",
                        "overscroll-none",
                        "scrolling-auto",
                        "fixed",
                        "w-full",
                        "h-full",
                    );
                document
                    .querySelector("html")
                    ?.classList.remove(
                        "touch-none",
                        "overflow-hidden",
                        "overflow-x-hidden",
                        "overscroll-none",
                        "scrolling-auto",
                        "fixed",
                        "w-full",
                        "h-full",
                    );
                // Clean up event listeners
                window.removeEventListener("pointermove", handlePointerMove);
                window.removeEventListener("pointerup", handlePointerUp);

                const availableAngles = [...$myTokens, 0];

                rotateToClosest(availableAngles);
            }

            // Add event listeners for move and up events
            window.addEventListener("pointermove", handlePointerMove);
            window.addEventListener("pointerup", handlePointerUp);
        }
    }

    // Helper function to generate SVG arc path
    function getProgressArc(
        cx: number,
        cy: number,
        r: number,
        startAngle: number,
        endAngle: number,
    ): string {
        // Convert angles from degrees to radians and adjust for SVG coordinates
        const start = angleToRadians(270 - startAngle);
        const end = angleToRadians(270 - endAngle);

        // Calculate start and end points
        const startX = cx + r * Math.cos(start);
        const startY = cy - r * Math.sin(start);
        const endX = cx + r * Math.cos(end);
        const endY = cy - r * Math.sin(end);

        // Determine if the arc should be drawn the long way around
        const largeArcFlag = endAngle - startAngle <= 180 ? "0" : "1";

        // Create the SVG arc path
        return `M ${startX} ${startY} A ${r} ${r} 0 ${largeArcFlag} 1 ${endX} ${endY}`;
    }

    function openBuyPage(angle: number) {
        if (angle === 0) {
            const url = PUBLIC_OPENSEA_URL;
            //this is for touch
            window.open(url, "_blank", "noopener,noreferrer");
            return;
        }

        const url = `${PUBLIC_OPENSEA_URL}/${angle}`;
        window.open(url, "_blank", "noopener,noreferrer");
    }

    function handlePositionBuy(e: Event, angle: number) {
        const rect = (e.target as Element).getBoundingClientRect();
        let x = rect.left + rect.width / 2;
        let y = rect.top + rect.height / 2;

        // Offset distance in pixels
        const offset = -100; // adjust as needed

        // Convert angle to radians, with 0° = up (north), positive clockwise
        const rad = ((90 - angle) * Math.PI) / 180;

        // Calculate offset
        x += Math.cos(rad) * offset;
        y -= Math.sin(rad) * offset;

        $buyTag = { x, y, angle };
    }
</script>

<svg
    viewBox="0 0 200 200"
    xmlns="http://www.w3.org/2000/svg"
    class={`${classes} pointer-events-none min-w-full touch-none`}
>
    <!-- Loading progress arc -->
    {#if $applicationState.showFileLoadingLine}
        <path
            d={getProgressArc(
                centerX,
                centerY,
                radius * 0.99,
                0,
                360 * (loadingProgress.current * 0.99999),
            )}
            stroke={isDarkMode() ? "white" : "black"}
            stroke-width="0.5"
            fill="none"
            opacity="1"
            transition:blur={{
                duration: 500,
            }}
            pointer-events="none"
            class="pointer-events-none"
        />
    {/if}
    <!-- with mouse, this is a fallback option -->
    {#if !$isTouch}
        <!-- ui element for drag-rotating -->
        <circle
            role="button"
            cx={centerX}
            cy={centerY}
            r={radius * 1.05}
            stroke-width="15"
            fill="none"
            stroke="red"
            opacity="0"
            onpointerdown={handleCirclePointerDown}
            pointer-events={!!$identityAgent && $applicationState.wheelActive
                ? "auto"
                : "none"}
            class={!!$identityAgent && $applicationState.wheelActive
                ? "z-20 cursor-grab active:cursor-grabbing"
                : "z-20"}
        />
        <g
            style={`transform-origin: center; transform: rotate(${mapRotation.current + 180}deg);`}
            class="z-20"
        >
            <!-- Draw lines for all angles -->
            {#each Array.from({ length: 360 }, (_, i) => i) as angle}
                <line
                    tabindex={isAngleAvailable(angle) ? 0 : -1}
                    role="button"
                    x1={centerX +
                        Math.cos(adjustedAngleToRadians(angle)) *
                            radius *
                            (hoveredAngle === angle ? 0.92 : 1) *
                            getPulseScale()}
                    y1={centerY -
                        Math.sin(adjustedAngleToRadians(angle)) *
                            radius *
                            (hoveredAngle === angle ? 0.92 : 1) *
                            getPulseScale()}
                    x2={centerX +
                        Math.cos(adjustedAngleToRadians(angle)) *
                            radius *
                            (hoveredAngle === angle ? 1.16 : 1.1) *
                            getPulseScale()}
                    y2={centerY -
                        Math.sin(adjustedAngleToRadians(angle)) *
                            radius *
                            (hoveredAngle === angle ? 1.16 : 1.1) *
                            getPulseScale()}
                    class={`z-20 outline-none transition-all duration-200 ease-in-out ${isAngleAvailable(angle) ? "" : !!$identityAgent && $applicationState.wheelActive ? "cursor-grab active:cursor-grabbing" : !$identityAgent ? "cursor-pointer" : "cursor-wait"}`}
                    onmouseover={(e) => {
                        if (
                            isAngleAvailable(angle) &&
                            $applicationState.wheelActive
                        ) {
                            hoveredAngle = angle;
                        } else if (!$identityAgent) {
                            hoveredAngle = angle;
                        }
                        handlePositionBuy(e, angle);
                    }}
                    onmouseleave={() => (hoveredAngle = null)}
                    onpointerdown={(e) => {
                        if (!$identityAgent || !$applicationState.wheelActive) {
                            openBuyPage(angle);
                            return;
                        }
                        if (isAngleAvailable(angle)) {
                            handleSelectAngle(angle);
                        } else {
                            handleCirclePointerDown(e);
                        }
                    }}
                    onfocus={(e) => {
                        if (
                            isAngleAvailable(angle) &&
                            $applicationState.wheelActive
                        ) {
                            hoveredAngle = angle;
                            handlePositionBuy(e, angle);
                        }
                    }}
                    onblur={() => (hoveredAngle = null)}
                    onkeydown={(e) => {
                        if (
                            isAngleAvailable(angle) &&
                            $applicationState.wheelActive &&
                            (e.key === "Enter" || e.key === " ")
                        ) {
                            handleSelectAngle(angle);
                        }
                    }}
                    stroke={getLineColor(angle, isAngleAvailable(angle))}
                    stroke-width={hoveredAngle === angle ? 2.2 : 0.5}
                    pointer-events={!!$identityAgent &&
                    isAngleAvailable(angle) &&
                    $applicationState.wheelActive
                        ? "auto"
                        : !$identityAgent
                          ? "auto"
                          : "none"}
                />
            {/each}
        </g>
    {/if}
    <!-- on touch devices, lines are only decorative and the circle is the main interaction -->
    {#if $isTouch}
        <g
            style={`transform-origin: center; transform: rotate(${mapRotation.current + 180}deg);`}
            class="z-20"
        >
            <!-- Draw lines for all angles -->
            {#each Array.from({ length: 360 }, (_, i) => i) as angle}
                <line
                    tabindex={isAngleAvailable(angle) ? 0 : -1}
                    role="button"
                    x1={centerX +
                        Math.cos(adjustedAngleToRadians(angle)) *
                            radius *
                            (hoveredAngle === angle ? 0.92 : 1) *
                            getPulseScale()}
                    y1={centerY -
                        Math.sin(adjustedAngleToRadians(angle)) *
                            radius *
                            (hoveredAngle === angle ? 0.92 : 1) *
                            getPulseScale()}
                    x2={centerX +
                        Math.cos(adjustedAngleToRadians(angle)) *
                            radius *
                            (hoveredAngle === angle ? 1.16 : 1.1) *
                            getPulseScale()}
                    y2={centerY -
                        Math.sin(adjustedAngleToRadians(angle)) *
                            radius *
                            (hoveredAngle === angle ? 1.16 : 1.1) *
                            getPulseScale()}
                    class={`z-20 outline-none transition-all duration-200 ease-in-out ${isAngleAvailable(angle) ? "" : !!$identityAgent && $applicationState.wheelActive ? "cursor-grab active:cursor-grabbing" : ""}`}
                    stroke={getLineColor(angle, isAngleAvailable(angle))}
                    stroke-width={hoveredAngle === angle ? 2.2 : 0.5}
                    pointer-events="none"
                />
            {/each}
        </g>

        <!-- ui element for drag-rotating -->
        <circle
            role="button"
            cx={centerX}
            cy={centerY}
            r={radius * 1.1}
            stroke-width="35"
            fill="none"
            stroke="red"
            opacity="0"
            onpointerdown={handleCirclePointerDown}
            pointer-events={"auto"}
            class={!!$identityAgent && $applicationState.wheelActive
                ? "z-20 cursor-grab active:cursor-grabbing"
                : "z-20 cursor-pointer"}
        />
    {/if}
</svg>

<style>
    .highlight {
        stroke: red; /* Highlight color */
        stroke-width: 2;
    }

    .locked-scroll {
        overflow: hidden;
        touch-action: none;
    }
</style>
