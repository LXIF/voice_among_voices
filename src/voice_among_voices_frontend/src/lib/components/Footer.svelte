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
    <h1 class="select-none text-xl font-bold">Beschrieb</h1>
    {#if $isAdmin}
        <Button
            onclick={() => ($showCensorModal = !$showCensorModal)}
            class="text-xl">Censor</Button
        >
    {/if}
    <Button onclick={() => ($showInfoModal = !$showInfoModal)} class="text-xl"
        >Info</Button
    >
</footer>
