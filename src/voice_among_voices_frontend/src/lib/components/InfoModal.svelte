<script lang="ts">
    import { showInfoModal } from "$lib/state/uxState";
    import { tick } from "svelte";
    import Button from "./Button.svelte";
    import Dialog from "./Dialog.svelte";
    import LogsDisplay from "./LogsDisplay.svelte";
    import { slide } from "svelte/transition";

    type SelectedState = "info" | "log";
    let selected = $state<SelectedState>("info");

    // Accordion data
    let sections = [
        {
            label: "A",
            title: "Amet dolor sit",
            content: "",
            children: [],
        },
        {
            label: "B",
            title: "Bamet",
            content: "",
            children: [
                { title: "Bolor sit amet", content: "" },
                { title: "Blrem ipsum dolor", content: "" },
                {
                    title: "Büber die machenden",
                    content:
                        "Büber die machenden. Über die machenden. Über die machenden. Über die machenden. Über die machenden. Über die machenden. Über die machenden. Über die machenden. Über die machenden. Über die machenden. Über die machenden. Über die machenden.",
                },
            ],
        },
        {
            label: "C",
            title: "Camet",
            content: "",
            children: [{ title: "Color sit amet", content: "" }],
        },
        {
            label: "D",
            title: "Damet",
            content: "",
            children: [],
        },
    ];

    // Track open sections by index
    let openSections = $state<Record<string, boolean>>({});

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
</script>

<Dialog
    class="max-h-[50vh] text-slate-950 dark:text-white"
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
        <div class="accordion">
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
                                    <div class="border-t border-slate-300 pl-8">
                                        <div
                                            class="flex cursor-pointer items-center py-2 font-semibold"
                                            on:click={() =>
                                                toggleChild(section.label, j)}
                                        >
                                            <span class="flex-1"
                                                >{child.title}</span
                                            >
                                            <span class="select-none text-xl">
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
                                                {child.content}
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
