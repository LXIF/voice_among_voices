<script lang="ts">
    import { toastMessage } from "$lib/state/uxState";
    import { onMount } from "svelte";
    import { slide } from "svelte/transition";

    onMount(() => {
        setTimeout(() => {
            toastMessage.set("");
        }, 3000);
    });
</script>

<div
    class="fixed bottom-0 left-0 right-0 z-50 flex justify-center"
    transition:slide={{
        axis: "y",
    }}
>
    <div
        class="rounded-lg border border-black bg-white p-4 text-black shadow-lg dark:border-white dark:bg-slate-950 dark:text-white"
    >
        <p>{$toastMessage}</p>
    </div>
</div>
