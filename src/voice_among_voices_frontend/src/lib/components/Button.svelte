<script lang="ts">
	let {
		children,
		class: classes,
		...others
	}: { class?: string; children?: any; [key: string]: any } = $props();
</script>

<button
	{...others}
	class={`${classes} transition-all text-xl font-bold hover:scale-110`}
>
	{@render children?.()}
</button>