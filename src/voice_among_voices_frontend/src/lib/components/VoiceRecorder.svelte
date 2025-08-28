<script lang="ts">
    import { onDestroy, onMount } from "svelte";

    import { browser } from "$app/environment";
    import { encodeWav } from "$lib/utils/convUtils";
    import type { AudioParameters } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import { fade, scale } from "svelte/transition";
    import { elasticOut } from "svelte/easing";
    import {
        applicationState,
        applicationStates,
        selectedAngle,
        toastMessage,
        sampleLength,
    } from "$lib/state/uxState";

    import { getVoiceNodes } from "$lib/icInteractions";
    import { getProgressArc } from "$lib/utils/uxUtils";

    // time the rec button needs to be held to be push-action
    const recordActionTimingCutoff = 200;

    let localStream: MediaStream | undefined = $state();
    let audioElement: HTMLAudioElement | undefined = $state();
    let mediaRecorder = $state<any>();
    let chunks: Blob[] = $state([]);
    let audioBlob: Blob | undefined = $state();
    // let register: any = $state();
    // let connect: any = $state();
    let recordingTimeout: ReturnType<typeof setTimeout> | undefined = $state();
    let encoderInitialized = $state(false);
    let elapsed = $state(0);

    let {
        audioParameters,
        voiceRecorded,
        recordingLength,
        class: classes,
    }: {
        audioParameters: AudioParameters | null;
        voiceRecorded: (blob: Blob) => void;
        recordingLength: (length: number) => void;
        class?: string;
    } = $props();

    onMount(async () => {
        if (browser) {
            const AudioRecorder = await import("audio-recorder-polyfill");
            window.MediaRecorder = AudioRecorder;

            handleActivateMicrophone();
        }
    });

    onDestroy(cleanup);

    function cleanup() {
        if (mediaRecorder && mediaRecorder.state !== "inactive") {
            mediaRecorder.stop();
        }
        if (localStream) {
            localStream.getTracks().forEach((track) => track.stop());
        }
        $applicationState = applicationStates.loggedInIdle;
    }

    function processAudioBlob(blob: Blob, trimLengthMs: number) {
        const fileReader = new FileReader();
        let trimmedBlob: Blob;
        fileReader.readAsArrayBuffer(blob);

        fileReader.onloadend = () => {
            const audioData = fileReader.result;

            if (!audioData || typeof audioData === "string") return;

            const audioContext = new AudioContext();

            audioContext.decodeAudioData(audioData).then((buffer) => {
                const targetDuration =
                    (trimLengthMs / 1000) * audioContext.sampleRate;
                const trimmedBuffer = audioContext.createBuffer(
                    1,
                    buffer.duration * audioContext.sampleRate > targetDuration
                        ? targetDuration
                        : buffer.duration * audioContext.sampleRate,
                    audioContext.sampleRate,
                );

                trimmedBuffer.copyToChannel(
                    buffer.getChannelData(0).slice(0, trimmedBuffer.length),
                    0,
                );

                trimmedBlob = encodeWav(trimmedBuffer);
                // checkAudioLength(trimmedBlob);

                if (!audioElement) throw "no audio element";
                const audioURL = window.URL.createObjectURL(trimmedBlob);
                audioElement.src = audioURL;

                voiceRecorded(trimmedBlob);
            });
        };
    }

    let audioDuration: number = 0;

    let progressPath = $derived(
        getProgressArc(
            40,
            40,
            34,
            0,
            !!audioParameters
                ? Math.min(elapsed / audioParameters.max_sample_length_ms, 1) *
                      360
                : 0,
        ),
    );

    $effect(() => recordingLength(audioDuration));

    function checkAudioLength(blob: Blob) {
        const fileReader = new FileReader();
        fileReader.readAsArrayBuffer(blob);

        fileReader.onloadend = () => {
            const audioData = fileReader.result;
            if (!audioData || typeof audioData === "string") return;

            const audioContext = new window.OfflineAudioContext({
                length: 44100 * 60,
                sampleRate: 44100,
                numberOfChannels: 1,
            });
            audioContext.decodeAudioData(
                audioData,
                (buffer) => {
                    audioDuration = buffer.duration * 1000;
                },
                (e) => {
                    console.error(e);
                },
            );
        };
    }

    let recordingStart: number;
    let recordingInterval: ReturnType<typeof setInterval>;

    async function handleRecordDown(e: PointerEvent) {
        e.preventDefault();
        if (!$applicationState.recorderActive) return;
        // if we're still recording, we used toggle action
        if ($applicationState.state === "recordingVoice") {
            handleRecordUp();
            return;
        }
        if (!audioParameters) throw "no parameters";

        handleActivateMicrophone();

        // reset the map
        $applicationState = applicationStates.recordingVoice;

        $applicationState = applicationStates.recordingVoice;
        window.addEventListener("pointerup", handleRecordUp);
        mediaRecorder?.start();

        recordingStart = Date.now();
        recordingInterval = setInterval(() => {
            elapsed = Date.now() - recordingStart;
            recordingLength(elapsed);
        }, 16);

        clearTimeout(recordingTimeout);
        recordingTimeout = setTimeout(
            handleStopRecording,
            audioParameters.max_sample_length_ms,
        );
    }

    function handleRecordUp() {
        // if it's just a tap, we use toggle.
        if (Date.now() - recordingStart < recordActionTimingCutoff) {
            return;
        }

        handleStopRecording();
    }

    function handleStopRecording() {
        // if it's just a tap, we use toggle.

        clearInterval(recordingInterval);
        clearTimeout(recordingTimeout);
        window.removeEventListener("pointerup", handleRecordUp);
        mediaRecorder?.stop();
        $applicationState = applicationStates.loggedInIdle;
        elapsed = 0;
    }

    function handleActivateMicrophone() {
        if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
            navigator.mediaDevices
                .getUserMedia({ audio: true })
                // .then((stream) => (localStream = stream))
                .then((stream) => setupMediaRecorder(stream))
                .catch((err) => {
                    $toastMessage = "Error activating microphone";
                    console.error(`getUserMedia hiccup: ${err}`);
                });
        } else {
            console.log("getUserMedia not supported on your browser!"); //TODO: handle better
        }
    }

    async function setupMediaRecorder(stream: MediaStream) {
        if (!browser || !audioParameters) {
            toastMessage.set("Error setting up media recorder");
            return;
        }

        if (encoderInitialized) {
            return;
        }

        const AudioRecorderModule = await import("audio-recorder-polyfill");
        const AudioRecorder = AudioRecorderModule.default;

        encoderInitialized = true;

        // Add audio context with limiter
        const audioContext = new AudioContext();
        const source = audioContext.createMediaStreamSource(stream);

        // Create a simple limiter using a WaveShaperNode
        const limiter = audioContext.createWaveShaper();

        // Limiter curve - prevents values above ~0.8 from going higher
        const curveLength = 44100;
        const curve = new Float32Array(curveLength);
        for (let i = 0; i < curveLength; i++) {
            const x = (i * 2) / curveLength - 1;
            curve[i] = Math.tanh(x * 2) * 0.8; // 0.8 = -2dB headroom
        }
        limiter.curve = curve;
        limiter.oversample = "4x";

        source.connect(limiter);

        // Create new stream from limited audio
        const dest = audioContext.createMediaStreamDestination();
        limiter.connect(dest);

        mediaRecorder = new AudioRecorder(dest.stream, {
            mimeType: "audio/wav",
        });

        mediaRecorder.addEventListener("dataavailable", (e: any) => {
            chunks.push(e.data);
        });
        mediaRecorder.addEventListener("stop", (e: any) => {
            audioBlob = new Blob(chunks, { type: "audio/wav" });
            chunks = [];
            processAudioBlob(audioBlob, audioParameters.max_sample_length_ms);
        });
    }
</script>

<audio hidden bind:this={audioElement}></audio>

<button
    onpointerdown={handleRecordDown}
    class={"pointer-events-auto relative h-20 max-h-20 min-h-20 w-20 min-w-20 max-w-20 cursor-pointer select-none rounded-full bg-red-600 text-2xl font-bold text-white transition-all disabled:cursor-wait disabled:bg-slate-500"}
    class:recording={$applicationState.state === "recordingVoice"}
    transition:scale={{
        duration: 500,
        easing: elasticOut,
    }}
    disabled={!$applicationState.recorderActive ||
        $selectedAngle < 1 ||
        $selectedAngle > 359}
    >Rec
    {#if elapsed > 0 && elapsed < 10000}
        <svg
            transition:fade
            height="80"
            width="80"
            class="pointer-events-none absolute inset-0"
        >
            <path
                d={progressPath}
                fill="none"
                stroke-width="5"
                stroke-linecap="round"
                class="stroke-slate-50 dark:stroke-slate-900"
            />
        </svg>
    {/if}
</button>

<style lang="postcss">
    .recording {
        @apply bg-slate-950 text-slate-50 dark:bg-slate-50 dark:text-slate-950;
    }
</style>
