<script lang="ts">
    import {backend} from '$lib/canisters';
    import {getContext, onMount} from 'svelte';
    import DroppableNode from '$lib/components/DroppableNode.svelte';
    import NodeMapPhysics from '$lib/components/NodeMapPhysics.svelte';
    import VoiceRecorder from '$lib/components/VoiceRecorder.svelte';
    import type {
        VoiceNodeEgress,
        SimulationParameters,
        AudioParameters,
    } from '../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did';
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
    import { canisterId, idlFactory } from "../../../declarations/ic_siwe_provider";
  import type { Principal } from '@dfinity/principal';

    let voiceNodes: VoiceNodeEgress[] = $state([]);
    let backendSimulationResult: VoiceNodeEgress[] = $state([]);
    let simulationParameters: SimulationParameters | undefined = $state();
    let audioParameters: AudioParameters | undefined = $state();
    let sampleLength = $state(0);
    let nodeWidthPx = $state(0);
    let nodeWidthLogical = $state(0);
    let currentVoiceBlob: Blob;
    let myVoice;

    let myCurrentSampleAudioElement: HTMLAudioElement | undefined = $state();

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
        myVoice = await backend.get_my_voice();

        if (myVoice.length > 0 && myCurrentSampleAudioElement) {
            const sample = myVoice[angle].sample;
            const audioURL = await handleBackendAudioData(
                sample instanceof Uint8Array ? sample : new Uint8Array(sample)
            );
            myCurrentSampleAudioElement.src = audioURL;
        }
    });

    const handleDropNewNode = async (event: CustomEvent) => {
        const sample = await blobToUint8Array(currentVoiceBlob);
        const {x, y, id} = event.detail;

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

    const handleRecordingLength = (e: CustomEvent) => {
        if(audioParameters === undefined || simulationParameters === undefined) throw "invalid params";

            sampleLength = e.detail;
            
            if(sampleLength > 0) {
                const nodeWidths = calculateNodeWidth(
                    sampleLength,
                    usableCanvasWidth,
                    audioParameters.total_length_ms,
                    simulationParameters.logical_radius * 2
                );
        
                nodeWidthPx = nodeWidths.nodeWidthPx;
                nodeWidthLogical = nodeWidths.nodeWidthLogical;
            }
    };

    const handleVoiceRecorded = (e: CustomEvent) => {
        currentVoiceBlob = e.detail;
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
        const addressResult = await backend.get_wallet_address(myPrincipal as unknown as Principal);
        if('Ok' in addressResult) {
            myAddress = addressResult.Ok
        }
    };
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
        <NodeMapPhysics
            nodes={voiceNodes}
            backendNodes={backendSimulationResult}
            on:dropNewNode={handleDropNewNode}
            {dragging}
            showPlayHead={fileLoaded}
            playHeadAngle={angle}
            playHeadPosition={playheadPosition}
            on:movePlayHead={(e) => {
                externalPlaybackPosition = e.detail;
            }}
        />
        <DroppableNode
            {nodeWidthPx}
            {nodeWidthLogical}
            on:dragstart={() => (dragging = true)}
            on:dragend={() => (dragging = false)}
        />
        <VoiceRecorder
            on:recordingLength={handleRecordingLength}
            on:voiceRecorded={handleVoiceRecorded}
            {audioParameters}
        />
        <h1>Current own voice:</h1>
        <audio
            controls
            bind:this={myCurrentSampleAudioElement}
        ></audio>
        <AngleFileBox
            {externalPlaybackPosition}
            on:playbackPosition={(e) => (playheadPosition = e.detail)}
            on:fileAngle={(e) => (angle = e.detail)}
            on:fileLoaded={(e) => (fileLoaded = e.detail)}
        />
        <ZeroFileBox
            {externalPlaybackPosition}
            on:playbackPosition={(e) => (playheadPosition = e.detail)}
            on:fileAngle={(e) => (angle = e.detail)}
            on:fileLoaded={(e) => (fileLoaded = e.detail)}
        />
    </main>
</SiweContext>
