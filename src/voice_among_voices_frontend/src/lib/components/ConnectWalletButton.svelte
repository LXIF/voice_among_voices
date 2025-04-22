<script lang="ts">
    import { appkitModal } from "$lib/appKit";

    // $appkitModal.subscribeWalletInfo((newState) => console.log(newState));
    // $appkitModal.subscribeAccount((newState) => console.log(newState));
</script>

<button
    class="font-display"
    onclick={() => {
        $appkitModal!.open();
    }}
>
    Connect Wallet
</button>
