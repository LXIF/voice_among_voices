<script>
    import '../app.css';

    let { children } = $props();
</script>

<div class="min-h-screen min-w-screen bg-white text-black dark:bg-slate-950 dark:text-gray-100">
    {@render children?.() }
</div>
