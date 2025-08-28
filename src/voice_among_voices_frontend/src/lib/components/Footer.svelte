<script lang="ts">
    import {
        myTokens,
        showInfoModal,
        showCensorModal,
        showParticipationOverlay,
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

<footer
    class="pointer-events-none absolute bottom-0 flex w-full justify-between px-4 pb-4"
>
    <div class="flex flex-1 items-start justify-start">
        <Button
            onclick={() =>
                ($showParticipationOverlay = !$showParticipationOverlay)}
            class="pointer-events-auto origin-center text-left text-xl"
            >How to participate?</Button
        >
    </div>
    {#if $isAdmin}
        <Button
            onclick={() => ($showCensorModal = !$showCensorModal)}
            class="pointer-events-auto flex-1 text-center text-xl"
            >Censor</Button
        >
    {/if}
    <div class="flex flex-1 items-end justify-end">
        <Button
            onclick={() => ($showInfoModal = !$showInfoModal)}
            class="pointer-events-auto origin-center text-right text-xl"
            >Info</Button
        >
    </div>
</footer>
