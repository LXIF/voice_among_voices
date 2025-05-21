import { getVoiceNodes } from "$lib/icInteractions";
import { angleToRadians, hsvToRgb } from "$lib/utils/convUtils";
import type { Handle } from "@sveltejs/kit";
import sharp from "sharp";

export const handle: Handle = async ({ event, resolve }) => {
    // Ask a special endpoint within your app about the destination
    if (event.url.pathname.startsWith("/nft")) return await generatePng();

    return await resolve(event);
};

async function generatePng() {
    let nodes = await getVoiceNodes();
    // Create a new image with sharp

    // Adjusted scaling parameters for SVG
    const centerX = 0;
    const centerY = 0;
    const radius = 43; // Radius of the outer circle
    const svg = `
            <svg
                viewBox="-50 -50 100 100"
                width="500"
                height="500"
                xmlns="http://www.w3.org/2000/svg"
            >
                ${Array.from({ length: 360 }, (_, i) => i)
                    .map((angle) => {
                        return `<line
                        x1="${
                            centerX +
                            Math.cos(adjustedAngleToRadians(angle)) * radius
                        }"
                        y1="${
                            centerY -
                            Math.sin(adjustedAngleToRadians(angle)) * radius
                        }"
                        x2="${
                            centerX +
                            Math.cos(adjustedAngleToRadians(angle)) *
                                radius *
                                1.1
                        }"
                        y2="${
                            centerY -
                            Math.sin(adjustedAngleToRadians(angle)) *
                                radius *
                                1.1
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
                    cy="${node.y}"
                    r="${node.radius}"
                    stroke="hsl(${node.id}, 100%, 50%)"
                    stroke-width="0.5"
                />`;
                        })
                        .join(" ")}
            </svg>
        `;

    console.log(svg);
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
