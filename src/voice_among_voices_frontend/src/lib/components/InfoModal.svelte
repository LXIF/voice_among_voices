<script lang="ts">
    import { showInfoModal } from "$lib/state/uxState";
    import Button from "./Button.svelte";
    import Dialog from "./Dialog.svelte";
    import LogsDisplay from "./LogsDisplay.svelte";

    type SelectedState = "info" | "log";

    let selected = $state<SelectedState>("info");

    function handleClick(newSelected: SelectedState) {
        selected = newSelected;
    }
</script>

<Dialog
    class="max-h-[50vh] text-slate-950 dark:text-white"
    onClose={() => ($showInfoModal = false)}
>
    <div class="mx-2 mb-2 mt-1 flex items-center justify-center gap-2">
        <Button
            onclick={() => handleClick("info")}
            class="{selected === 'info'
                ? 'border border-slate-950 dark:border-slate-50'
                : ''} rounded-full px-4 py-2">Info</Button
        >
        <Button
            onclick={() => handleClick("log")}
            class="{selected === 'log'
                ? 'border border-slate-950 dark:border-slate-50'
                : ''} rounded-full px-4 py-2">Log</Button
        >
    </div>
    {#if selected === "info"}
        <div>
            Lorem ipsum dolor sit amet consectetur adipisicing elit. Eum totam
            asperiores distinctio commodi facere rerum et quam blanditiis. Amet
            a delectus ducimus esse iure consequatur minus corporis odio quam
            vel.
        </div>
    {:else if selected === "log"}
        <div class="h-[calc(50vh-4rem)] overflow-hidden">
            <LogsDisplay />
        </div>
    {/if}
</Dialog>
