<script>
    import "../app.css";
    import Toast from "$lib/components/Toast.svelte";
    import { toastMessage } from "$lib/state/uxState";

    let { children } = $props();
</script>

<div
    class="min-w-screen -z-50 h-screen min-h-screen bg-slate-50 text-black dark:bg-slate-950 dark:text-gray-100"
>
    {@render children?.()}
    {#if $toastMessage}
        <Toast />
    {/if}
</div>

<svelte:window
    on:error={(e) => console.error("[Debug] Global error:", e)}
    on:unhandledrejection={(e) => {
        console.error("[Debug] Window error:", {
            message: e.reason?.message,
            stack: e.reason?.stack,
            type: e.reason?.type,
        });
    }}
/>
