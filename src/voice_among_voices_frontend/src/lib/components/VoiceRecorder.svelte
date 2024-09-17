<script lang="ts">
    import {onMount} from 'svelte';
    import {createEventDispatcher} from 'svelte';

    const dispatch = createEventDispatcher();

    let localStream;
    let audioElement;
    let mediaRecorder;
    let recording = false;
    let chunks = [];

    onMount(handleActivateMicrophone);

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
            dispatch('recordingLength', chunks.length);
        };
        mediaRecorder.onstop = (e) => {
            const blob = new Blob(chunks, {type: 'audio/wav'});
            chunks = [];
            const audioURL = window.URL.createObjectURL(blob);
            audioElement.src = audioURL;
        };
    }

    function handleRecordDown() {
        recording = true;
        window.addEventListener('pointerup', handleRecordUp);
        mediaRecorder?.start();
    }

    function handleRecordUp() {
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
