<script lang="ts">
    import { showInfoModal } from "$lib/state/uxState";
    import Button from "./Button.svelte";
    import Dialog from "./Dialog.svelte";
    import LogsDisplay from "./LogsDisplay.svelte";
    import { slide } from "svelte/transition";
    import { sections } from "$lib/content/abecedaire";

    type SelectedState = "info" | "log";
    let selected = $state<SelectedState>("info");

    // Track open sections by index
    let openSections = $state<Record<string, boolean>>({});

    // Track scroll state for indicator
    let hasScrolled = $state(false);

    function toggleSection(label: string) {
        openSections[label] = !openSections[label];
    }
    function toggleChild(parentLabel: string, idx: number) {
        openSections[`${parentLabel}-${idx}`] =
            !openSections[`${parentLabel}-${idx}`];
    }
    function handleClick(newSelected: SelectedState) {
        selected = newSelected;
    }

    function handleScroll() {
        if (!hasScrolled) {
            hasScrolled = true;
        }
    }
</script>

<Dialog
    class="max-h-[50vh] max-w-full text-slate-950 md:max-w-[80vw] lg:max-w-[60vw] dark:text-white"
    onClose={() => ($showInfoModal = false)}
>
    <div class="mx-2 mb-2 mt-1 flex items-center justify-center gap-2">
        <Button
            onclick={() => handleClick("info")}
            class="{selected === 'info'
                ? 'border border-slate-950 dark:border-slate-50'
                : ''} rounded-full px-4 py-2">Info</Button
        >
        <Button
            onclick={() => handleClick("log")}
            class="{selected === 'log'
                ? 'border border-slate-950 dark:border-slate-50'
                : ''} rounded-full px-4 py-2">Log</Button
        >
    </div>
    {#if selected === "info"}
        <div class="relative">
            <!-- Scroll indicator -->
            {#if !hasScrolled}
                <div
                    class="absolute bottom-0 flex w-full items-center justify-center"
                >
                    <div
                        class="bottom-2 z-10 rounded-full bg-slate-200/80 px-2 py-1 text-xs text-slate-600 dark:bg-slate-700/80 dark:text-slate-300"
                        transition:slide={{ duration: 200 }}
                    >
                        ↓
                    </div>
                </div>
            {/if}

            <div
                class="accordion h-[calc(50vh-4rem)] overflow-y-auto"
                on:scroll={handleScroll}
            >
                {#each sections as section, i}
                    <div class="border-t border-slate-400">
                        <div
                            class="flex cursor-pointer items-center gap-2 py-2 text-lg font-bold"
                            on:click={() => toggleSection(section.label)}
                        >
                            <span class="w-6">{section.label}</span>
                            <span class="flex-1">{section.title}</span>
                            <span class="select-none text-2xl">
                                {openSections[section.label] ? "−" : "+"}
                            </span>
                        </div>
                        {#if openSections[section.label]}
                            <div transition:slide>
                                {#if section.children.length > 0}
                                    {#each section.children as child, j}
                                        <div
                                            class="border-t border-slate-300 pl-8"
                                        >
                                            <div
                                                class="flex cursor-pointer items-center py-2 font-semibold"
                                                on:click={() =>
                                                    toggleChild(
                                                        section.label,
                                                        j,
                                                    )}
                                            >
                                                <span class="flex-1"
                                                    >{child.title}</span
                                                >
                                                <span
                                                    class="select-none text-xl"
                                                >
                                                    {openSections[
                                                        `${section.label}-${j}`
                                                    ]
                                                        ? "−"
                                                        : "+"}
                                                </span>
                                            </div>
                                            {#if openSections[`${section.label}-${j}`]}
                                                <div
                                                    class="pb-2 text-slate-900 dark:text-slate-300"
                                                    transition:slide
                                                >
                                                    {@html child.content}
                                                </div>
                                            {/if}
                                        </div>
                                    {/each}
                                {/if}
                            </div>
                        {/if}
                    </div>
                {/each}
            </div>
        </div>
    {:else if selected === "log"}
        <div class="h-[calc(50vh-4rem)] overflow-hidden">
            <LogsDisplay />
        </div>
    {/if}
</Dialog>

<style>
    .accordion {
        font-family: inherit;
    }
</style>
