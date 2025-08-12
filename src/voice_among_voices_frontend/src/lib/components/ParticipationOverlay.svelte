<script lang="ts">
    import { showParticipationOverlay } from "$lib/state/uxState";
    import { fade, scale } from "svelte/transition";
    import { quintOut } from "svelte/easing";
    import { walletAddress, myTokens } from "$lib/state/uxState";
    import { PUBLIC_OPENSEA_URL } from "$lib/config/public";

    function closeOverlay() {
        $showParticipationOverlay = false;
    }

    function handleOverlayClick(event: MouseEvent) {
        // Close if clicking on the overlay background, not on the content
        if (event.target === event.currentTarget) {
            closeOverlay();
        }
    }
</script>

<!-- Backdrop -->
<div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm"
    onclick={handleOverlayClick}
    onkeydown={(e) => e.key === "Escape" && closeOverlay()}
    role="presentation"
    transition:fade={{ duration: 200 }}
>
    <!-- Modal Content -->
    <div
        class="relative flex flex-col items-center justify-center p-8 text-white"
        transition:scale={{ duration: 300, easing: quintOut }}
    >
        <!-- Instructional Text -->
        <div class="mb-8 text-center text-2xl leading-relaxed">
            {#if $walletAddress.length > 0 && $myTokens.length > 0}
                <p class="mb-4">
                    At this very moment, something may catch your eye, a sound
                    may make your eardrums vibrate, a thought may linger.
                </p>
                <p class="mb-4">
                    Use your voice to articulate such a perception of the
                    moment.
                </p>
                <p class="mb-4">
                    Press <span class="font-bold text-red-400">Rec</span> and sing
                    or sigh, verbalize or hum etc.
                </p>
                <p>Make your voice heard among voices.</p>
            {:else}
                <p class="mb-4">
                    Voice among Voices is a participative sound artwork.
                </p>
                <p class="mb-4">
                    You can listen to the zero angle by clicking Load, then Play
                    once it's loaded.
                </p>
                <p class="mb-4">
                    To add your voice to the voices, you need an EVM wallet and
                    a <a
                        href={PUBLIC_OPENSEA_URL}
                        target="_blank"
                        class="text-blue-400 underline transition-colors hover:text-blue-300"
                        >VaV NFT.</a
                    >
                </p>
            {/if}
        </div>

        <!-- Close Button -->
        <button
            class="mb-6 rounded-full transition-colors hover:scale-105"
            onclick={closeOverlay}
            aria-label="Close overlay"
        >
            <svg
                class="h-6 w-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M6 18L18 6M6 6l12 12"
                ></path>
            </svg>
        </button>
    </div>
</div>
