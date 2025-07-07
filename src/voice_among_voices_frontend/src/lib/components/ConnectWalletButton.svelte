<script lang="ts">
    import { appkitModal } from "$lib/appKit";
</script>

<button
    class="font-display"
    onclick={() => {
        $appkitModal!.open();
    }}
>
    Connect Wallet
</button>
