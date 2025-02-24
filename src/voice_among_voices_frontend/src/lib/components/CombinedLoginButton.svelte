<script lang="ts">
    import { getContext } from "svelte";
    import type { SiweContextInterface } from "$lib/siwe/SiweContext.type";
    import { onMount } from "svelte";
    import { setIdentityAgent } from "$lib/canisters";
    import Button from "./Button.svelte";
    import { appkitModal } from "$lib/appKit.svelte";
    import { identityAgent } from "$lib/canisters";

    let context = getContext<SiweContextInterface>('siwe');
    let walletConnected = $state(false);
    let isLoggingIn = $state(false);

    $effect(() => {
        if(walletConnected && !$identityAgent && !isLoggingIn) {
            isLoggingIn = true;
            context.login()
                .then((response) => {
                    setIdentityAgent(response);
                    isLoggingIn = false;
                });
        }
    })

    onMount(() => {
        context = getContext<SiweContextInterface>('siwe');
        $appkitModal.subscribeState((newState) => {
            if (newState.initialized) {
                walletConnected = $appkitModal.getIsConnectedState();
            }
        });
    });

    async function handleLogin() {
            $appkitModal.open();

            $appkitModal.subscribeAccount((newState) => {
                walletConnected = newState.isConnected
            });
            // const loginResponse = await context.login();
            // setIdentityAgent(loginResponse);
        }

    async function handleLogout() {
        $appkitModal.disconnect();
    }

        //TODO: add connect wallet functionality into here
        //TODO: Login button should reflect logged-in state with address
</script>

{#if !$identityAgent}
<Button class="text-xl font-bold" onclick={handleLogin}>Login</Button>
{:else}
<Button class="text-xl font-bold" onclick={handleLogout}>Logout</Button>
{/if}