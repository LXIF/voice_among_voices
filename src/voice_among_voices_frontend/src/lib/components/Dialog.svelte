<script lang="ts">
    import { onMount } from "svelte";
    import { scale, fly } from "svelte/transition";
    import { type HTMLAttributes } from "svelte/elements";
    import { nonNullish } from "@dfinity/utils";
    import Button from "./Button.svelte";

    type Props = HTMLAttributes<HTMLDialogElement> & {
        onClose?: () => void;
        title?: string;
        closeOnOutsideClick?: boolean;
        showCloseButton?: boolean;
        backdrop?: boolean;
        alwaysOnBottom?: boolean;
    };

    const {
        children,
        onClose,
        title,
        class: className,
        closeOnOutsideClick = true,
        showCloseButton = true,
        backdrop = true,
        alwaysOnBottom = false,
        ...props
    }: Props = $props();

    let dialogRef: HTMLDialogElement;
    let onCancel = (e: Event): void => {
        e.preventDefault();
        onClose?.();
    };

    const transitionFn = $derived(
        window.innerWidth < 480
            ? (node: Element) => fly(node, { duration: 200, y: "100%" })
            : (node: Element) => scale(node, { duration: 200, start: 0.9 }),
    );

    const fadeOutBackDrop = () => {
        dialogRef.removeAttribute("data-visible");
    };

    onMount(() => {
        dialogRef.showModal();
        dialogRef.setAttribute("data-visible", "true");

        // Use the virtualKeyboard API to intentionally render the software keyboard
        // on top of the page, we manually adjust the dialog positioning for it.
        //
        // If the API is not supported (e.g. iOS) polyfill it with visualViewport.
        let visualViewportResizeTimeout: ReturnType<typeof setTimeout>;
        const updateKeyboardInset = () => {
            clearTimeout(visualViewportResizeTimeout);
            visualViewportResizeTimeout = setTimeout(() => {
                dialogRef.style.setProperty(
                    "--keyboard-inset-height",
                    `${Math.max(window.innerHeight - window.visualViewport!.height, 0)}px`,
                );
            }, 100);
        };
        if ("virtualKeyboard" in navigator) {
            (
                navigator.virtualKeyboard as { overlaysContent: boolean }
            ).overlaysContent = true;
        } else {
            window.visualViewport?.addEventListener(
                "resize",
                updateKeyboardInset,
            );
            window.visualViewport?.addEventListener(
                "scroll",
                updateKeyboardInset,
            );
        }
        return () => {
            if ("virtualKeyboard" in navigator) {
                (
                    navigator.virtualKeyboard as { overlaysContent: boolean }
                ).overlaysContent = false;
            } else {
                window.visualViewport?.removeEventListener(
                    "resize",
                    updateKeyboardInset,
                );
                window.visualViewport?.removeEventListener(
                    "scroll",
                    updateKeyboardInset,
                );
                updateKeyboardInset();
            }
        };
    });
</script>

<dialog
    bind:this={dialogRef}
    oncancel={onCancel}
    closedby={closeOnOutsideClick ? "any" : "none"}
    class={[
        // Layout base/dialog/bottomsheet
        "fixed flex max-w-full flex-col overflow-hidden bg-transparent",
        alwaysOnBottom
            ? "bottom-0 top-auto w-full"
            : "sm:w-100 w-full max-sm:bottom-0 max-sm:top-auto sm:m-auto sm:min-h-[100dvh]",
        // Backdrop base/visible
        "backdrop:bg-bg-overlay backdrop:opacity-0 backdrop:transition-opacity backdrop:duration-200",
        backdrop && "[&[data-visible]]:backdrop:opacity-80",
    ]}
    style="--keyboard-inset-height: env(keyboard-inset-height);"
    transition:transitionFn
    onoutrostart={fadeOutBackDrop}
    {...props}
>
    <div
        class={[
            // Container base/dialog/bottomsheet
            "relative flex max-h-screen flex-col overflow-hidden  bg-slate-50 dark:bg-slate-950",
            alwaysOnBottom
                ? "bottom-0 top-auto w-full rounded-t-2xl border-x border-t px-4 pb-6 pt-4"
                : "sm:w-100 w-full rounded-t-2xl border border-b-0 px-4 pb-6 pt-4 sm:m-auto sm:rounded-2xl sm:border-b sm:px-6 sm:pb-8 sm:pt-6",
            className,
        ]}
    >
        <!-- Non-interactive element to render dark-mode bottom sheet border gradient -->
        <div
            class=" pointer-events-none absolute left-0 right-0 top-0 z-0 hidden rounded-t-2xl p-[1px] max-sm:dark:block"
        >
            <div class="h-24 rounded-t-2xl bg-slate-50 dark:bg-slate-950"></div>
        </div>
        <div
            class="z-1 relative flex flex-1 flex-col text-slate-950 dark:text-slate-50"
        >
            {@render children?.()}
            <!-- Element that pushes bottom sheet away from mobile keyboard or gesture navigation -->
            <div class="flex sm:hidden">
                <div class="h-[var(--keyboard-inset-height)]"></div>
                <div class="h-[env(safe-area-inset-bottom)]"></div>
            </div>
        </div>
        {#if showCloseButton && nonNullish(onClose)}
            <Button
                variant="tertiary"
                size="lg"
                iconOnly
                type="button"
                class="z-2 absolute right-2 top-2 !rounded-full"
                onclick={onClose}
            >
                ✕
            </Button>
        {/if}
    </div>
    <!-- Element that pushes dialog away from mobile keyboard or gesture navigation -->
    <div class="flex max-sm:hidden">
        <div class="h-[var(--keyboard-inset-height)]"></div>
        <div class="h-[env(safe-area-inset-bottom)]"></div>
    </div>
</dialog>
