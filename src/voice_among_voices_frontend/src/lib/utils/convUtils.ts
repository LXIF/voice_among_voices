import { get } from "svelte/store";
import type { ColliderCoordinate } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
import { audioParameters } from "$lib/state/uxState";

export function convertColliderCoordinatesToFloat32Array(
    coordinates: ColliderCoordinate[],
): Float32Array {
    const flatCoordinates = coordinates.flatMap((coord) => [coord.x, coord.y]);
    return new Float32Array(flatCoordinates);
}

export function mapToCanvasX(
    logicalX: number,
    logicalWidth: number,
    canvasWidth: number,
) {
    return (logicalX / logicalWidth) * canvasWidth;
}

export function mapToCanvasY(
    logicalY: number,
    logicalHeight: number,
    canvasHeight: number,
) {
    return (logicalY / logicalHeight) * canvasHeight;
}

// Converts canvas pixel coordinates to logical coordinates
export function canvasToLogical(
    x: number,
    y: number,
    usableCanvasDiameter: number,
    margin: number,
    logical_radius: number,
) {
    const logicalX =
        ((x - margin) / usableCanvasDiameter) * 2 * logical_radius -
        logical_radius;
    const logicalY = -(
        ((y - margin) / usableCanvasDiameter) * 2 * logical_radius -
        logical_radius
    );
    return { logicalX, logicalY };
}

export function blobToUint8Array(blob: Blob): Promise<Uint8Array> {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onloadend = function () {
            const arrayBuffer = reader.result as ArrayBuffer;
            const uint8Array = new Uint8Array(arrayBuffer);
            resolve(uint8Array);
        };
        reader.onerror = reject;
        reader.readAsArrayBuffer(blob);
    });
}

export function encodeWav(audioBuffer: AudioBuffer) {
    const audioParams = get(audioParameters);
    if(!audioParams) throw "No audio params";
    const numberOfChannels = audioBuffer.numberOfChannels;
    const sampleRate = audioParams.sample_rate;
    const numSamples = audioBuffer.length;

    // Create an empty buffer to hold the WAV file data
    const wavBuffer = new ArrayBuffer(44 + numSamples * 2); // 16-bit PCM
    const view = new DataView(wavBuffer);

    // RIFF chunk descriptor
    writeString(view, 0, "RIFF"); // ChunkID
    view.setUint32(4, 36 + numSamples * 2, true); // ChunkSize
    writeString(view, 8, "WAVE"); // Format

    // fmt sub-chunk (Format chunk)
    writeString(view, 12, "fmt "); // Subchunk1ID
    view.setUint32(16, 16, true); // Subchunk1Size (16 for PCM)
    view.setUint16(20, 1, true); // AudioFormat (1 for PCM)
    view.setUint16(22, numberOfChannels, true); // NumChannels
    view.setUint32(24, sampleRate, true); // SampleRate
    view.setUint32(28, sampleRate * numberOfChannels * 2, true); // ByteRate
    view.setUint16(32, numberOfChannels * 2, true); // BlockAlign
    view.setUint16(34, 16, true); // BitsPerSample

    // data sub-chunk
    writeString(view, 36, "data"); // Subchunk2ID
    view.setUint32(40, numSamples * 2, true); // Subchunk2Size

    // Write PCM samples
    const channelData = audioBuffer.getChannelData(0); // Assuming mono audio
    let offset = 44; // Start of data section
    for (let i = 0; i < numSamples; i++) {
        // Clamp the samples to the valid range (-1 to 1) and scale to 16-bit PCM
        const sample = Math.max(-1, Math.min(1, channelData[i]));
        view.setInt16(
            offset,
            sample < 0 ? sample * 0x8000 : sample * 0x7fff,
            true,
        );
        offset += 2;
    }

    return new Blob([view], { type: "audio/wav" });
}

function writeString(view: DataView, offset: number, string: string) {
    for (let i = 0; i < string.length; i++) {
        view.setUint8(offset + i, string.charCodeAt(i));
    }
}

export function handleBackendAudioData(audioData: Uint8Array): Promise<string> {
    return new Promise((resolve, reject) => {
        // Create audio context to ensure proper stereo handling
        const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
        const audioParams = get(audioParameters);
        if(!audioParams) throw "No audio params";
        
        // Decode the audio data
        audioContext.decodeAudioData(audioData.buffer as ArrayBuffer)
            .then(audioBuffer => {
                // Create a new audio buffer to ensure stereo
                const stereoBuffer = audioContext.createBuffer(
                    2, // Force 2 channels
                    audioBuffer.length,
                    audioParams.sample_rate
                );
                
                // Copy left channel
                const leftChannel = stereoBuffer.getChannelData(0);
                leftChannel.set(audioBuffer.getChannelData(0));
                
                // Copy right channel if it exists, otherwise duplicate left
                const rightChannel = stereoBuffer.getChannelData(1);
                if (audioBuffer.numberOfChannels > 1) {
                    rightChannel.set(audioBuffer.getChannelData(1));
                } else {
                    rightChannel.set(audioBuffer.getChannelData(0));
                }
                
                // Convert back to WAV
                const wavBlob = encodeWav(stereoBuffer);
                const audioURL = window.URL.createObjectURL(wavBlob);
                resolve(audioURL);
            })
            .catch(reject);
    });
}

export function angleToRadians(angle: number): number {
    return (angle * Math.PI) / 180;
}

export function abbreviateWalletAddress(address: string) {
    return (
        address.slice(0, 6) +
        "..." +
        address.slice(address.length - 4, address.length)
    );
}
