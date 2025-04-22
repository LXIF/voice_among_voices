<script lang="ts">
    import { onMount } from "svelte";
    import { setIdentityAgent } from "$lib/canisters";
    import Button from "./Button.svelte";
    import { appkitModal, wagmiConfig } from "$lib/appKit";
    import { identityAgent } from "$lib/canisters";
    import { abbreviateWalletAddress } from "$lib/utils/convUtils";
    import { walletAddress, resetUxState } from "$lib/state/uxState";
    import { siwe } from "$lib/siwe/siwe";
    import Dialog from "./Dialog.svelte";
    // import { getWalletClient } from "@wagmi/core";

    let walletConnected = $state(false);
    let isLoggingIn = $state(false);
    let isLoggingOut = $state(false);

    onMount(async () => {
        if (!$appkitModal) throw "Appkit Modal not initialized!";

        $appkitModal.subscribeState(async (newState) => {
            if (newState.initialized) {
                //TODO
                // console.log($wagmiConfig);

                // const walletClient = await getWalletClient($wagmiConfig);
                // console.log(walletClient);

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
                loginSiwe();
            }
        });
    }

    async function loginSiwe() {
        $siwe!.login().then(async (response) => {
            setIdentityAgent(response);
            isLoggingIn = false;
        });
    }

    async function handleLogout() {
        isLoggingOut = true;
        if (!$appkitModal) throw "Appkit Modal not initialized!";
        await $appkitModal.disconnect();
        setIdentityAgent(undefined);
        $siwe!.clear();
        resetUxState();
        isLoggingOut = false;
    }
</script>

{#if !$appkitModal || !$wagmiConfig}
    <Button class="cursor-wait text-xl font-bold">...</Button>
{:else if !$identityAgent}
    <Button class="text-xl font-bold" onclick={handleLogin}>Login</Button>
    <!-- <Button class="text-xl font-bold" onclick={handleLoginSiwe}>Login harder</Button> -->
{:else}
    <!-- TODO handle styling -->
    <div class="flex flex-col items-end justify-end">
        <Button class="text-xl font-bold" onclick={handleLogout}>Logout</Button>
        <p>{abbreviateWalletAddress($walletAddress)}</p>
    </div>
{/if}
{#if isLoggingIn && !isLoggingOut}
    <Dialog title={"Finish Login"}>
        Not connecting automatically?
        <Button onclick={loginSiwe}>Finish connecting</Button>
    </Dialog>
{/if}
