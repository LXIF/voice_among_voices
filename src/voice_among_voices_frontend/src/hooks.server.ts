import { error } from "@sveltejs/kit";
import { angleToRadians, hsvToRgb } from "$lib/utils/convUtils";
import type { Handle } from "@sveltejs/kit";
import sharp from "sharp";
import type { VoiceNodeEgress } from "../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
import {
    canisterId,
    createActor,
    idlFactory,
} from "../../declarations/voice_among_voices_backend";
import { Actor, HttpAgent } from "@dfinity/agent";

export const handle: Handle = async ({ event, resolve }) => {
    const nftMatch = event.url.pathname.match(/^\/(\d+)$/);
    if (nftMatch) {
        const nftId = parseInt(nftMatch[1], 10);
        if (nftId < 1 || nftId > 360) {
            throw error(404, "Not Found");
        }
        // Create an agent for server-side
        const agent = await HttpAgent.create({
            host:
                process.env.DFX_NETWORK === "ic"
                    ? "https://ic0.app"
                    : "http://127.0.0.1:4943",
        });

        // Fetch root key in development mode
        if (process.env.DFX_NETWORK !== "ic") {
            await agent.fetchRootKey();
        }

        // Create the actor
        const actor = Actor.createActor(idlFactory, {
            agent,
            canisterId,
        });
        let nodesResponse = (await actor.get_voice_nodes()) as
            | { Ok: VoiceNodeEgress[] }
            | { Err: string };
        if ("Ok" in nodesResponse) {
            return await generatePng(nodesResponse.Ok, nftId);
        } else {
            throw error(404, "Not Found");
        }
    }

    return await resolve(event);
};

async function generatePng(nodes: VoiceNodeEgress[], nftId: number) {
    const centerX = 0;
    const centerY = 0;
    const radius = 45;
    const svg = `
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

    // Convert SVG to PNG
    const pngBuffer = await sharp(Buffer.from(svg)).png().toBuffer();

    return new Response(pngBuffer, {
        headers: {
            "Content-Type": "image/png",
            "Cache-Control": "public, max-age=360",
        },
    });
}

// Function to adjust angles: 0 is north, and positive angles go clockwise
function adjustedAngleToRadians(angle: number) {
    return angleToRadians(90 - angle); // Subtracting angle from 90 to shift 0 degrees to "north"
}

function getLineColor(angle: number): string {
    // Normal state
    return hsvToRgb(angle, 100, 100);
}

// ${Array.from({ length: 10 }, (_, i) => i)
// .map((angle) => {
//     return `<circle cx="0" cy="0" r="${angle * 5 + 5}" stroke-width="0.5" stroke="${getLineColor(angle)}" fill="none" />`;
// })
// .join(" ")}
