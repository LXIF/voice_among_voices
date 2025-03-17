<script lang="ts">
    import { getContext } from "svelte";
    import type { SiweContextInterface } from "$lib/siwe/SiweContext.type";
    import { onMount } from "svelte";
    import { setIdentityAgent } from "$lib/canisters";
    import Button from "./Button.svelte";
    import { appkitModal } from "$lib/appKit.svelte";
    import { identityAgent } from "$lib/canisters";
  import { abbreviateWalletAddress } from "$lib/utils/convUtils";

    let context = getContext<SiweContextInterface>('siwe');
    let walletConnected = $state(false);
    let walletAddress = $state("");
    let isLoggingIn = $state(false);

    // $effect(() => {
    //     if(walletConnected && !$identityAgent && !isLoggingIn) {

    //     }
    // })

    onMount(() => {
        context = getContext<SiweContextInterface>('siwe');
        $appkitModal.subscribeState((newState) => {
            if (newState.initialized) {
                walletConnected = $appkitModal.getIsConnectedState();
                if(walletConnected && !$identityAgent && !isLoggingIn) {
                    $appkitModal.disconnect();
                }
            }
        });
    });

    async function handleLogin() {
            $appkitModal.open();

            $appkitModal.subscribeAccount((newState) => {
                walletConnected = newState.isConnected
                walletAddress = newState.address ?? "";
                if(walletConnected && !$identityAgent && !isLoggingIn) {
                    isLoggingIn = true;
                    context.login()
                        .then(async (response) => {
                            setIdentityAgent(response);
                            isLoggingIn = false;
                        });
                }
            });
        }

    async function handleLogout() {
        $appkitModal.disconnect();
        setIdentityAgent(undefined);
    }
</script>

{#if !$identityAgent}
<Button class="text-xl font-bold" onclick={handleLogin}>Login</Button>
{:else}
<!-- TODO handle styling -->
<div class="flex flex-col justify-end items-end">
    <Button class="text-xl font-bold" onclick={handleLogout}>Logout</Button>
    <p>{abbreviateWalletAddress(walletAddress)}</p>
</div>
{/if}