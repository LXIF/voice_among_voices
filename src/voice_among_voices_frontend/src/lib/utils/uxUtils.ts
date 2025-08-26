import { angleToRadians } from "./convUtils";

export const isDarkMode = () => {
    //todo: this might better be a store with an eventlistened
    return (
        window.matchMedia &&
        window.matchMedia("(prefers-color-scheme: dark)").matches
    );
};

// Helper function to generate SVG arc path
export function getProgressArc(
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