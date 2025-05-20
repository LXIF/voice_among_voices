<script lang="ts">
    let {
        children,
        class: classes,
        ...others
    }: { class?: string; children?: any; [key: string]: any } = $props();
</script>

<button
    {...others}
    class={`${classes} select-none font-bold transition-transform hover:scale-110`}
>
    {@render children?.()}
</button>
