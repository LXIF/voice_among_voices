<script lang="ts">
	type ButtonVariant = 'primary' | 'secondary' | 'tertiary';
	let {
		children,
		class: classes,
		variant = 'primary',
		...others
	}: { variant?: ButtonVariant; class?: string; children?: any; [key: string]: any } = $props();
</script>

<button
	{...others}
	class={`${classes} ${variant} transition-all text-xl font-bold hover:scale-110`}
>
	{@render children?.()}
</button>

<style lang="postcss">
	/* .primary {
		@apply bg-tertiary dark:bg-tertiaryDark;
	}

	.secondary {
		@apply bg-secondary dark:bg-secondaryDark;
	}

	.tertiary {
		@apply bg-primary dark:bg-primaryDark;
	} */
</style>
