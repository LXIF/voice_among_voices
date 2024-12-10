<script>
    import '../app.css';
    import SiweContext from '$lib/siwe/SiweContext.svelte';

    // TODO
    import { onMount } from 'svelte';
    import { PUBLIC_ALCHEMY_ID } from '$env/static/public';
	import { injected } from '@wagmi/connectors';

    onMount(async () => {
		const erckit = defaultConfig({
			appName: 'voice_among_voices',
            alchemyId: PUBLIC_ALCHEMY_ID,
			connectors: [injected()]
		});

        await erckit.init();
	});

    $inspect($wagmiLoaded);
    $inspect($connected);

    let { children } = $props();
</script>

<SiweContext>
    {@render children?.() }
</SiweContext>
