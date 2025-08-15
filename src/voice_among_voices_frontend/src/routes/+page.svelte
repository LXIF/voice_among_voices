<script lang="ts">
    import Header from "$lib/components/Header.svelte";
    import Footer from "$lib/components/Footer.svelte";
    import NodeMapContainer from "$lib/components/NodeMapContainer.svelte";
    import AudioContainer from "$lib/components/AudioContainer.svelte";
    import {
        showCensorModal,
        showInfoModal,
        showParticipationOverlay,
    } from "$lib/state/uxState";
    import InfoModal from "$lib/components/InfoModal.svelte";
    import CensorModal from "$lib/components/CensorModal.svelte";
    import ParticipationOverlay from "$lib/components/ParticipationOverlay.svelte";

    let nodeMapCont: NodeMapContainer | undefined = $state();

    const handleDropNodeWithRadius = ({
        nodeX,
        nodeY,
        nodeRadius,
    }: {
        nodeX: number;
        nodeY: number;
        nodeRadius: number;
    }) => {
        if (nodeMapCont) {
            nodeMapCont.handleDrop({ nodeX, nodeY, nodeRadius });
        }
    };

    const handleFinishRecord = () => {
        if (nodeMapCont) {
            nodeMapCont.resetNodes();
        }
    };
</script>

<Header />
<div class="flex flex-col pt-[16vh] md:min-h-screen md:pt-0">
    <div class="flex flex-1 items-center justify-center">
        <div class="flex flex-col items-center gap-4">
            <NodeMapContainer bind:this={nodeMapCont} />
            <AudioContainer
                onDropNodeWithRadius={handleDropNodeWithRadius}
                onFinishRecord={handleFinishRecord}
            />
        </div>
    </div>
</div>
<Footer />
{#if $showInfoModal}
    <InfoModal />
{/if}
{#if $showCensorModal}
    <CensorModal />
{/if}
{#if $showParticipationOverlay}
    <ParticipationOverlay />
{/if}
