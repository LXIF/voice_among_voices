<script lang="ts">
    import { showInfoModal } from "$lib/state/uxState";
    import Button from "./Button.svelte";
</script>

<footer class="absolute bottom-0 flex w-full justify-between px-4 pb-4">
    <h1 class="select-none text-xl font-bold">Beschrieb</h1>
    <Button onclick={() => ($showInfoModal = !$showInfoModal)} class="text-xl"
        >Info</Button
    >
</footer>
