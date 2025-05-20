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
</script>

<div class={[classes, "max-h-full overflow-y-auto"]}>
    {#each logs as log}
        {#if "Drop" in log.action}
            <p>
                {abbreviateWalletAddress(log.initiator.address)} dropped voice
                {log.id} at x: {log.position[0]?.x.toFixed(3)} y: {log.position[0]?.y.toFixed(
                    3,
                )} at {new Date(
                    Number(log.timestamp / BigInt(1_000_000)),
                ).toLocaleString("en-US", {
                    month: "long",
                    day: "numeric",
                    year: "numeric",
                    hour: "2-digit",
                    minute: "2-digit",
                })}
            </p>
        {:else if "Censor" in log.action}
            {abbreviateWalletAddress(log.initiator.address)} censored voice {log.id}
            at x: {log.position[0]?.x.toFixed(3)} y:
            {log.position[0]?.y.toFixed(3)} at {new Date(
                Number(log.timestamp / BigInt(1_000_000)),
            ).toLocaleString("en-US", {
                month: "long",
                day: "numeric",
                year: "numeric",
                hour: "2-digit",
                minute: "2-digit",
            })}
        {/if}
    {/each}
</div>
