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

export function blobToUint8Array(blob: Blob) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onloadend = function () {
            const arrayBuffer = reader.result;
            const uint8Array = new Uint8Array(arrayBuffer as ArrayBufferLike);
            console.log(uint8Array);
            resolve(uint8Array);
        }
        reader.onerror = reject;
        reader.readAsArrayBuffer(blob);
    });
}