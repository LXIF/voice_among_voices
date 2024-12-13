<script lang="ts">
    import { appkitModal } from "$lib/appKit.svelte";

// $appkitModal.subscribeWalletInfo((newState) => console.log(newState));
// $appkitModal.subscribeAccount((newState) => console.log(newState));
</script>

<button onclick={() => {
    $appkitModal.open();
}}>
    Connect Wallet
</button>