<script>
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import {
        commandPaletteOpen,
        settingsPanelOpen,
        onboardingOpen,
    } from "./stores/app.ts";

    import "./main.css";

    import Sidebar from "./components/Sidebar.svelte";
    import CommandPalette from "./components/CommandPalette.svelte";
    import SettingsPanel from "./components/SettingsPanel.svelte";
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
            settingsPanelOpen.set(false);
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

        document.addEventListener("keydown", handleKeydown);

        return () => {
            document.body.classList.remove("macos-traffic-lights");
            document.removeEventListener("keydown", handleKeydown);
        };
    });
</script>

<div
    class="fixed inset-0 z-40 transition-all duration-300 {$commandPaletteOpen ||
    $settingsPanelOpen
        ? 'opacity-100 bg-black/50 pointer-events-auto'
        : 'opacity-0 bg-black/0 pointer-events-none'}"
    onclick={() => {
        commandPaletteOpen.set(false);
        settingsPanelOpen.set(false);
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
                        class="pointer-events-none absolute top-full left-1/2 z-30 mt-2 inline-flex -translate-x-1/2 items-center gap-1 whitespace-nowrap rounded-md border border-divider bg-background px-2 py-1 text-xs text-white opacity-0 transition-all duration-200 group-hover:opacity-100"
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
                        class="pointer-events-none absolute top-full left-1/2 z-30 mt-2 inline-flex -translate-x-1/2 items-center gap-1 whitespace-nowrap rounded-md border border-divider bg-background px-2 py-1 text-xs text-white opacity-0 transition-all duration-200 group-hover:opacity-100"
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
    <SettingsPanel />
    {#if $onboardingOpen}
        <OnboardingOverlay />
    {/if}
</div>
