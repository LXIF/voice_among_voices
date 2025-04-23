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
    import { getWalletClient, type Config } from "@wagmi/core";

    let walletConnected = $state(false);
    let isLoggingIn = $state(false);
    let isLoggingOut = $state(false);

    onMount(async () => {
        if (!$appkitModal) throw "Appkit Modal not initialized!";

        $appkitModal.subscribeState(async (newState) => {
            if (newState.initialized) {
                walletConnected = $appkitModal.getIsConnectedState();

                if (walletConnected && !$identityAgent && !isLoggingIn) {
                    isLoggingIn = true;
                    setSiweWalletClient();
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

        $appkitModal.subscribeAccount(async (newState) => {
            walletConnected = newState.isConnected;
            $walletAddress = newState.address ?? "";

            setSiweWalletClient();

            if (walletConnected && !$identityAgent && !isLoggingIn) {
                isLoggingIn = true;
                loginSiwe();
            }
        });
    }

    async function loginSiwe() {
        if (!$walletAddress) {
            try {
                $walletAddress = $appkitModal?.getAddress()!;
            } catch {
                console.log("Please connect a wallet first");
                return;
            }
        }
        $siwe!.clear();
        $siwe!
            .login()
            .then(async (response) => {
                setIdentityAgent(response);
                isLoggingIn = false;
            })
            .then(() => {
                $appkitModal?.close();
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
        isLoggingIn = false;
    }

    async function setSiweWalletClient() {
        if (!$appkitModal) throw "AppkitModal not initialized";

        const client = await getWalletClient(
            (
                $appkitModal.chainAdapters?.eip155 as any as {
                    wagmiConfig: Config;
                }
            ).wagmiConfig,
        );
        $siwe?.setWalletClient(client);
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
    <Dialog
        title={"Finish Login"}
        onClose={() => {
            isLoggingIn = false;
            isLoggingOut = false;
        }}
        closeOnOutsideClick
    >
        Not connecting automatically?
        <div class="m-2 mt-4 flex justify-between lg:min-w-96">
            <Button
                class="rounded-full border border-slate-950 px-4 py-2 dark:border-white"
                onclick={handleLogout}>Disconnect</Button
            >
            <Button
                class="rounded-full border border-slate-950 px-4 py-2 dark:border-white"
                onclick={loginSiwe}>Finish connecting</Button
            >
        </div>
    </Dialog>
{/if}
