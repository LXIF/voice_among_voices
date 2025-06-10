<script lang="ts">
    import { backend } from "$lib/canisters";
    import { onMount } from "svelte";
    import { type VoiceLog } from "../../../../declarations/voice_among_voices_backend/voice_among_voices_backend.did";
    import { abbreviateWalletAddress } from "$lib/utils/convUtils";

    let logs = $state<VoiceLog[]>([]);

    let { class: classes }: { class?: string } = $props();

    onMount(async () => {
        logs = await backend.get_voice_logs(BigInt(0), BigInt(50));
    });

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

<div class={[classes, "max-h-full overflow-y-auto"]}>
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
</div>
