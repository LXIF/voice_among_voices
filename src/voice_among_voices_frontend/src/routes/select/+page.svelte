<script lang="ts">
    import AngleSelector from "$lib/components/AngleSelector.svelte";
    import { backend } from "$lib/canisters";
    import type { VoiceNodeEgress } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import { onMount } from "svelte";
    import { getVoiceNodes } from "$lib/icInteractions";

    let availableAngles = [1, 2, 3, 4, 5, 6, 7, 100, 101, 102, 251, 252, 253];
    let nodes: VoiceNodeEgress[];

    onMount(async () => {
        nodes = await getVoiceNodes();
    });
</script>

<AngleSelector {availableAngles} {nodes} />
