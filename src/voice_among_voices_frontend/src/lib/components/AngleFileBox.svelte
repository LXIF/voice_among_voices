<script lang="ts">
    import {onMount, tick} from 'svelte';
    import {Principal} from '@dfinity/principal';
    import {backend} from '$lib/canisters'; // Import your backend canister
    import {handleBackendAudioData} from '$lib/utils/convUtils';

    let angle: number = 0;
    let audioURL: string = '';
    let error: string = '';

    let audioElement: HTMLAudioElement;
    let downloadLink: HTMLAnchorElement;

    async function fetchAudioFile() {
        if (angle < 0 || angle > 359) {
            error = 'Please input an angle between 0 and 359.';
            return;
        }

        try {
            // Clear previous results
            error = '';
            audioURL = '';

            // Fetch the audio data from the backend
            const response = await backend.get_angle_file(Math.round(angle));

            if (!response.streaming_strategy) {
                throw new Error('No streaming strategy provided.');
            }

            // Handle first chunk of data
            const chunks = [response.body];
            let streamingToken = response.streaming_strategy[0].Callback.token;

            console.log(streamingToken);

            // Loop through the streaming response until we receive all chunks
            while (streamingToken) {
                const {body, token} =
                    await backend.http_request_streaming_callback(
                        streamingToken
                    );
                chunks.push(body);
                if (token[0]) {
                    streamingToken = token[0];
                    continue;
                }
                streamingToken = undefined;
            }

            // combine all chunks into uint8array
            const audioData = new Uint8Array(
                chunks.reduce((acc, chunk) => acc + chunk.length, 0)
            );
            let offset = 0;
            for (const chunk of chunks) {
                audioData.set(chunk, offset);
                offset += chunk.length;
            }

            // Handle and convert the audio data to a playable URL
            audioURL = await handleBackendAudioData(audioData);

            await tick();
            // Update download link
            downloadLink.href = audioURL;
            downloadLink.download = `audio_angle_${angle}.wav`; //TODO: add history frame / generation to filename
        } catch (e) {
            error = 'Error fetching the audio file.';
            console.error(e);
        }
    }
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

    <button on:click={fetchAudioFile}>Request Audio File</button>

    {#if error}
        <p class="error">{error}</p>
    {/if}

    {#if audioURL}
        <div>
            <audio
                controls
                bind:this={audioElement}
            >
                <source
                    src={audioURL}
                    type="audio/wav"
                />
                Your browser does not support the audio element.
            </audio>
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
