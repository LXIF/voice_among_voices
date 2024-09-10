<script lang="ts">
    import {backend} from '$lib/canisters';
    import {onMount} from 'svelte';
    import NodeMap from '$lib/components/NodeMap.svelte';
    import NodeMapPhysics from '$lib/components/NodeMapPhysics.svelte';

    let voiceNodes: VoiceNode[] = [];
    let id = '';
    let x = '';
    let y = '';
    let sample = '';

    async function onSubmit(event: SubmitEvent) {
        const voiceNode = {
            id: Number(id),
            x: Number(x),
            y: Number(y),
            sample: sample,
        };

        // Add the voice node to the backend
        await backend.add_voice_node(voiceNode);

        // Fetch the updated list of voice nodes
        voiceNodes = await backend.voice_nodes();
    }

    onMount(async () => {
        voiceNodes = await backend.voice_nodes();
    });
</script>

<main class="flex justify-center items-center flex-col">
    <h1>VOICE AMONG VOICES</h1>
    <form on:submit|preventDefault={onSubmit}>
        <label for="id">Enter ID (u16): &nbsp;</label>
        <input
            id="id"
            alt="ID"
            type="number"
            bind:value={id}
        />
        <br /><br />

        <label for="x">Enter X coordinate (u64): &nbsp;</label>
        <input
            id="x"
            alt="X"
            type="number"
            bind:value={x}
        />
        <br /><br />

        <label for="y">Enter Y coordinate (u64): &nbsp;</label>
        <input
            id="y"
            alt="Y"
            type="number"
            bind:value={y}
        />
        <br /><br />

        <label for="sample">Enter Sample: &nbsp;</label>
        <input
            id="sample"
            alt="Sample"
            type="text"
            bind:value={sample}
        />
        <br /><br />

        <button type="submit">Add Voice Node</button>
    </form>

    <NodeMap nodes={voiceNodes} />
    <NodeMapPhysics nodes={voiceNodes} />
</main>
