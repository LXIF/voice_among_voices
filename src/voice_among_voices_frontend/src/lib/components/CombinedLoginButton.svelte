<script lang="ts">
    import { onMount } from "svelte";
    import Button from "./Button.svelte";
    import { appkitModal, wagmiConfig } from "$lib/appKit";
    import { backend, identityAgent } from "$lib/canisters";
    import { abbreviateWalletAddress } from "$lib/utils/convUtils";
    import {
        walletAddress,
        resetUxState,
        applicationState,
        applicationStates,
        toastMessage,
        myTokens,
        hasNoTokens,
    } from "$lib/state/uxState";
    import { siwe } from "$lib/siwe/siwe";
    import Dialog from "./Dialog.svelte";
    import { getWalletClient, type Config } from "@wagmi/core";
    import { scale } from "svelte/transition";
    import { elasticIn, elasticInOut, elasticOut } from "svelte/easing";

    let walletConnected = $state(false);
    let isLoggingIn = $state(false);
    let isLoggingOut = $state(false);
    let tokenBuyLink = $state("");
    let showCopied = $state(false);
    let showCopiedNoTokensModal = $state(false);

    onMount(async () => {
        if (!$appkitModal) throw "Appkit Modal not initialized!";
        tokenBuyLink = await backend.get_token_buy_link();
        setAddress();
    });

    // less elegant, but subscribestate is insufficient apparently
    function setAddress() {
        const address = $appkitModal?.getAddress();
        if (address === undefined) {
            setTimeout(setAddress, 50);
        } else {
            $walletAddress = address;
        }
    }

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
                $toastMessage = "Please connect a wallet first";
                return;
            }
        }
        $siwe!.clear();
        $siwe!
            .login()
            .then(() => {
                isLoggingIn = false;
            })
            .then(() => {
                $appkitModal?.close();
            });
    }

    async function handleLogout() {
        console.log("Logging out of Voice among Voices!"); // somehow needs to be here lol
        isLoggingOut = true;
        if (!$appkitModal) throw "Appkit Modal not initialized!";
        await $appkitModal.disconnect();
        $siwe!.clear();
        resetUxState();
        isLoggingOut = false;
        isLoggingIn = false;
        $applicationState = applicationStates.loggedOut;
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
        <Button
            class="relative font-light"
            onclick={() => {
                navigator.clipboard.writeText($walletAddress);
                showCopied = true;
                setTimeout(() => (showCopied = false), 2000);
            }}
        >
            {abbreviateWalletAddress($walletAddress)}
            {#if showCopied}
                <div
                    class="absolute -top-8 left-1/2 -translate-x-1/2 transform rounded bg-slate-800 px-2 py-1 text-sm text-white"
                    transition:scale={{
                        easing: elasticOut,
                    }}
                >
                    Copied!
                </div>
            {/if}
        </Button>
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

{#if $hasNoTokens && $identityAgent}
    <Dialog>
        <p>
            You don't own any Voice among Voices NFTs. In order to contribute
            your voice, please acquire one <a
                target="_blank"
                class="origin-center underline outline-none transition-transform hover:scale-110"
                href={tokenBuyLink}>here</a
            >
            and send it to your wallet:
            <Button
                class="mt-2 w-full rounded-full bg-slate-700 py-2 text-center text-xs hover:scale-105"
                onclick={() => {
                    navigator.clipboard.writeText($walletAddress);
                    showCopiedNoTokensModal = true;
                    setTimeout(() => (showCopiedNoTokensModal = false), 1000);
                }}
                >{$walletAddress}
                {#if showCopiedNoTokensModal}
                    <div
                        class="absolute -top-8 left-1/2 -translate-x-1/2 transform rounded bg-slate-800 px-2 py-1 text-sm text-white"
                        transition:scale={{
                            easing: elasticOut,
                        }}
                    >
                        Copied!
                    </div>
                {/if}</Button
            >
        </p>
        <div class="m-2 mt-4 flex items-center justify-center lg:min-w-96">
            <Button
                class="rounded-full border border-slate-950 px-4 py-2 dark:border-white"
                onclick={handleLogout}
                >Disconnect
            </Button>
        </div>
    </Dialog>
{/if}
