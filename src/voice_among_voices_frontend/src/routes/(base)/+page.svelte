<script lang="ts">
    import Header from "$lib/components/Header.svelte";
    import Footer from "$lib/components/Footer.svelte";
    import NodeMapContainer from "$lib/components/NodeMapContainer.svelte";
    import AudioContainer from "$lib/components/AudioContainer.svelte";
    import { showCensorModal, showInfoModal } from "$lib/state/uxState";
    import InfoModal from "$lib/components/InfoModal.svelte";
    import CensorModal from "$lib/components/CensorModal.svelte";

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
</script>

<Header />
<NodeMapContainer bind:this={nodeMapCont} />
<AudioContainer onDropNodeWithRadius={handleDropNodeWithRadius} />
{#if $showInfoModal}
    <InfoModal />
{/if}
{#if $showCensorModal}
    <CensorModal />
{/if}
<Footer />
