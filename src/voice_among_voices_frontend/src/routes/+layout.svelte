<script>
    import "../app.css";

    let { children } = $props();
</script>

<div
    class="min-w-screen -z-50 min-h-dvh bg-white text-black dark:bg-slate-950 dark:text-gray-100"
>
    {@render children?.()}
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
