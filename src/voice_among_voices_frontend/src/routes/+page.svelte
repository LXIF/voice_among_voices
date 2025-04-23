<script lang="ts">
    import Header from "$lib/components/Header.svelte";
    import Footer from "$lib/components/Footer.svelte";
    import NodeMapContainer from "$lib/components/NodeMapContainer.svelte";
    import AudioContainer from "$lib/components/AudioContainer.svelte";

    const handleDropNodeWithRadius = ({
        nodeX,
        nodeY,
        nodeRadius,
    }: {
        nodeX: number;
        nodeY: number;
        nodeRadius: number;
    }) => {
        console.log(nodeX, nodeY, nodeRadius);
    };
</script>

<Header />
<NodeMapContainer bind:handleCoolDrop={handleDropNodeWithRadius} />
<AudioContainer onDropNodeWithRadius={handleDropNodeWithRadius} />
<Footer />
