<script lang="ts">
    import { onMount } from "svelte";
    import { setIdentityAgent } from "$lib/canisters";
    import Button from "./Button.svelte";
    import { appkitModal, wagmiConfig } from "$lib/appKit";
    import { identityAgent } from "$lib/canisters";
    import { abbreviateWalletAddress } from "$lib/utils/convUtils";
    import { walletAddress, resetUxState } from "$lib/state/uxState";
    import { siwe } from "$lib/siwe/siwe";

    let walletConnected = $state(false);
    let isLoggingIn = $state(false);

    // $effect(() => {
    //     if(walletConnected && !$identityAgent && !isLoggingIn) {

    //     }
    // })

    onMount(() => {
        if (!$appkitModal) throw "Appkit Modal not initialized!";
        $appkitModal.subscribeState((newState) => {
            if (newState.initialized) {
                walletConnected = $appkitModal.getIsConnectedState();
                if (walletConnected && !$identityAgent && !isLoggingIn) {
                    // TODO: handle better
                    isLoggingIn = true;
                    $siwe!.login().then(async (response) => {
                        setIdentityAgent(response);
                        isLoggingIn = false;
                    });
                }
            }
        });
    });

    async function handleLogin() {
        if (!$appkitModal) throw "Appkit Modal not initialized!";
        $appkitModal.open();

        $appkitModal.subscribeAccount((newState) => {
            walletConnected = newState.isConnected;
            $walletAddress = newState.address ?? "";
            if (walletConnected && !$identityAgent && !isLoggingIn) {
                isLoggingIn = true;
                $siwe!.login().then(async (response) => {
                    setIdentityAgent(response);
                    isLoggingIn = false;
                });
            }
        });
    }

    // async function handleLoginSiwe() {
    //     // $appkitModal.open();
    //     context.login()
    //                 .then(async (response) => {
    //                     setIdentityAgent(response);
    //                     isLoggingIn = false;
    //                 });
    // }

    async function handleLogout() {
        if (!$appkitModal) throw "Appkit Modal not initialized!";
        $appkitModal.disconnect();
        setIdentityAgent(undefined);
        $siwe!.clear();
        resetUxState();
    }
</script>

{#if !$appkitModal || !$wagmiConfig}
    <Button class="text-xl font-bold cursor-wait">...</Button>
{:else if !$identityAgent}
    <Button class="text-xl font-bold" onclick={handleLogin}>Login</Button>
    <!-- <Button class="text-xl font-bold" onclick={handleLoginSiwe}>Login harder</Button> -->
{:else}
    <!-- TODO handle styling -->
    <div class="flex flex-col justify-end items-end">
        <Button class="text-xl font-bold" onclick={handleLogout}>Logout</Button>
        <p>{abbreviateWalletAddress($walletAddress)}</p>
    </div>
{/if}
