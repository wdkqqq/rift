<script>
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { RefreshCw, X } from "lucide-svelte";
    import {
        activeUpdateNotification,
        commandPaletteOpen,
        hideUpdateNotification,
        notifyInfo,
        onboardingOpen,
        showUpdateNotification,
    } from "./stores/app.ts";

    import "./main.css";

    import Sidebar from "./components/Sidebar.svelte";
    import CommandPalette from "./components/CommandPalette.svelte";
    import OnboardingOverlay from "./components/OnboardingOverlay.svelte";
    import PlaylistView from "./components/PlaylistView.svelte";
    import Player from "./components/Player.svelte";

    /** @typedef {{ onboarding_played: boolean }} AppConfig */

    function isMacOS() {
        const userAgentDataPlatform = navigator.userAgentData?.platform ?? "";
        const platform = navigator.platform ?? "";
        const userAgent = navigator.userAgent ?? "";

        return /mac/i.test(userAgentDataPlatform + platform + userAgent);
    }

    function handleKeydown(e) {
        const key = e.key.toLowerCase();
        const shouldOpenSearch =
            (e.metaKey && (key === "f" || key === "k")) ||
            (e.ctrlKey && (key === "f" || key === "k"));
        const shouldGoBack = e.metaKey && key === "[";
        const shouldGoForward = e.metaKey && key === "]";

        if (shouldOpenSearch) {
            e.preventDefault();
            commandPaletteOpen.set(true);
        }

        if (shouldGoBack) {
            e.preventDefault();
            goBack();
        }

        if (shouldGoForward) {
            e.preventDefault();
            goForward();
        }

        if (e.key === "Escape") {
            commandPaletteOpen.set(false);
            onboardingOpen.set(false);
        }
    }

    function goBack() {
        window.history.back();
    }

    function goForward() {
        window.history.forward();
    }

    async function handleTopBarMouseDown(event) {
        if (event.button !== 0) return;

        const target = event.target;
        if (
            target instanceof HTMLElement &&
            target.closest(".window-nav-button")
        ) {
            return;
        }

        try {
            await getCurrentWindow().startDragging();
        } catch (error) {
            console.error("Failed to start window dragging", error);
        }
    }

    function handleUpdateAction() {
        hideUpdateNotification();
        notifyInfo("Update restart flow is not connected yet.");
    }

    let isMac = $state(false);

    onMount(() => {
        void (async () => {
            try {
                /** @type {AppConfig} */
                const config = await invoke("get_app_config");
                if (!config.onboarding_played) {
                    onboardingOpen.set(true);
                }
            } catch (error) {
                console.error("Failed to load config on app startup:", error);
            }
        })();

        isMac = isMacOS();
        if (isMac) {
            document.body.classList.add("macos-traffic-lights");
        }

        // TODO: Re-enable update notification once real auto-update flow is implemented.
        // showUpdateNotification();

        document.addEventListener("keydown", handleKeydown);

        return () => {
            document.body.classList.remove("macos-traffic-lights");
            document.removeEventListener("keydown", handleKeydown);
        };
    });
</script>

<div
    class="fixed inset-0 z-40 transition-all duration-300 {$commandPaletteOpen
        ? 'opacity-100 bg-black/50 pointer-events-auto'
        : 'opacity-0 bg-black/0 pointer-events-none'}"
    onclick={() => {
        commandPaletteOpen.set(false);
        onboardingOpen.set(false);
    }}
></div>

<div
    class="app-shell bg-background text-white h-screen flex flex-col overflow-hidden"
>
    {#if isMac}
        <div class="window-decor-strip" onmousedown={handleTopBarMouseDown}>
            <div class="window-nav-controls">
                <div class="group relative">
                    <button
                        type="button"
                        class="window-nav-button"
                        aria-label="Back"
                        onclick={goBack}
                    >
                        ←
                    </button>
                    <span
                        class="pointer-events-none absolute top-full left-1/2 z-30 mt-2 inline-flex -translate-x-1/2 -translate-y-1 items-center gap-1 whitespace-nowrap rounded-md border border-divider bg-background px-2 py-1 text-xs text-white opacity-0 transition-all duration-200 group-hover:opacity-100 group-hover:translate-y-0 group-focus-within:opacity-100 group-focus-within:translate-y-0"
                    >
                        <span>Back</span>
                        <span class="text-secondary">⌘[</span>
                    </span>
                </div>
                <div class="group relative">
                    <button
                        type="button"
                        class="window-nav-button"
                        aria-label="Forward"
                        onclick={goForward}
                    >
                        →
                    </button>
                    <span
                        class="pointer-events-none absolute top-full left-1/2 z-30 mt-2 inline-flex -translate-x-1/2 -translate-y-1 items-center gap-1 whitespace-nowrap rounded-md border border-divider bg-background px-2 py-1 text-xs text-white opacity-0 transition-all duration-200 group-hover:opacity-100 group-hover:translate-y-0 group-focus-within:opacity-100 group-focus-within:translate-y-0"
                    >
                        <span>Forward</span>
                        <span class="text-secondary">⌘]</span>
                    </span>
                </div>
            </div>
        </div>
    {/if}

    <div class="app-main flex-1 flex overflow-hidden">
        <Sidebar />

        <div class="flex-1 flex flex-col min-h-0 min-w-0">
            <PlaylistView />

            <Player />
        </div>
    </div>

    <CommandPalette />
    {#if $onboardingOpen}
        <OnboardingOverlay />
    {/if}

    {#if $activeUpdateNotification}
        <aside
            class="fixed top-4 right-4 z-[60] w-[20.5rem] max-w-[calc(100vw-1.5rem)] rounded-xl border border-border bg-[#0f0f0f] p-3.5 shadow-[0_10px_24px_rgba(0,0,0,0.35),0_2px_6px_rgba(0,0,0,0.24)]"
            role="status"
            aria-live="polite"
        >
            <div class="flex items-start justify-between gap-2.5">
                <div class="flex min-w-0 items-start gap-2.5">
                    <span
                        class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border bg-hover text-[#d4d4d4]"
                    >
                        <RefreshCw class="h-3.5 w-3.5" />
                    </span>
                    <div class="min-w-0">
                        <p
                            class="text-[13px] font-semibold leading-5 text-white"
                        >
                            {$activeUpdateNotification.title}
                        </p>
                        <p class="mt-0.5 text-[12px] leading-4 text-secondary">
                            {$activeUpdateNotification.message}
                        </p>
                    </div>
                </div>
                <button
                    type="button"
                    class="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-secondary transition-colors duration-200 hover:bg-hover hover:text-white"
                    aria-label="Close update notification"
                    onclick={hideUpdateNotification}
                >
                    <X class="h-3.5 w-3.5" />
                </button>
            </div>

            <div class="mt-3 flex items-center justify-end">
                <button
                    type="button"
                    class="rounded-md bg-[#2a2a2a] px-3 py-1.5 text-[12px] font-medium text-white transition-[background-color,transform] duration-200 hover:bg-[#353535] active:translate-y-[0.5px] active:bg-[#3c3c3c]"
                    onclick={handleUpdateAction}
                >
                    {$activeUpdateNotification.actionLabel}
                </button>
            </div>
        </aside>
    {/if}
</div>
