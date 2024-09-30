<script lang="ts">
    import {onMount} from 'svelte';
    import {createEventDispatcher} from 'svelte';
    // import {
    //     MediaRecorder,
    //     register,
    //     type IMediaRecorder,
    // } from 'extendable-media-recorder';
    // import {connect} from 'extendable-media-recorder-wav-encoder';
    import {browser} from '$app/environment';

    const dispatch = createEventDispatcher();

    let localStream: MediaStream;
    let audioElement: HTMLAudioElement;
    let mediaRecorder: any;
    let recording = false;
    let chunks: Blob[] = [];
    let audioBlob: Blob;
    let register: any;
    let connect: any;

    onMount(async () => {
        if (browser) {
            const {
                MediaRecorder: ImportedMediaRecorder,
                register: ImportedRegister,
            } = await import('extendable-media-recorder');

            // Dynamically import the WAV encoder
            const {connect: ImportedConnect} = await import(
                'extendable-media-recorder-wav-encoder'
            );

            // Store the imports in local variables
            // mediaRecorder = ImportedMediaRecorder;
            register = ImportedRegister;
            connect = ImportedConnect;

            function handleActivateMicrophone() {
                if (
                    navigator.mediaDevices &&
                    navigator.mediaDevices.getUserMedia
                ) {
                    navigator.mediaDevices
                        .getUserMedia({audio: true})
                        .then((stream) => (localStream = stream))
                        .then(setupMediaRecorder)
                        .catch((err) => {
                            console.error(`getUserMedia hiccup: ${err}`);
                        });
                } else {
                    console.log('getUserMedia not supported on your browser!');
                }
            }

            async function setupMediaRecorder() {
                if (!browser) return;

                await register(await connect());

                const audioContext = new AudioContext({sampleRate: 44100});
                const mediaStreamAudioSourceNode =
                    new MediaStreamAudioSourceNode(audioContext, {
                        mediaStream: localStream,
                    });
                const mediaStreamAudioDestinationNode =
                    new MediaStreamAudioDestinationNode(audioContext);

                mediaStreamAudioSourceNode.connect(
                    mediaStreamAudioDestinationNode
                );

                mediaRecorder = new ImportedMediaRecorder(
                    mediaStreamAudioDestinationNode.stream,
                    {
                        mimeType: 'audio/wav',
                    }
                );
                mediaRecorder.ondataavailable = (e: any) => {
                    chunks.push(e.data);
                };
                mediaRecorder.onstop = (e: any) => {
                    audioBlob = new Blob(chunks, {type: 'audio/wav'});
                    chunks = [];
                    const audioURL = window.URL.createObjectURL(audioBlob);
                    audioElement.src = audioURL;
                    checkAudioLength(audioBlob);
                    dispatch('voiceRecorded', audioBlob);
                };
            }

            handleActivateMicrophone();
        }
    });

    let audioDuration: number = 0;

    $: dispatch('recordingLength', audioDuration);

    function checkAudioLength(blob: Blob) {
        const fileReader = new FileReader();
        fileReader.readAsArrayBuffer(blob);

        fileReader.onloadend = () => {
            const audioData = fileReader.result;
            if (!audioData || typeof audioData === 'string') return;

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
                }
            );
        };
    }

    let recordingStart: number;
    let recordingInterval: ReturnType<typeof setInterval>;

    function handleRecordDown() {
        recording = true;
        window.addEventListener('pointerup', handleRecordUp);
        mediaRecorder?.start();

        recordingStart = Date.now();
        recordingInterval = setInterval(() => {
            const elapsed = Date.now() - recordingStart;
            dispatch('recordingLength', elapsed);
        }, 16);
    }

    function handleRecordUp() {
        clearInterval(recordingInterval);
        recording = false;
        window.removeEventListener('pointerup', handleRecordUp);
        mediaRecorder?.stop();
    }
</script>

<audio
    controls
    bind:this={audioElement}
></audio>
<!-- <button
    class="px-4 py-2 bg-slate-500 rounded-full"
    on:click={handleActivateMicrophone}>activate microphone</button
> -->
<button
    on:pointerdown={handleRecordDown}
    class="bg-red-600 rounded-full w-20 h-20"
    class:recording>record</button
>

<style lang="postcss">
    .recording {
        @apply bg-red-600;
    }
</style>
