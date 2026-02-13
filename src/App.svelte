<script>
    import { onMount } from "svelte";
    import { commandPaletteOpen, settingsPanelOpen } from "./stores/app.ts";

    import "./main.css";

    import Sidebar from "./components/Sidebar.svelte";
    import CommandPalette from "./components/CommandPalette.svelte";
    import SettingsPanel from "./components/SettingsPanel.svelte";
    import PlaylistView from "./components/PlaylistView.svelte";
    import Player from "./components/Player.svelte";

    function isMacOS() {
        const userAgentDataPlatform = navigator.userAgentData?.platform ?? "";
        const platform = navigator.platform ?? "";
        const userAgent = navigator.userAgent ?? "";

        return /mac/i.test(userAgentDataPlatform + platform + userAgent);
    }

    function handleKeydown(e) {
        if (e.ctrlKey && e.key === "f") {
            e.preventDefault();
            commandPaletteOpen.set(true);
        }

        if (e.key === "Escape") {
            commandPaletteOpen.set(false);
            settingsPanelOpen.set(false);
        }
    }

    function goBack() {
        window.history.back();
    }

    function goForward() {
        window.history.forward();
    }

    let isMac = false;

    onMount(() => {
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
    on:click={() => {
        commandPaletteOpen.set(false);
        settingsPanelOpen.set(false);
    }}
/>

<div
    class="app-shell bg-background text-white h-screen flex flex-col overflow-hidden"
>
    {#if isMac}
        <div class="window-decor-strip">
            <div class="window-decor-drag" data-tauri-drag-region></div>
            <div class="window-nav-controls">
                <button
                    type="button"
                    class="window-nav-button"
                    on:click={goBack}
                >
                    ←
                </button>
                <button
                    type="button"
                    class="window-nav-button"
                    on:click={goForward}
                >
                    →
                </button>
            </div>
        </div>
    {/if}

    <div class="app-main flex-1 flex overflow-hidden">
        <Sidebar />

        <div class="flex-1 flex flex-col min-h-0">
            <PlaylistView />

            <Player />
        </div>
    </div>

    <CommandPalette />
    <SettingsPanel />
</div>
