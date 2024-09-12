import type { ColliderCoordinate } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";

export function convertColliderCoordinatesToFloat32Array(coordinates: ColliderCoordinate[]): Float32Array {
    const flatCoordinates = coordinates.flatMap(coord => [coord.x, coord.y]);
    return new Float32Array(flatCoordinates);
}

export function mapToCanvasX(logicalX: number, logicalWidth: number, canvasWidth: number) {
    return (logicalX / logicalWidth) * canvasWidth;
}

export function mapToCanvasY(logicalY: number, logicalHeight: number, canvasHeight: number) {
    return (logicalY / logicalHeight) * canvasHeight;
}

// Converts canvas pixel coordinates to logical coordinates
export function canvasToLogical(x: number, y: number, canvasWidth: number, canvasHeight: number, logical_width: number, logical_height: number) {
    const logicalX = (x / canvasWidth) * logical_width;
    const logicalY = (y / canvasHeight) * logical_height;
    return {logicalX, logicalY};
}