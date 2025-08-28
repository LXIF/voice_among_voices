<script lang="ts">
    import { onMount, tick } from "svelte";
    import { backend } from "$lib/canisters";
    import { handleBackendAudioData } from "$lib/utils/convUtils";
    import type {
        HttpStreamingResponse,
        StreamingCallbackToken,
    } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import {
        loadingProgress,
        applicationState,
        voiceNodes,
        applicationStates,
        toastMessage,
        audioParameters,
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
    import { MultiBufferPlayer } from "$lib/custom-audio/multiBufferPlayer";

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

    let generating = $state(false);

    let totalLength;

    let player = $state<MultiBufferPlayer>();

    function setupAudio() {
        if (!$audioParameters) {
            console.log("No audio params yet, retrying");
            setTimeout(setupAudio, 300);
            return;
        }
        totalLength =
            ($audioParameters.total_length_ms / 1000) *
            $audioParameters.sample_rate;

        player = new MultiBufferPlayer();
        player.setOnTimeUpdate(onTimeUpdate);
        player.setOnEnded(onEnded);
    }

    onMount(setupAudio);

    async function newFetchAudioAndPlay() {
        if (!$audioParameters) throw "No audio parameters";
        if (!player) throw "No player";

        onPressPlay();

        generating = true;
        // fetch first chunk of raw PCM data
        const response: HttpStreamingResponse | null =
            $selectedAngle === 0
                ? await getZeroFile()
                : await getAngleFile($selectedAngle);

        if (!response || !response.body) throw "Invalid response";

        const responseText = new TextDecoder().decode(
            response.body instanceof Uint8Array
                ? response.body
                : new Uint8Array(response.body),
        );
        const chunkData = JSON.parse(responseText);
        console.log("got chunks!");
        const { left_channel, right_channel } = chunkData;

        const leftArray = new Float32Array(left_channel.length);
        const rightArray = new Float32Array(right_channel.length);

        for (let i = 0; i < left_channel.length; i++) {
            leftArray[i] = left_channel[i] / 32768.0;
            rightArray[i] = right_channel[i] / 32768.0;
        }

        player.appendPCMData(leftArray, 0);
        player.appendPCMData(rightArray, 1);

        player.play();
        generating = false;
        isPlaying = true;
        console.log("started playback!");

        fetchChunksAndAddToPlayer(response, player);
    }

    async function fetchChunksAndAddToPlayer(
        response: HttpStreamingResponse,
        player: MultiBufferPlayer,
    ) {
        if (!response.streaming_strategy) return;

        const streamingToken = response.streaming_strategy[0]?.Callback.token;
        const nTokens = streamingToken?.chunks;
        if (!nTokens) return;

        // Fetch all remaining chunks in parallel
        for (let i = 0; i < nTokens - 1; i++) {
            const chunkToken = {
                angle: streamingToken.angle!,
                auth_token: streamingToken.auth_token!,
                chunk_index: i,
                chunks: streamingToken.chunks!,
            };

            const result =
                await backend.http_request_streaming_callback(chunkToken);

            // Decode and add to buffer immediately when chunk arrives
            const responseText = new TextDecoder().decode(
                result.body instanceof Uint8Array
                    ? result.body
                    : new Uint8Array(result.body),
            );
            const chunkData = JSON.parse(responseText);
            const { left_channel, right_channel } = chunkData;
            console.log("got channels for chunk " + i);

            const leftArray = new Float32Array(left_channel.length);
            const rightArray = new Float32Array(right_channel.length);

            for (let i = 0; i < left_channel.length; i++) {
                leftArray[i] = left_channel[i] / 32768.0;
                rightArray[i] = right_channel[i] / 32768.0;
            }

            player.appendPCMData(leftArray, 0);
            player.appendPCMData(rightArray, 1);
        }
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
            // audioURL = '';
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
            // togglePlayPause();
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
            player?.pause();
            isPlaying = false;
            $applicationState = applicationStates.loggedInIdle;
        } else {
            try {
                player?.play();
                isPlaying = true;
            } catch (error) {
                console.error("Playback failed:", error);
                // If autoplay fails, we'll let the user manually trigger play
                isPlaying = false;
                $applicationState = applicationStates.loggedInIdle;
            }
        }
    }

    // Dispatch the current playback position (normalized)
    function onTimeUpdate(currentTime: number) {
        if (!$audioParameters) throw "no audio params";
        const playbackPosition = currentTime / $audioParameters.total_length_ms;
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
        <h1 class="w-min text-center text-lg font-bold lg:text-2xl">
            Generating...
        </h1>
    {:else if $applicationState.showLoadingAnimation || $applicationState.showFileLoadingLine}
        <h1 class="w-min text-center text-2xl font-bold">Loading...</h1>
    {:else if audioURL && !isPlaying}
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
            onclick={newFetchAudioAndPlay}>Load</Button
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

    <!-- {#if audioURL}
        <div>
            <audio
                bind:this={audioElement}
                ontimeupdate={onTimeUpdate}
                onended={onEnded}
                hidden
            >
                <source src={audioURL} type="audio/wav" />
                Your browser does not support the audio element.
            </audio>
            <Button class="z-10 w-min text-center text-lg "
                ><a bind:this={downloadLink} href={audioURL} download>
                    Download
                </a>
            </Button>
        </div>
    {/if} -->
</div>

<style>
    .error {
        color: red;
        font-weight: bold;
    }
</style>
