<script lang="ts">
    import Header from "$lib/components/Header.svelte";
    import Footer from "$lib/components/Footer.svelte";
    import NodeMapContainer from "$lib/components/NodeMapContainer.svelte";
    import AudioContainer from "$lib/components/AudioContainer.svelte";

    let nodeMapCont: HTMLDivElement | undefined = $state();

    const handleDropNodeWithRadius = ({
        nodeX,
        nodeY,
        nodeRadius,
    }: {
        nodeX: number;
        nodeY: number;
        nodeRadius: number;
    }) => {
        nodeMapCont.handleDrop({ nodeX, nodeY, nodeRadius });
    };
</script>

<Header />
<NodeMapContainer bind:this={nodeMapCont} />
<AudioContainer onDropNodeWithRadius={handleDropNodeWithRadius} />
<Footer />
