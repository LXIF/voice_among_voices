<script lang="ts">
    import {
        myTokens,
        showInfoModal,
        showCensorModal,
        fetchAdminTokenId,
        adminTokenId,
        isAdmin,
    } from "$lib/state/uxState";
    import { onMount } from "svelte";
    import Button from "./Button.svelte";

    onMount(() => {
        fetchAdminTokenId();
    });
</script>

<footer class="absolute bottom-0 flex w-full justify-between px-4 pb-4">
    <h1 class="flex-1 select-none text-left text-xl font-bold">Beschrieb</h1>
    {#if $isAdmin}
        <Button
            onclick={() => ($showCensorModal = !$showCensorModal)}
            class="flex-1 text-center text-xl">Censor</Button
        >
    {/if}
    <div class="flex flex-1 items-end justify-end">
        <Button
            onclick={() => ($showInfoModal = !$showInfoModal)}
            class="origin-center text-right text-xl">Info</Button
        >
    </div>
</footer>
