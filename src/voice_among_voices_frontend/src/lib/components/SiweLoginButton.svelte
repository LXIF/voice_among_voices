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

        //TODO: add connect wallet functionality into here
        //TODO: Login button should reflect logged-in state with address
</script>

<button class="text-xl font-bold" onclick={handleLogin}>Login</button>