<script>
    import '../app.css';
    import { onMount } from 'svelte';

    let { children } = $props();
    let initialized = $state(false);

    onMount(() => {
        console.log("[Debug] Layout mounted");
        console.log("[Debug] Ethereum object:", window.ethereum);
        console.log("[Debug] Web3 availability:", {
            ethereum: !!window.ethereum,
            wagmi: !!window.wagmi,
            viem: !!window.viem
        });
        
        try {
            // Add any global initialization checks here
            console.log("[Debug] Layout initialization complete");
            initialized = true;
        } catch (error) {
            console.error("[Debug] Layout initialization failed:", error);
        }
    });
</script>

<div class="min-w-screen min-h-dvh bg-white text-black dark:bg-slate-950 dark:text-gray-100">
    {#if initialized}
        {@render children?.() }
    {:else}
        <div>Initializing...</div>
    {/if}
</div>

<svelte:window 
    on:error={(e) => console.error("[Debug] Global error:", e)} 
    on:unhandledrejection={(e) => {
        console.error("[Debug] Window error:", {
            message: e.reason?.message,
            stack: e.reason?.stack,
            type: e.reason?.type
        });
    }} 
/>
