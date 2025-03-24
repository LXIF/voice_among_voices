<script>
    import '../app.css';

    let { children } = $props();
</script>

<div class="min-w-screen min-h-dvh bg-white text-black dark:bg-slate-950 dark:text-gray-100">
    {@render children?.() }
</div>
