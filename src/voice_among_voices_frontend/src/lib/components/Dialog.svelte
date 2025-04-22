<script lang="ts">
    import { isTouch } from "$lib/state/isMobile";
    import { onMount } from "svelte";
    import { scale, fly } from "svelte/transition";
    import { type HTMLAttributes } from "svelte/elements";
    import { nonNullish } from "@dfinity/utils";

    type Props = HTMLAttributes<HTMLDialogElement> & {
        onClose?: () => void;
        title?: string;
        closeOnOutsideClick?: boolean;
        showCloseButton?: boolean;
        backdrop?: boolean;
        bottomSheet?: "mobile" | "always" | "never";
    };

    const {
        children,
        onClose,
        title,
        class: className,
        closeOnOutsideClick = true,
        showCloseButton = true,
        backdrop = true,
        bottomSheet = "mobile",
        ...props
    }: Props = $props();

    let dialogRef: HTMLDialogElement;
    let onCancel = (e: Event): void => {
        e.preventDefault();
        onClose?.();
    };

    const transitionFn = $derived(
        ($isTouch && bottomSheet === "mobile") || bottomSheet === "always"
            ? (node: Element) => fly(node, { duration: 200, y: "100%" })
            : (node: Element) => scale(node, { duration: 200, start: 0.9 }),
    );

    const fadeOutBackDrop = () => {
        dialogRef.removeAttribute("data-visible");
    };

    onMount(() => {
        dialogRef.showModal();
        dialogRef.setAttribute("data-visible", "true");
    });
</script>

<div class="fixed left-0 top-0 z-20 h-screen w-screen backdrop-blur-lg"></div>
<dialog
    bind:this={dialogRef}
    oncancel={onCancel}
    closedby={closeOnOutsideClick ? "any" : "none"}
    class={[
        "flex max-h-screen max-w-full flex-col overflow-hidden bg-transparent bg-white text-black backdrop:opacity-0 backdrop:backdrop-brightness-75 backdrop:transition-opacity backdrop:duration-200 max-[460px]:min-w-full dark:bg-slate-950 dark:text-gray-100",
        backdrop && "[&[data-visible]]:backdrop:opacity-100",
        ($isTouch && bottomSheet === "mobile") || bottomSheet === "always"
            ? "mx-auto mt-auto"
            : "m-auto max-[460px]:m-0 max-[460px]:min-h-full",
    ]}
    transition:transitionFn|global
    onoutrostart={fadeOutBackDrop}
    {...props}
>
    <div
        class={[
            "w-100 flex max-h-screen flex-col overflow-hidden bg-slate-100 p-6 max-[460px]:min-w-full max-[460px]:max-w-full dark:bg-slate-900",
            ($isTouch && bottomSheet === "mobile") || bottomSheet === "always"
                ? "fixed bottom-0 max-w-full rounded-t-2xl"
                : "rounded-2xl max-[460px]:flex-1 max-[460px]:rounded-none",
            className,
        ]}
    >
        <div class="flex">
            {#if nonNullish(title)}
                <h1 class="h1 -mt-1 mb-4 flex-1 items-center text-2xl">
                    {title}
                </h1>
            {/if}
            {#if showCloseButton && nonNullish(onClose)}
                <button type="button" class="rounded-full" onclick={onClose}
                    >✕</button
                >
            {/if}
        </div>
        <div class="flex flex-1 flex-col overflow-y-auto overflow-x-hidden">
            {@render children?.()}
        </div>
    </div>
</dialog>
