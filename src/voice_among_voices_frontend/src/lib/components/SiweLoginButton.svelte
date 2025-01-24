<script lang="ts">
    import { getContext } from "svelte";
    import type { SiweContextInterface } from "$lib/siwe/SiweContext.type";
    import { onMount } from "svelte";
    import { setIdentityAgent } from "$lib/canisters";

    let context = getContext<SiweContextInterface>('siwe');

        onMount(() => {
            context = getContext<SiweContextInterface>('siwe');
                setTimeout(() => {
                    console.log(context);
                }, 200)
        });

    async function handleLogin() {
            const loginResponse = await context.login();
            console.log(loginResponse);
            setIdentityAgent(loginResponse);
        }
    
</script>

<button onclick={handleLogin}>login with siwe</button>