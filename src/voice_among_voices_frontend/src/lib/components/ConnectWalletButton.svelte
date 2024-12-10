<script lang="ts">
    // TODO
</script>

{#if $web3Modal}
    <button on:click={$web3Modal.open()}>Connect Wallet</button>
{:else}
    <p>login not yet available</p>
{/if}