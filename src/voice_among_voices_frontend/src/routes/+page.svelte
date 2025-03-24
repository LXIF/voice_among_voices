<script lang="ts">
    import Header from "$lib/components/Header.svelte";
    import Footer from "$lib/components/Footer.svelte";
    import NodeMapContainer from "$lib/components/NodeMapContainer.svelte";
    import AudioContainer from "$lib/components/AudioContainer.svelte";
    import SiweContext from "$lib/siwe/SiweContext.svelte";
    import { canisterId, idlFactory } from "../../../declarations/ic_siwe_provider";
</script>

<SiweContext {canisterId} {idlFactory}>
        <Header />
        <NodeMapContainer />
        <AudioContainer />
        <Footer />
</SiweContext>