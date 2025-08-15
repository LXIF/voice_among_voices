<script lang="ts">
    import { showParticipationOverlay } from "$lib/state/uxState";
    import { fade, scale } from "svelte/transition";
    import { quintOut } from "svelte/easing";
    import { myTokens } from "$lib/state/uxState";
    import { PUBLIC_OPENSEA_URL } from "$lib/config/public";
    import { identityAgent } from "$lib/canisters";

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
            {#if $identityAgent && $myTokens.length > 0}
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
                    <span class="font-italic font-bold">Voice among Voices</span
                    >
                    is a participatory sound artwork that invites you to become part
                    of a collective voice and listening experience.<br /><br
                    />To contribute your voice to the artwork, purchase one of
                    the 360
                    <span class="font-italic">VaV</span>
                    NFTs available on
                    <a
                        href={PUBLIC_OPENSEA_URL}
                        target="_blank"
                        class="font-italic">OpenSea</a
                    >.<br /><br />
                    To listen to the public audio at angle 0, click
                    <span class="font-italic">Load</span>
                    and then <span class="font-italic">Play</span>.
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
