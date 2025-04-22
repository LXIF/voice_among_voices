<script lang="ts">
    import { angleToRadians } from "$lib/utils/convUtils";
    import type { VoiceNodeEgress } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import { usableCanvasWidth, usableCanvasHeight } from "$lib/config/nodeMap";
    import {
        mapRotation,
        selectedAngle,
        playheadPosition,
        loadingProgress,
        loadingFile,
        myTokens,
    } from "$lib/state/uxState";
    import { identityAgent } from "$lib/canisters";
    import { blur } from "svelte/transition";
    import { isDarkMode } from "$lib/utils/uxUtils";
    import { untrack } from "svelte";
    import { isTouch } from "$lib/state/isMobile";

    let {
        class: classes = "",
        loading,
        onSelectAngle,
        onHoverAngle,
        loggedIn,
    }: {
        nodes: VoiceNodeEgress[];
        class?: string;
        loading: boolean;
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
                rotateToClosest($myTokens);
            });
        }
    });

    const handleSelectAngle = (angle: number) => {
        //TODO: not that great selection function
        rotateTo(angle);
        onSelectAngle(angle); // Still send the original angle (0-359) to the callback
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
                    duration: 0,
                    easing: undefined,
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
        let closestAngle = angle + (180 % 360);
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

        const angleDifference = Math.abs(targetAngle - currentMapRotation);

        // Store the promise from the set operation and return it
        const rotationPromise = mapRotation.set(-targetAngle, {
            duration: 200 + angleDifference * 10,
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

    function hsvToRgb(h: number, s: number, v: number): string {
        s = s / 100;
        v = v / 100;
        const i = Math.floor(h / 60);
        const f = h / 60 - i;
        const p = v * (1 - s);
        const q = v * (1 - f * s);
        const t = v * (1 - (1 - f) * s);

        let r, g, b;
        switch (i % 6) {
            case 0:
                [r, g, b] = [v, t, p];
                break;
            case 1:
                [r, g, b] = [q, v, p];
                break;
            case 2:
                [r, g, b] = [p, v, t];
                break;
            case 3:
                [r, g, b] = [p, q, v];
                break;
            case 4:
                [r, g, b] = [t, p, v];
                break;
            case 5:
                [r, g, b] = [v, p, q];
                break;
            default:
                [r, g, b] = [0, 0, 0];
        }

        return `rgb(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`;
    }

    function isAngleAvailable(angle: number): boolean {
        return $myTokens.includes(angle);
    }

    // Add a rotating offset for the loading animation
    let rotatingOffset = $state(0);
    let animationFrameId: number;

    // Add state for pulsing animation
    let pulseOffset = $state(0);

    // Update the animation
    $effect(() => {
        if (loading) {
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
        if (!loading) return 1;
        // Create a heartbeat-like effect with two quick pulses
        const t = pulseOffset;
        const pulse = Math.sin(t) * 0.5 + Math.sin(2 * t) * 0.25;
        return 1 + pulse * 0.05; // Scale the pulse effect (15% variation)
    }

    function getLineColor(angle: number, isAvailable: boolean): string {
        if (!loggedIn) {
            return hsvToRgb(angle, 100, 100);
        } else if (loading) {
            // During loading, rotate the hue and create a wave effect for saturation
            const adjustedAngle = (angle + rotatingOffset) % 360;
            // Create a wave effect based on the angle's position relative to the rotating offset
            const angleDiff = ((angle - rotatingOffset + 360) % 360) / 360;
            const pulsingSaturation =
                50 + Math.cos(angleDiff * 2 * Math.PI) * 50;
            const pulsingBrightness =
                60 + Math.sin(angleDiff * 2 * Math.PI) * 40;
            return hsvToRgb(
                adjustedAngle,
                pulsingSaturation,
                pulsingBrightness,
            );
        }
        // Normal state
        return isAvailable
            ? hsvToRgb(angle, 100, 100)
            : hsvToRgb(angle, 30, 20);
    }

    function handleCirclePointerDown(e: PointerEvent) {
        //prevent body scroll
        document.querySelector("body")?.classList.add("touch-none");
        document.querySelector("body")?.classList.add("overflow-hidden");
        e.preventDefault();
        e.stopPropagation();

        // Capture the starting point of the interaction
        const startX = e.clientX;
        const startY = e.clientY;
        let isDragging = false;
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
            Math.atan2(startY - centerY, startX - centerX) * (180 / Math.PI);

        // Handle pointer movement
        function handlePointerMove(moveEvent: PointerEvent) {
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
                duration: 0,
            });
            lastAngle = angleDiff;

            // Update hoveredAngle to match current rotation
            const currentRotation = -mapRotation.current; // Convert from map rotation to angle
            const normalizedRotation = Math.round(
                ((currentRotation % 360) + 360) % 360,
            ); // Normalize to 0-359 and round
            hoveredAngle = normalizedRotation;
        }

        // Handle pointer up
        async function handlePointerUp(upEvent: PointerEvent) {
            document.querySelector("body")?.classList.remove("touch-none");
            document.querySelector("body")?.classList.remove("overflow-hidden");
            // Clean up event listeners
            window.removeEventListener("pointermove", handlePointerMove);
            window.removeEventListener("pointerup", handlePointerUp);

            const availableAngles = $myTokens;

            rotateToClosest(availableAngles);
        }

        // Add event listeners for move and up events
        window.addEventListener("pointermove", handlePointerMove);
        window.addEventListener("pointerup", handlePointerUp);

        // Prevent default behavior to avoid text selection, etc.
        e.preventDefault();
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
</script>

<svg
    width={usableCanvasWidth}
    height={usableCanvasHeight}
    viewBox="0 0 200 200"
    xmlns="http://www.w3.org/2000/svg"
    class={`${classes} pointer-events-none`}
>
    <!-- Loading progress arc -->
    {#if $loadingFile}
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
            pointer-events={!!$identityAgent ? "auto" : "none"}
            class={!!$identityAgent ? "cursor-grab active:cursor-grabbing" : ""}
        />
        <g
            style={`transform-origin: center; transform: rotate(${mapRotation.current + 180}deg);`}
        >
            <!-- Draw lines for all angles -->
            {#each Array.from({ length: 360 }, (_, i) => i + 1) as angle}
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
                            (hoveredAngle === angle ? 1.18 : 1.1) *
                            getPulseScale()}
                    y2={centerY -
                        Math.sin(adjustedAngleToRadians(angle)) *
                            radius *
                            (hoveredAngle === angle ? 1.18 : 1.1) *
                            getPulseScale()}
                    class={`transition-all duration-200 ease-in-out outline-none ${isAngleAvailable(angle) ? "" : !!$identityAgent ? "cursor-grab active:cursor-grabbing" : ""}`}
                    onmouseover={() =>
                        isAngleAvailable(angle) && (hoveredAngle = angle)}
                    onmouseleave={() => (hoveredAngle = null)}
                    onpointerdown={(e) => {
                        if (!$identityAgent) return;
                        if (isAngleAvailable(angle)) {
                            handleSelectAngle(angle);
                        } else {
                            handleCirclePointerDown(e);
                        }
                    }}
                    onfocus={() =>
                        isAngleAvailable(angle) && (hoveredAngle = angle)}
                    onblur={() => (hoveredAngle = null)}
                    onkeydown={(e) => {
                        if (
                            isAngleAvailable(angle) &&
                            (e.key === "Enter" || e.key === " ")
                        ) {
                            handleSelectAngle(angle);
                        }
                    }}
                    stroke={getLineColor(angle, isAngleAvailable(angle))}
                    stroke-width={hoveredAngle === angle ? 2.2 : 0.5}
                    pointer-events={!!$identityAgent && isAngleAvailable(angle)
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
        >
            <!-- Draw lines for all angles -->
            {#each Array.from({ length: 360 }, (_, i) => i + 1) as angle}
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
                            (hoveredAngle === angle ? 1.18 : 1.1) *
                            getPulseScale()}
                    y2={centerY -
                        Math.sin(adjustedAngleToRadians(angle)) *
                            radius *
                            (hoveredAngle === angle ? 1.18 : 1.1) *
                            getPulseScale()}
                    class={`transition-all duration-200 ease-in-out outline-none ${isAngleAvailable(angle) ? "" : !!$identityAgent ? "cursor-grab active:cursor-grabbing" : ""}`}
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
            r={radius * 1.05}
            stroke-width="15"
            fill="red"
            stroke="red"
            opacity="0"
            onpointerdown={handleCirclePointerDown}
            pointer-events={!!$identityAgent ? "auto" : "none"}
            class={!!$identityAgent ? "cursor-grab active:cursor-grabbing" : ""}
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
