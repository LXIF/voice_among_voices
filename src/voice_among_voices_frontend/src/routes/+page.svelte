<script lang="ts">
    import {backend} from '$lib/canisters'; // complains but works
    import {onMount} from 'svelte';
    import NodeMap from '$lib/components/NodeMap.svelte';
    import NodeMapPhysics from '$lib/components/NodeMapPhysics.svelte';
    import VoiceRecorder from '$lib/components/VoiceRecorder.svelte';
    import type {VoiceNodeEgress} from '../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did';

    let voiceNodes: VoiceNodeEgress[] = [];
    let backendSimulationResult: VoiceNodeEgress[] = [];

    onMount(async () => {
        voiceNodes = await backend.voice_nodes();
    });

    const handleDropNewNode = async (event: CustomEvent) => {
        let backend_simulation_result = await backend.add_voice_node(
            event.detail
        );
        if (backend_simulation_result.Ok) {
            backendSimulationResult = backend_simulation_result.Ok;
        }
        voiceNodes = await backend.voice_nodes();
    };

    const handleRecording = (e: CustomEvent) => {
        console.log(e.detail);
    };
</script>

<main class="flex justify-center items-center flex-col h-[100vh]">
    <NodeMapPhysics
        nodes={voiceNodes}
        backendNodes={backendSimulationResult}
        on:dropNewNode={handleDropNewNode}
    />
    <VoiceRecorder on:recordingLength={handleRecording} />
</main>
