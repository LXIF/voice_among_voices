<script lang="ts">
    import { tick } from "svelte";
    import { backend } from "$lib/canisters";
    import { handleBackendAudioData } from "$lib/utils/convUtils";
    import type { HttpStreamingResponse } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import {
        loadingProgress,
        applicationState,
        backendSimulationResult,
        voiceNodes,
        applicationStates,
        toastMessage,
    } from "$lib/state/uxState";
    import {
        selectedAngle,
        externalPlaybackPosition,
    } from "$lib/state/uxState";
    import Button from "./Button.svelte";
    import {
        getAngleFile,
        getVoiceNodes,
        getZeroFile,
    } from "$lib/icInteractions";

    let {
        onPlaybackPosition,
        onFileAngle,
        onFileLoaded,
        onPressPlay,
    }: {
        onPlaybackPosition: (normalizedPosition: number) => void;
        onFileAngle: (angle: number) => void;
        onFileLoaded: (loaded: boolean) => void;
        onPressPlay: () => void;
    } = $props();

    let audioURL: string = $state("");
    let error: string = $state("");
    let isPlaying = $state(false); // To track play/pause state
    let isStreaming = $state(false); // Track if we're in streaming mode

    let audioElement: HTMLAudioElement | undefined = $state();
    let downloadLink: HTMLAnchorElement | undefined = $state();
    let streamingAudioContext: AudioContext | undefined = $state();
    let streamingSource: AudioBufferSourceNode | undefined = $state();
    let audioBuffer: AudioBuffer | undefined = $state();
    let isBufferComplete = $state(false);

    let generating = $state(false);
    let crossfadeDuration = 0.1; // 100ms crossfade
    let totalExpectedLength = 0; // Store total expected samples

    // Start streaming playback with first chunk
    async function startStreamingPlayback(firstChunk: Uint8Array) {
        try {
            streamingAudioContext = new AudioContext();

            // Create initial buffer from first chunk to get format info
            const firstChunkBuffer =
                await streamingAudioContext.decodeAudioData(
                    firstChunk.buffer.slice(
                        firstChunk.byteOffset,
                        firstChunk.byteOffset + firstChunk.byteLength,
                    ) as ArrayBuffer,
                );

            // Create full-size buffer filled with silence
            totalExpectedLength = firstChunkBuffer.length * nTokens; // Estimate total length
            audioBuffer = streamingAudioContext.createBuffer(
                firstChunkBuffer.numberOfChannels,
                totalExpectedLength,
                firstChunkBuffer.sampleRate,
            );

            // Fill with silence (0s)
            for (
                let channel = 0;
                channel < audioBuffer.numberOfChannels;
                channel++
            ) {
                const channelData = audioBuffer.getChannelData(channel);
                channelData.fill(0);
            }

            // Start playback
            startBufferPlayback();
        } catch (e) {
            console.error("Failed to start streaming playback:", e);
            isStreaming = false;
        }
    }

    // Start or restart buffer playback
    function startBufferPlayback() {
        if (!streamingAudioContext || !audioBuffer) return;

        streamingSource = streamingAudioContext.createBufferSource();
        streamingSource.buffer = audioBuffer;
        streamingSource.connect(streamingAudioContext.destination);
        streamingSource.start(0);

        isStreaming = true;
        isPlaying = true;
        $applicationState = applicationStates.playingFile;

        streamingSource.onended = () => {
            if (isStreaming && !isBufferComplete) {
                // Restart playback if buffer is still growing
                startBufferPlayback();
            }
        };
    }

    // Add new chunk to existing buffer at specific position
    async function addChunkToBuffer(newChunk: Uint8Array, chunkIndex: number) {
        if (!streamingAudioContext || !audioBuffer) return;

        try {
            // Decode new chunk
            const newChunkBuffer = await streamingAudioContext.decodeAudioData(
                newChunk.buffer.slice(
                    newChunk.byteOffset,
                    newChunk.byteOffset + newChunk.byteLength,
                ) as ArrayBuffer,
            );

            // Calculate position in the full buffer
            const chunkSize = newChunkBuffer.length;
            const startPosition = chunkIndex * chunkSize;

            // Copy chunk data to the correct position in the full buffer
            for (
                let channel = 0;
                channel < audioBuffer.numberOfChannels;
                channel++
            ) {
                const newData = newChunkBuffer.getChannelData(channel);
                const fullBufferData = audioBuffer.getChannelData(channel);

                newData.forEach((sample, index) => {
                    fullBufferData[startPosition + index] = sample;
                });
            }

            // No need to restart playback - buffer is already playing
            // The audio will automatically include the new data
        } catch (e) {
            console.error("Failed to add chunk to buffer:", e);
        }
    }

    // Stop streaming playback and switch to full file
    async function stopStreamingPlayback() {
        if (streamingSource && streamingAudioContext) {
            try {
                // Create a crossfade instead of immediate stop
                const gainNode = streamingAudioContext.createGain();
                streamingSource.disconnect();
                streamingSource.connect(gainNode);
                gainNode.connect(streamingAudioContext.destination);

                // Fade out over crossfade duration
                gainNode.gain.setValueAtTime(
                    1,
                    streamingAudioContext.currentTime,
                );
                gainNode.gain.linearRampToValueAtTime(
                    0,
                    streamingAudioContext.currentTime + crossfadeDuration,
                );

                // Stop after crossfade
                setTimeout(() => {
                    if (streamingSource) {
                        streamingSource.stop();
                        streamingSource.disconnect();
                    }
                    if (streamingAudioContext) {
                        streamingAudioContext.close();
                    }
                }, crossfadeDuration * 1000);
            } catch (e) {
                console.error("Error stopping streaming playback:", e);
            }
        }
        isStreaming = false;
        streamingSource = undefined;
        streamingAudioContext = undefined;
    }

    // Fetch audio file based on angle
    async function fetchAudioFileOrPlayPause() {
        onPressPlay();
        if (isPlaying) {
            togglePlayPause();
            return;
        } else if (audioURL) {
            togglePlayPause();
            return;
        }
        if ($selectedAngle < 0 || $selectedAngle > 359) {
            error = "Please input an angle between 0 and 359.";
            return;
        }

        // reset the map
        $applicationState = applicationStates.loadingFile;
        $voiceNodes = await getVoiceNodes();

        try {
            generating = true;
            loadingProgress.set(0, {
                duration: 0,
            });
            $applicationState = applicationStates.loadingFile;
            error = "";

            const response: HttpStreamingResponse | null =
                $selectedAngle === 0
                    ? await getZeroFile()
                    : await getAngleFile($selectedAngle);
            if (!response) {
                generating = false;
                toastMessage.set("Error fetching the audio file.");
                throw new Error("No response provided.");
            }
            if (!response.streaming_strategy) {
                generating = false;
                throw new Error("No streaming strategy provided.");
            }

            generating = false;
            const chunks = [response.body];

            let streamingToken = response.streaming_strategy[0]?.Callback.token;
            const nTokens =
                response.streaming_strategy[0]?.Callback.token.chunks;

            if (nTokens === undefined) throw new Error("No tokens provided.");

            let currentlyDownloaded = 1 / nTokens;
            // First chunk is already loaded
            loadingProgress.target = currentlyDownloaded;

            // Start streaming playback immediately with first chunk
            const firstChunk =
                response.body instanceof Uint8Array
                    ? response.body
                    : new Uint8Array(response.body);
            await startStreamingPlayback(firstChunk);

            // Fetch all remaining chunks in parallel
            const chunkPromises = [];
            for (let i = 0; i < nTokens - 1; i++) {
                const chunkToken = {
                    angle: streamingToken?.angle!,
                    auth_token: streamingToken?.auth_token!,
                    chunk_index: i,
                    chunks: streamingToken?.chunks!,
                };
                chunkPromises.push(
                    backend
                        .http_request_streaming_callback(chunkToken)
                        .then(async (result) => {
                            // Update progress after each chunk loads
                            currentlyDownloaded += 1 / nTokens;
                            loadingProgress.target = currentlyDownloaded;

                            // Add chunk to buffer at specific position
                            const chunk =
                                result.body instanceof Uint8Array
                                    ? result.body
                                    : new Uint8Array(result.body);
                            await addChunkToBuffer(chunk, i + 1); // +1 because first chunk is already loaded

                            return result;
                        }),
                );
            }

            // Wait for all chunks to complete
            await Promise.all(chunkPromises);

            // Mark buffer as complete
            isBufferComplete = true;

            // Stop streaming and switch to full file
            stopStreamingPlayback();

            // Small delay to allow crossfade to complete
            await new Promise((resolve) =>
                setTimeout(resolve, crossfadeDuration * 1000 + 50),
            );

            const audioData = new Uint8Array(
                chunks.reduce((acc, chunk) => acc + chunk.length, 0),
            );

            let offset = 0;
            for (const chunk of chunks) {
                audioData.set(chunk, offset);
                offset += chunk.length;
            }

            audioURL = await handleBackendAudioData(audioData);
            await tick();
            downloadLink!.href = audioURL;
            downloadLink!.download = `voice_among_voices_${$selectedAngle}°_${Date.now()}.wav`;

            onFileAngle($selectedAngle);
            onFileLoaded(true);
            $applicationState = applicationStates.loggedInIdle;
        } catch (e) {
            generating = false;
            error = "Error fetching the audio file.";
            $toastMessage = "Error fetching the audio file.";
            console.error(e);
        }
    }

    // Toggle play/pause
    async function togglePlayPause() {
        if (isPlaying) {
            if (isStreaming) {
                // Pause streaming playback
                if (streamingAudioContext) {
                    streamingAudioContext.suspend();
                }
            } else {
                // Pause regular audio element
                audioElement!.pause();
            }
            isPlaying = false;
            $applicationState = applicationStates.loggedInIdle;
        } else {
            try {
                if (isStreaming) {
                    // Resume streaming playback
                    if (streamingAudioContext) {
                        streamingAudioContext.resume();
                    }
                    isPlaying = true;
                    $applicationState = applicationStates.playingFile;
                } else {
                    // Resume regular audio element
                    const playPromise = audioElement!.play();
                    if (playPromise !== undefined) {
                        await playPromise;
                        isPlaying = true;
                        $applicationState = applicationStates.playingFile;
                    } else {
                        throw "empty audio element play promise!";
                    }
                }
            } catch (error) {
                console.error("Playback failed:", error);
                isPlaying = false;
                $applicationState = applicationStates.loggedInIdle;
            }
        }
    }

    // Dispatch the current playback position (normalized)
    function onTimeUpdate() {
        if (
            !audioElement ||
            !audioElement.duration ||
            !audioElement.currentTime
        )
            return;
        const playbackPosition =
            audioElement!.currentTime / audioElement!.duration;
        onPlaybackPosition(playbackPosition);
    }

    // Set playback position externally (in response to incoming props)
    $effect(() => {
        $externalPlaybackPosition;
        if (
            audioElement &&
            audioElement.duration &&
            $externalPlaybackPosition >= 0 &&
            $externalPlaybackPosition <= 1
        ) {
            audioElement.currentTime =
                $externalPlaybackPosition * audioElement.duration;
            onPlaybackPosition($externalPlaybackPosition);
        }
    });

    $effect(() => {
        if ($selectedAngle) {
            audioURL = "";
            isPlaying = false;
        }
    });

    // Listen for playback end
    function onEnded() {
        isPlaying = false;
    }

    // // Toggle play/pause button state
    // $: isPlaying = !audioElement?.paused;
