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

    let audioElement: HTMLAudioElement | undefined = $state();
    let downloadLink: HTMLAnchorElement | undefined = $state();

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
            loadingProgress.set(0, {
                duration: 0,
            });
            $applicationState = applicationStates.loadingFile;
            error = "";
            // audioURL = '';
            const response: HttpStreamingResponse | null =
                $selectedAngle === 0
                    ? await getZeroFile()
                    : await getAngleFile($selectedAngle);
            if (!response) {
                toastMessage.set("Error fetching the audio file.");
                throw new Error("No response provided.");
            }
            if (!response.streaming_strategy) {
                throw new Error("No streaming strategy provided.");
            }
            const chunks = [response.body];

            let streamingToken = response.streaming_strategy[0]?.Callback.token;
            const nTokens =
                response.streaming_strategy[0]?.Callback.token.chunks;

            if (nTokens === undefined) throw new Error("No tokens provided.");

            let currentlyDownloaded = 1 / nTokens;
            // First chunk is already loaded
            loadingProgress.target = currentlyDownloaded;

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
                        .then((result) => {
                            // Update progress after each chunk loads
                            currentlyDownloaded += 1 / nTokens;
                            loadingProgress.target = currentlyDownloaded;
                            return result;
                        }),
                );
            }

            // Wait for all chunks and sort them by index
            const chunkResults = await Promise.all(chunkPromises);
            chunkResults.sort(
                (a, b) => a.token[0]?.chunk_index! - b.token[0]?.chunk_index!,
            );

            // Add sorted chunks to the chunks array
            chunks.push(...chunkResults.map((result) => result.body));

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
            // setTimeout(() => {
            //     $applicationState = applicationStates.playingFile;
            // }, 750);
            onFileAngle($selectedAngle);
            onFileLoaded(true);
            togglePlayPause();
        } catch (e) {
            error = "Error fetching the audio file.";
            $toastMessage = "Error fetching the audio file.";
            console.error(e);
        }
    }

    // Toggle play/pause
    async function togglePlayPause() {
        if (isPlaying) {
            audioElement!.pause();
            isPlaying = false;
            $applicationState = applicationStates.loggedInIdle;
        } else {
            try {
                const playPromise = audioElement!.play();
                if (playPromise !== undefined) {
                    await playPromise;
                    isPlaying = true;
                    $applicationState = applicationStates.playingFile;
                } else {
                    throw "empty audio element play promise!";
                }
            } catch (error) {
                console.error("Playback failed:", error);
                // If autoplay fails, we'll let the user manually trigger play
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
    {#if $applicationState.showLoadingAnimation || $applicationState.showFileLoadingLine}
        <h1 class="w-min text-center text-2xl font-bold lg:text-5xl">
            Loading...
        </h1>
    {:else}
        <Button
            class="z-10 w-min text-center text-4xl font-bold md:text-4xl lg:text-5xl"
            onclick={fetchAudioFileOrPlayPause}
            >{isPlaying ? "Pause" : "Play"}</Button
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
