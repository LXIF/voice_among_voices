<script lang="ts">
    import NodeMapPhysics from "./NodeMapPhysics.svelte";
    import NewAngleSelector from "./NewAngleSelector.svelte";

    import { onMount } from "svelte";

    import { backend, identityAgent } from "$lib/canisters";
    import type { VoiceNodeIngress } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import { appkitModal } from "$lib/appKit.svelte";
    import { blobToUint8Array } from "$lib/utils/convUtils";
    import { voiceNodes, simulationParameters, backendSimulationResult, myAddress, myTokens, loadingTokens, selectedAngle, hoveredAngle, currentVoiceBlob, dragging, playheadPosition, externalPlaybackPosition, angle, fileLoaded } from "$lib/state/uxState.svelte";

    let myPrincipal: string | undefined = $state();



    let { class: classes } : { class?: string } = $props();

    onMount(async () => {
        $voiceNodes = await backend.get_voice_nodes();
        $simulationParameters = await backend.get_simulation_parameters();
    });

    $effect(() => {if($identityAgent) {
        $myAddress = $appkitModal.getAddress()!;
    }});

    $effect(() => {
        if($myAddress !== "") {
            fetchOwnedTokens()
        }
    });

    async function getOwnedTokensWithRetry(attempt = 1, maxRetries = 3): Promise<number[]> {
        const response = await backend.get_owned_tokens();
        
        if ('Ok' in response) {
            return response.Ok.map((token) => Number(token));
        }

        console.log("error loading tokens, retrying: ", response.Err);
        
        if (attempt < maxRetries) {
            await new Promise(resolve => setTimeout(resolve, 1000));
            return getOwnedTokensWithRetry(attempt + 1, maxRetries);
        }
        
        console.error('Failed to fetch owned tokens after', maxRetries, 'attempts');
        return [];
    }

    async function fetchOwnedTokens() {
        if($myTokens.length === 0) {
            $loadingTokens = true;
        }

        $myTokens = await getOwnedTokensWithRetry();
        $loadingTokens = false;
    }

    const handleDropNewNode = async (voiceNode: VoiceNodeIngress) => {
        const sample = await blobToUint8Array($currentVoiceBlob!);
        const {x, y, id} = voiceNode;

        let backend_simulation_result = await backend.update_voice_node({
            id,
            x,
            y,
            sample,
        });

        if ('Ok' in backend_simulation_result) {
            $backendSimulationResult = backend_simulation_result.Ok;
        } else {
            if ('NotValidAudioFileError' in backend_simulation_result.Err) {
                console.log(backend_simulation_result.Err.NotValidAudioFileError);
            } else if ('NotWithinCircleError' in backend_simulation_result.Err) {
                console.log('Not within circle error');
            }
        }
        $voiceNodes = await backend.get_voice_nodes();
    };

</script>

<div class="w-full h-full flex justify-center items-center scale-100">
    <NodeMapPhysics
        nodes={$voiceNodes}
        backendNodes={$backendSimulationResult}
        dropNewNode={handleDropNewNode}
        dragging={$dragging}
        showPlayHead={$fileLoaded}
        playHeadAngle={$angle}
        playHeadPosition={$playheadPosition}
        movePlayHead={(normalizedPosition) => {
            $externalPlaybackPosition = normalizedPosition;
        }}
        class="w-full h-full lg:max-w-[600px]"
    />
    <NewAngleSelector
        availableAngles={$myTokens}
        nodes={$voiceNodes}
        loading={$loadingTokens}
        class="absolute top-0 w-full h-full lg:max-w-[600px]"
        onSelectAngle={(angle) => $selectedAngle = angle}
        onHoverAngle={(angle) => $hoveredAngle = angle}
    />
</div>
