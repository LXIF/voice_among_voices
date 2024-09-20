<script lang="ts">
    import {onMount} from 'svelte';
    import {createEventDispatcher} from 'svelte';

    const dispatch = createEventDispatcher();

    let localStream: MediaStream;
    let audioElement: HTMLAudioElement;
    let mediaRecorder: MediaRecorder;
    let recording = false;
    let chunks: Blob[] = [];
    let audioBlob: Blob;

    onMount(handleActivateMicrophone);

    let audioDuration: number = 0;

    $: dispatch('recordingLength', audioDuration);

    function handleActivateMicrophone() {
        if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
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

    function setupMediaRecorder() {
        mediaRecorder = new MediaRecorder(localStream);
        mediaRecorder.ondataavailable = (e) => {
            chunks.push(e.data);
        };
        mediaRecorder.onstop = (e) => {
            audioBlob = new Blob(chunks, {type: 'audio/wav'});
            chunks = [];
            const audioURL = window.URL.createObjectURL(audioBlob);
            audioElement.src = audioURL;
            checkAudioLength(audioBlob);
            dispatch('voiceRecorded', audioBlob);
        };
    }

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
