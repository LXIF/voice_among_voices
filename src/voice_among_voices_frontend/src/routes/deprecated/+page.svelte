<script lang="ts">
    import {backend} from '$lib/canisters';
    import {getContext, onMount} from 'svelte';
    import DroppableNode from '$lib/components/DroppableNode.svelte';
    import NodeMapPhysics from '$lib/components/NodeMapPhysics.svelte';
    import VoiceRecorder from '$lib/components/VoiceRecorder.svelte';
    import type {
        VoiceNodeEgress,
        VoiceNodeIngress,
        SimulationParameters,
        AudioParameters,
        AudioSample,
    } from '../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did';
    import {usableCanvasWidth} from '$lib/config/nodeMap';
    import {
        blobToUint8Array,
        handleBackendAudioData,
    } from '$lib/utils/convUtils';
    import AngleFileBox from '$lib/components/AngleFileBox.svelte';
    import ZeroFileBox from '$lib/components/ZeroFileBox.svelte';
    import ConnectWalletButton from '$lib/components/ConnectWalletButton.svelte';
    import SiweLoginButton from '$lib/components/SiweLoginButton.svelte';
    import SiweContext from '$lib/siwe/SiweContext.svelte';
    import { canisterId, idlFactory } from "../../../../declarations/ic_siwe_provider";

    let voiceNodes: VoiceNodeEgress[] = $state([]);
    let backendSimulationResult: VoiceNodeEgress[] = $state([]);
    let simulationParameters: SimulationParameters | undefined = $state();
    let audioParameters: AudioParameters | undefined = $state();
    let sampleLength = $state(0);
    let nodeWidthPx = $state(0);
    let nodeWidthLogical = $state(0);
    let currentVoiceBlob: Blob | undefined = $state();

    let dragging = $state(false);

    let playheadPosition = $state(0);
    let externalPlaybackPosition = $state(0);
    let angle = $state(0);
    let fileLoaded = $state(false);

    let myAddress = $state("");

    let myPrincipal: string | undefined = $state();

    onMount(async () => {
        voiceNodes = await backend.get_voice_nodes();
        simulationParameters = await backend.get_simulation_parameters();
        audioParameters = await backend.get_audio_parameters();
    });

    const handleDropNewNode = async (voiceNode: VoiceNodeIngress) => {
        const sample = await blobToUint8Array(currentVoiceBlob!);
        const {x, y, id} = voiceNode;

        let backend_simulation_result = await backend.update_voice_node({
            id,
            x,
            y,
            sample,
        });

        if ('Ok' in backend_simulation_result) {
            backendSimulationResult = backend_simulation_result.Ok;
        } else {
            if ('NotValidAudioFileError' in backend_simulation_result.Err) {
                console.log(backend_simulation_result.Err.NotValidAudioFileError);
            } else if ('NotWithinCircleError' in backend_simulation_result.Err) {
                console.log('Not within circle error');
            }
        }
        voiceNodes = await backend.get_voice_nodes();
    };

    const handleRecordingLength = (length: number) => {
        if(audioParameters === undefined || simulationParameters === undefined) throw "invalid params";

            sampleLength = length;

            if(sampleLength > 0) {
                const nodeWidths = calculateNodeWidth(
                    length,
                    usableCanvasWidth,
                    audioParameters.total_length_ms,
                    simulationParameters.logical_radius * 2
                );
        
                nodeWidthPx = nodeWidths.nodeWidthPx;
                nodeWidthLogical = nodeWidths.nodeWidthLogical;
            }
    };

    const handleVoiceRecorded = (blob: Blob) => {
        currentVoiceBlob = blob;
    };

    const calculateNodeWidth = (
        sampleLength: number,
        canvasWidth: number,
        totalLength: number,
        logicalWidth: number
    ) => {
        const pixelPerMs = canvasWidth / totalLength;
        const logicalPerMs = logicalWidth / totalLength;

        return {
            nodeWidthPx: sampleLength * pixelPerMs,
            nodeWidthLogical: sampleLength * logicalPerMs,
        };
    };

    const handleGetPrincipal = async () => {
        myPrincipal = await backend.get_my_principal();
    };

    const handleGetAddress = async () => {
        if(myPrincipal === undefined) throw "Principal is undefined";
        const addressResult = await backend.get_wallet_address();
        console.log(addressResult);
        if('Ok' in addressResult) {
            myAddress = addressResult.Ok
        }
    };

    const handleGetTokens = async () => {
        if(myAddress === undefined) throw "Address is undefined";
        const tokensResult = await backend.get_owned_tokens();
        console.log(tokensResult);
    }

    const handleGetBalance = async () => {
        if(myAddress === undefined) throw "Address is undefined";
        const balanceResult = await backend.get_balance();
        console.log(balanceResult);
    }

    const handleIsOwnerOf = async () => {
        if(myAddress === undefined) throw "Address is undefined";
        const isOwnerResult = await backend.is_owner_of(BigInt(1));
        console.log(isOwnerResult);
    }

</script>

<SiweContext {canisterId} {idlFactory}>
    <main class="flex justify-center items-center flex-col h-[100vh] pt-4">
        <ConnectWalletButton />
        <SiweLoginButton />
        <button onclick={handleGetPrincipal}>get principal</button>
        {#if myPrincipal}
            <div>{myPrincipal}</div>
        {/if}
        <button onclick={handleGetAddress}>get address</button>
        {#if myAddress}
            <div>{myAddress}</div>
        {/if}
        <button onclick={handleGetTokens}>get tokens</button>
        <button onclick={handleGetBalance}>get balance</button>
        <button onclick={handleIsOwnerOf}>get is Owner Of 1</button>
        <NodeMapPhysics
            nodes={voiceNodes}
            backendNodes={backendSimulationResult}
            dropNewNode={handleDropNewNode}
            {dragging}
            showPlayHead={fileLoaded}
            playHeadAngle={angle}
            playHeadPosition={playheadPosition}
            movePlayHead={(normalizedPosition) => {
                externalPlaybackPosition = normalizedPosition;
            }}
        />
        <DroppableNode
            {nodeWidthPx}
            {nodeWidthLogical}
            ondragstart={() => (dragging = true)}
            ondragend={() => (dragging = false)}
        />
        <VoiceRecorder
            recordingLength={handleRecordingLength}
            voiceRecorded={handleVoiceRecorded}
            {audioParameters}
        />
        <AngleFileBox
            {externalPlaybackPosition}
            onPlaybackPosition={(position) => (playheadPosition = position)}
            onFileAngle={(newAngle) => (angle = newAngle)}
            onFileLoaded={(loaded) => (fileLoaded = loaded)}
        />
        <ZeroFileBox
            {externalPlaybackPosition}
            onPlaybackPosition={(position) => (playheadPosition = position)}
            onFileAngle={(newAngle) => (angle = newAngle)}
            onFileLoaded={(loaded) => (fileLoaded = loaded)}
        />
    </main>
</SiweContext>
