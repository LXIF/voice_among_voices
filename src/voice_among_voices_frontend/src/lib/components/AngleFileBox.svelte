<script lang="ts">
    import {tick, createEventDispatcher} from 'svelte';
    import {backend} from '$lib/canisters';
    import {handleBackendAudioData} from '$lib/utils/convUtils';
    import type {
        HttpStreamingResponse,
        StreamingCallbackHttpResponse,
    } from '../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did';


    let { externalPlaybackPosition, onPlaybackPosition, onFileAngle, onFileLoaded }: { externalPlaybackPosition: number, onPlaybackPosition: (normalizedPosition: number) => void, onFileAngle: (angle: number) => void, onFileLoaded: (loaded: boolean) => void } = $props();

    let angle: number = $state(0);
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
            error = '';
            audioURL = '';

            const response: HttpStreamingResponse =
                await backend.get_angle_file(Math.round(angle));

            if (!response.streaming_strategy) {
                throw new Error('No streaming strategy provided.');
            }
            const chunks = [response.body];

            let streamingToken = response.streaming_strategy[0]?.Callback.token;

            while (streamingToken) {
                const {body, token} =

                    await backend.http_request_streaming_callback(
                        streamingToken
                    );
                chunks.push(body);
                streamingToken = token[0] || undefined;
            }

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
    $effect(() => { if (
        audioElement &&
        audioElement.duration &&
        externalPlaybackPosition >= 0 &&
        externalPlaybackPosition <= 1
    ) {
        audioElement.currentTime =
            externalPlaybackPosition * audioElement.duration;
    }});

    // Listen for playback end
    function onEnded() {
        isPlaying = false;
    }

    // // Toggle play/pause button state
    // $: isPlaying = !audioElement?.paused;
</script>

<div class="container">
    <div>
        <label for="angle-input">Enter an angle (0 - 359):</label>
        <input
            type="number"
            id="angle-input"
            bind:value={angle}
            min="0"
            max="359"
            class="angle-input"
        />
    </div>

    <button onclick={fetchAudioFile}>Request Audio File</button>

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
