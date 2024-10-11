<script lang="ts">
    import '../app.css';

    import {Connect2ICProvider} from '@connect2ic/svelte';

    import {backend} from '$lib/canisters';

    import {browser} from '$app/environment';
    import {onMount} from 'svelte';

    let client;

    onMount(async () => {
        if (browser) {
            try {
                const core = await import('@connect2ic/core');
                const providers = await import('@connect2ic/core/providers');
                client = core.createClient({
                    canisters: {
                        backend,
                    },
                    providers: providers.defaultProviders,
                });
            } catch (e) {
                console.error(e);
            }
        }
    });
</script>

<Connect2ICProvider>
    <slot />
</Connect2ICProvider>
