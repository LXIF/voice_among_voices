<script lang="ts">
    import { onMount } from "svelte";
    import { angleToRadians, hsvToRgb } from "$lib/utils/convUtils";
    import type { VoiceNodeEgress } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import { getVoiceNodes } from "$lib/icInteractions";

    export let data;
    const nftId = parseInt(data.params.nftId, 10);

    onMount(async () => {
        const nodes = await getVoiceNodes();
        const svg = generateSvg(nodes, nftId);
        const img = document.getElementById("nft-image") as HTMLImageElement;
        if (img) {
            img.src = `data:image/svg+xml;base64,${btoa(svg)}`;
        }
    });

    function generateSvg(nodes: VoiceNodeEgress[], nftId: number) {
        const centerX = 0;
        const centerY = 0;
        const radius = 45;
        return `
            <svg
                viewBox="-70 -70 140 140"
                width="1000"
                height="1000"
                xmlns="http://www.w3.org/2000/svg"
            >
                ${Array.from({ length: 360 }, (_, i) => i)
                    .map((angle) => {
                        return `<line
                        x1="${
                            centerX -
                            Math.cos(adjustedAngleToRadians(angle)) *
                                radius *
                                (nftId === angle ? 0.3 : 1)
                        }"
                        y1="${
                            centerY +
                            Math.sin(adjustedAngleToRadians(angle)) *
                                radius *
                                (nftId === angle ? 0.3 : 1)
                        }"
                        x2="${
                            centerX -
                            Math.cos(adjustedAngleToRadians(angle)) *
                                radius *
                                (nftId === angle ? 1.4 : 1.1)
                        }"
                        y2="${
                            centerY +
                            Math.sin(adjustedAngleToRadians(angle)) *
                                radius *
                                (nftId === angle ? 1.4 : 1.1)
                        }"
                        stroke="${getLineColor(angle)}"
                        stroke-width="0.5"
                    />`;
                    })
                    .join(" ")}

                    ${nodes
                        .map((node) => {
                            return `<circle
                    cx="${node.x}"
                    cy="${-node.y}"
                    r="${node.radius}"
                    stroke="hsl(${node.id}, 100%, 50%)"
                    stroke-width="0.5"
                    fill="${Number(node.id) === nftId ? getLineColor(nftId) : "none"}"
                />
                ${
                    Number(node.id) === nftId
                        ? `
                <circle
                    cx="${node.x}"
                    cy="${-node.y}"
                    r="${node.radius + 1}"
                    stroke="hsl(${node.id}, 100%, 50%)"
                    stroke-width="0.5"
                    fill="none"
                    opacity="0.66"
                />
                <circle
                    cx="${node.x}"
                    cy="${-node.y}"
                    r="${node.radius + 2}"
                    stroke="hsl(${node.id}, 100%, 50%)"
                    stroke-width="0.5"
                    fill="none"
                    opacity="0.33"
                />
                `
                        : ""
                }
                `;
                        })
                        .join(" ")}
            </svg>
        `;
    }

    function adjustedAngleToRadians(angle: number) {
        return angleToRadians(90 - angle);
    }

    function getLineColor(angle: number): string {
        return hsvToRgb(angle, 100, 100);
    }
</script>

<img id="nft-image" alt="NFT Image" />
