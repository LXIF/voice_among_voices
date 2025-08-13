<script lang="ts">
    import { backend } from "$lib/canisters";
    import { onMount } from "svelte";
    import { type VoiceLog } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import { abbreviateWalletAddress } from "$lib/utils/convUtils";

    let logs = $state<VoiceLog[]>([]);
    let isLoading = $state(false);
    let hasMore = $state(true);
    let offset = $state(BigInt(0));
    const limit = BigInt(50);

    let { class: classes }: { class?: string } = $props();

    onMount(async () => {
        await loadLogs();
    });

    async function loadLogs() {
        if (isLoading || !hasMore) return;

        isLoading = true;
        try {
            const newLogs = await backend.get_voice_logs(offset, limit);

            if (newLogs.length < Number(limit)) {
                hasMore = false;
            }

            if (newLogs.length > 0) {
                logs = [...logs, ...newLogs];
                offset += BigInt(newLogs.length);
            }
        } catch (error) {
            console.error("Failed to load logs:", error);
        } finally {
            isLoading = false;
        }
    }

    function handleScroll(event: Event) {
        const target = event.target as HTMLElement;
        const { scrollTop, scrollHeight, clientHeight } = target;

        // Load more when user is near the bottom (within 100px)
        if (
            scrollHeight - scrollTop - clientHeight < 100 &&
            !isLoading &&
            hasMore
        ) {
            loadLogs();
        }
    }

    function formatDate(timestamp: bigint) {
        return new Date(Number(timestamp / BigInt(1_000_000))).toLocaleString(
            "en-US",
            {
                month: "long",
                day: "numeric",
                year: "numeric",
                hour: "2-digit",
                minute: "2-digit",
            },
        );
    }
</script>

<div class={[classes, "max-h-full overflow-y-auto"]} onscroll={handleScroll}>
    {#each logs as log, i}
        <div
            class="px-6 py-4 {i !== logs.length - 1
                ? 'border-b border-gray-200'
                : ''}"
        >
            <div class="font-semibold">
                {abbreviateWalletAddress(log.initiator.address)}
                {#if "Drop" in log.action}
                    <span> dropped voice <b>{log.id}</b></span>
                {:else if "Censor" in log.action}
                    <span> censored voice <b>{log.id}</b></span>
                {/if}
            </div>
            {#if log.position && log.position[0]}
                <div class="mt-1 text-sm text-gray-600">
                    x: {log.position[0]?.x.toFixed(3)} y: {log.position[0]?.y.toFixed(
                        3,
                    )}
                </div>
            {/if}
            <div class="mt-1 text-sm text-gray-500">
                {formatDate(log.timestamp)}
            </div>
        </div>
    {/each}

    <!-- Loading indicator -->
    {#if isLoading}
        <div class="px-6 py-4 text-center text-gray-500">
            Loading more logs...
        </div>
    {/if}

    <!-- End indicator -->
    {#if !hasMore && logs.length > 0}
        <div class="px-6 py-4 text-center text-sm text-gray-400">
            No more logs to load
        </div>
    {/if}
</div>