</script>

<div class="flex w-full flex-col items-center gap-4">
    {#if generating}
        <h1 class="w-min text-center text-2xl font-bold">Generating...</h1>
    {:else if $applicationState.showLoadingAnimation || $applicationState.showFileLoadingLine}
        <h1 class="w-min text-center text-2xl font-bold">Loading...</h1>
    {:else if (audioURL || isStreaming) && !isPlaying}
        <Button
            class="z-10 w-min text-center text-4xl font-bold md:text-4xl lg:text-5xl"
            onclick={togglePlayPause}>Play</Button
        >
    {:else if isPlaying}
        <Button
            class="z-10 w-min text-center text-4xl font-bold md:text-4xl lg:text-5xl"
            onclick={togglePlayPause}>Pause</Button
        >
    {:else}
        <Button
            class="z-10 w-min text-center text-4xl font-bold md:text-4xl lg:text-5xl"
            onclick={fetchAudioFileOrPlayPause}>Load</Button
        >
    {/if}
    <h1
        style={`color: hsl(${$selectedAngle},100%,50%)`}
        class="pointer-events-none cursor-none select-none text-center text-5xl font-bold"
    >
        {$selectedAngle}°
    </h1>

    {#if error}
        <p class="error">{error}</p>
    {/if}

    {#if audioURL}
        <div>
            <!-- Hidden audio element (no controls) -->
            <audio
                bind:this={audioElement}
                ontimeupdate={onTimeUpdate}
                onended={onEnded}
                hidden
            >
                <source src={audioURL} type="audio/wav" />
                Your browser does not support the audio element.
            </audio>

            <!-- Download link for the audio -->
            <Button class="z-10 w-min text-center text-lg "
                ><a bind:this={downloadLink} href={audioURL} download>
                    Download
                </a>
            </Button>
        </div>
    {/if}
</div>

<style>
    .error {
        color: red;
        font-weight: bold;
    }
</style>
