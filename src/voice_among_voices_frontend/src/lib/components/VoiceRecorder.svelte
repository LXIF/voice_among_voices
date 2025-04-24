<script lang="ts">
    import { onDestroy, onMount } from "svelte";

    import { browser } from "$app/environment";
    import { encodeWav } from "$lib/utils/convUtils";
    import type { AudioParameters } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";

    // time the rec button needs to be held to be push-action
    const recordActionTimingCutoff = 200;

    let localStream: MediaStream | undefined = $state();
    let audioElement: HTMLAudioElement | undefined = $state();
    let mediaRecorder: any = $state();
    let recording = $state(false);
    let chunks: Blob[] = $state([]);
    let audioBlob: Blob | undefined = $state();
    let register: any = $state();
    let connect: any = $state();
    let recordingTimeout: ReturnType<typeof setTimeout> | undefined = $state();

    let {
        audioParameters,
        voiceRecorded,
        recordingLength,
        class: classes,
    }: {
        audioParameters: AudioParameters | undefined;
        voiceRecorded: (blob: Blob) => void;
        recordingLength: (length: number) => void;
        class?: string;
    } = $props();

    onMount(async () => {
        if (browser) {
            const {
                MediaRecorder: ImportedMediaRecorder,
                register: ImportedRegister,
            } = await import("extendable-media-recorder");

            // Dynamically import the WAV encoder
            const { connect: ImportedConnect } = await import(
                "extendable-media-recorder-wav-encoder"
            );

            // Store the imports in local variables
            // mediaRecorder = ImportedMediaRecorder;
            register = ImportedRegister;
            connect = ImportedConnect;

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
        recording = false;
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
                    console.log(buffer);
                },
                (e) => {
                    console.error(e);
                },
            );
        };
    }

    let recordingStart: number;
    let recordingInterval: ReturnType<typeof setInterval>;

    function handleRecordDown(e: PointerEvent) {
        e.preventDefault();
        // if we're still recording, we used toggle action
        if (recording) {
            handleRecordUp();
            return;
        }
        if (!audioParameters) throw "no parameters";

        handleActivateMicrophone();

        recording = true;
        window.addEventListener("pointerup", handleRecordUp);
        mediaRecorder?.start();

        recordingStart = Date.now();
        recordingInterval = setInterval(() => {
            const elapsed = Date.now() - recordingStart;
            recordingLength(elapsed);
        }, 16);

        clearTimeout(recordingTimeout);
        recordingTimeout = setTimeout(
            handleRecordUp,
            audioParameters.max_sample_length_ms,
        );
    }

    function handleRecordUp() {
        // if it's just a tap, we use toggle.
        if (Date.now() - recordingStart < recordActionTimingCutoff) {
            return;
        }

        clearInterval(recordingInterval);
        recording = false;
        window.removeEventListener("pointerup", handleRecordUp);
        mediaRecorder?.stop();
    }

    function handleActivateMicrophone() {
        if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
            navigator.mediaDevices
                .getUserMedia({ audio: true })
                .then((stream) => (localStream = stream))
                .then(setupMediaRecorder)
                .catch((err) => {
                    console.error(`getUserMedia hiccup: ${err}`);
                });
        } else {
            console.log("getUserMedia not supported on your browser!");
        }
    }

    async function setupMediaRecorder() {
        if (!browser || !audioParameters || !localStream) return;

        const { MediaRecorder: ImportedMediaRecorder } = await import(
            "extendable-media-recorder"
        );

        await register(await connect());

        const audioContext = new AudioContext({ sampleRate: 44100 });
        const mediaStreamAudioSourceNode = new MediaStreamAudioSourceNode(
            audioContext,
            {
                mediaStream: localStream,
            },
        );
        const mediaStreamAudioDestinationNode =
            new MediaStreamAudioDestinationNode(audioContext);

        mediaStreamAudioSourceNode.connect(mediaStreamAudioDestinationNode);

        mediaRecorder = new ImportedMediaRecorder(
            mediaStreamAudioDestinationNode.stream,
            {
                mimeType: "audio/wav",
            },
        );
        mediaRecorder.ondataavailable = (e: any) => {
            chunks.push(e.data);
        };
        mediaRecorder.onstop = (e: any) => {
            audioBlob = new Blob(chunks, { type: "audio/wav" });
            chunks = [];
            processAudioBlob(audioBlob, audioParameters.max_sample_length_ms);

            // checkAudioLength(audioBlob);
            // dispatch('voiceRecorded', audioBlob);
        };
    }
</script>

<audio hidden bind:this={audioElement}></audio>

<button
    onpointerdown={handleRecordDown}
    class={"h-20 w-20 select-none rounded-full bg-red-600 text-2xl font-bold " +
        classes}
    class:recording>Rec</button
>

<style lang="postcss">
    .recording {
        @apply bg-slate-950 text-slate-50 dark:bg-slate-50 dark:text-slate-950;
    }
</style>
