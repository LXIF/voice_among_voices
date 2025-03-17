<script lang="ts">
    import {tick} from 'svelte';
    import {backend} from '$lib/canisters';
    import {handleBackendAudioData} from '$lib/utils/convUtils';
    import type {
        HttpStreamingResponse,
    } from '../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did';
    import { loadingProgress, loadingFile } from '$lib/state/uxState.svelte';


    let {
        externalPlaybackPosition,
        onPlaybackPosition,
        onFileAngle,
        onFileLoaded,
        angle
    }: {
        externalPlaybackPosition: number,
        onPlaybackPosition: (normalizedPosition: number) => void,
        onFileAngle: (angle: number) => void,
        onFileLoaded: (loaded: boolean) => void,
        angle: number
    } = $props();

    let audioURL: string = $state('');
    let error: string = $state('');
    let isPlaying = $state(false); // To track play/pause state

    let audioElement: HTMLAudioElement | undefined = $state();
    let downloadLink: HTMLAnchorElement | undefined = $state();
 
    // Fetch audio file based on angle
    async function fetchAudioFile() {
        if (angle < 0 || angle > 359) {
            error = 'Please input an angle between 0 and 359.';
            return;
        }

        try {
            $loadingFile = true;
            error = '';
            audioURL = '';
            const response: HttpStreamingResponse = angle === 0 ? await backend.get_zero_file() :
                await backend.get_angle_file(BigInt(Math.round(angle)));
            if (!response.streaming_strategy) {
                throw new Error('No streaming strategy provided.');
            }
            const chunks = [response.body];

            let streamingToken = response.streaming_strategy[0]?.Callback.token;
            const nTokens = response.streaming_strategy[0]?.Callback.token.chunks;

            // while (streamingToken) {
            //     const {body, token} =
            //         await backend.http_request_streaming_callback(
            //             streamingToken
            //         );
            //         console.log(token);
            //     chunks.push(body);
            //     streamingToken = token[0] || undefined;
            // }

            if (nTokens === undefined) throw new Error('No tokens provided.');

            // First chunk is already loaded
            loadingProgress.target = 1 / nTokens;

            // Fetch all remaining chunks in parallel
            const chunkPromises = [];
            for (let i = 0; i < nTokens - 1; i++) {
                const chunkToken = {
                    angle: streamingToken?.angle!,
                    auth_token: streamingToken?.auth_token!,
                    chunk_index: i,
                    chunks: streamingToken?.chunks!
                };
                chunkPromises.push(
                    backend.http_request_streaming_callback(chunkToken)
                    .then(result => {
                            // Update progress after each chunk loads
                            loadingProgress.target = (loadingProgress.current + (1 / nTokens));
                            return result;
                        })
                );
            }

            // Wait for all chunks and sort them by index
            const chunkResults = await Promise.all(chunkPromises);
            chunkResults.sort((a, b) => a.token[0]?.chunk_index! - b.token[0]?.chunk_index!);
            
            // Add sorted chunks to the chunks array
            chunks.push(...chunkResults.map(result => result.body));

            const audioData = new Uint8Array(
                chunks.reduce((acc, chunk) => acc + chunk.length, 0)
            );

            let offset = 0;
            for (const chunk of chunks) {
                audioData.set(chunk, offset);
                offset += chunk.length;
            }

            audioURL = await handleBackendAudioData(audioData);
            await tick();
            downloadLink!.href = audioURL;
            downloadLink!.download = `audio_angle_${angle}.wav`;
            $loadingFile = false;
            onFileAngle(angle);
            onFileLoaded(true);
        } catch (e) {
            error = 'Error fetching the audio file.';
            console.error(e);
        }
    }

    // Toggle play/pause
    function togglePlayPause() {
        if (isPlaying) {
            audioElement!.pause();
            isPlaying = false;
        } else {
            audioElement!.play();
            isPlaying = true;
        }
    }

    // Dispatch the current playback position (normalized)
    function onTimeUpdate() {
        const playbackPosition =
            audioElement!.currentTime / audioElement!.duration;
        onPlaybackPosition(playbackPosition);
    }

    // Set playback position externally (in response to incoming props)
    $effect(() => {
        console.log(externalPlaybackPosition);
        console.log(audioElement);
        if (
            audioElement &&
            audioElement.duration &&
            externalPlaybackPosition >= 0 &&
            externalPlaybackPosition <= 1
        ) {
            audioElement.currentTime =
                externalPlaybackPosition * audioElement.duration;
        }
    });

    // Listen for playback end
    function onEnded() {
        isPlaying = false;
    }

    // // Toggle play/pause button state
    // $: isPlaying = !audioElement?.paused;
</script>

<div class="container">
    <button onclick={fetchAudioFile}>Generate file for angle {angle}</button>

    {#if error}
        <p class="error">{error}</p>
    {/if}

    {#if audioURL}
        <div>
            <!-- Custom Play/Pause Button -->
            <button onclick={togglePlayPause}>
                {isPlaying ? 'Pause' : 'Play'}
            </button>

            <!-- Hidden audio element (no controls) -->
            <audio
                bind:this={audioElement}
                ontimeupdate={onTimeUpdate}
                onended={onEnded}
            >
                <source
                    src={audioURL}
                    type="audio/wav"
                />
                Your browser does not support the audio element.
            </audio>

            <!-- Download link for the audio -->
            <a
                bind:this={downloadLink}
                href={audioURL}
                download
            >
                Download Audio
            </a>
        </div>
    {/if}
</div>

<style>
    .container {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .angle-input {
        width: 100px;
        padding: 0.5rem;
    }

    .error {
        color: red;
        font-weight: bold;
    }
</style>
