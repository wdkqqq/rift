<script lang="ts">
    import {
        activeLibraryView,
        commandPaletteOpen,
        pushRiftHistoryState,
        settingsPanelOpen,
    } from "../stores/app";
    import { Search, House, ListMusic, Settings } from "lucide-svelte";

    const isMac =
        typeof navigator !== "undefined" &&
        /mac/i.test(
            (navigator.userAgentData?.platform ?? "") +
                (navigator.platform ?? "") +
                (navigator.userAgent ?? ""),
        );
    const shortcutPrefix = isMac ? "⌘" : "Ctrl+";

    function openCommandPalette() {
        commandPaletteOpen.set(true);
    }

    function openSettingsPanel() {
        settingsPanelOpen.set(true);
    }

    function openLibraryView() {
        pushRiftHistoryState({ riftView: "library-home" });
        activeLibraryView.set("library");
    }

    function openSongsView() {
        pushRiftHistoryState({ riftView: "songs" });
        activeLibraryView.set("songs");
    }
</script>

<div
    class="pt-3 shrink-0 flex flex-col items-start justify-between border-r border-divider h-full bg-background relative transition-all"
>
    <div class="flex flex-col w-full px-4 space-y-1">
        <!-- Library Button -->
        <div class="group relative self-start">
            <div
                onclick={openLibraryView}
                class="flex items-center p-1 rounded-lg hover:bg-hover active:scale-90 [transition:all_0.2s_ease]"
                class:bg-hover={$activeLibraryView === "library"}
            >
                <div
                    class="w-10 h-10 flex items-center justify-center relative"
                >
                    <div
                        class="absolute inset-0 rounded-lg bg-transparent scale-125"
                    ></div>
                    <House
                        class="w-5 h-5 relative z-10 {$activeLibraryView ===
                        'library'
                            ? 'text-white'
                            : 'text-secondary'}"
                    />
                </div>
            </div>
            <span
                class="pointer-events-none absolute top-1/2 left-full ml-2 inline-flex items-center gap-1 whitespace-nowrap rounded-md border border-divider bg-background px-2 py-1 text-xs text-white opacity-0 -translate-x-1 -translate-y-1/2 transition-all duration-200 group-hover:opacity-100 group-hover:translate-x-0 group-focus-within:opacity-100 group-focus-within:translate-x-0 z-30"
            >
                <span>Home</span>
            </span>
        </div>

        <!-- Search Button -->
        <div class="group relative self-start">
            <div
                onclick={openCommandPalette}
                class="flex items-center p-1 rounded-lg hover:bg-hover active:scale-90 [transition:all_0.2s_ease]"
            >
                <div
                    class="w-10 h-10 flex items-center justify-center relative"
                >
                    <div
                        class="absolute inset-0 rounded-lg bg-transparent scale-125"
                    ></div>
                    <Search class="w-5 h-5 text-secondary relative z-10" />
                </div>
            </div>
            <span
                class="pointer-events-none absolute top-1/2 left-full ml-2 inline-flex items-center gap-1 whitespace-nowrap rounded-md border border-divider bg-background px-2 py-1 text-xs text-white opacity-0 -translate-x-1 -translate-y-1/2 transition-all duration-200 group-hover:opacity-100 group-hover:translate-x-0 group-focus-within:opacity-100 group-focus-within:translate-x-0 z-30"
            >
                <span>Search</span>
                <span class="text-secondary">{shortcutPrefix}K</span>
            </span>
        </div>

        <!-- Menu Button -->
        <div class="group relative self-start">
            <div
                onclick={openSongsView}
                class="flex items-center p-1 rounded-lg hover:bg-hover active:scale-90 [transition:all_0.2s_ease]"
                class:bg-hover={$activeLibraryView === "songs"}
            >
                <div
                    class="w-10 h-10 flex items-center justify-center relative"
                >
                    <div
                        class="absolute inset-0 rounded-lg bg-whit scale-125"
                    ></div>
                    <ListMusic
                        class="w-5 h-5 relative z-10 {$activeLibraryView ===
                        'songs'
                            ? 'text-white'
                            : 'text-secondary'}"
                    />
                </div>
            </div>
            <span
                class="pointer-events-none absolute top-1/2 left-full ml-2 inline-flex items-center gap-1 whitespace-nowrap rounded-md border border-divider bg-background px-2 py-1 text-xs text-white opacity-0 -translate-x-1 -translate-y-1/2 transition-all duration-200 group-hover:opacity-100 group-hover:translate-x-0 group-focus-within:opacity-100 group-focus-within:translate-x-0 z-30"
            >
                <span>Library</span>
            </span>
        </div>
    </div>

    <!-- Settings Button -->
    <div class="flex flex-col w-full px-4 space-y-5 pb-3">
        <div class="group relative self-start">
            <div
                onclick={openSettingsPanel}
                class="flex items-center p-1 rounded-lg hover:bg-hover active:scale-90 [transition:all_0.2s_ease]"
            >
                <div
                    class="w-10 h-10 flex items-center justify-center relative"
                >
                    <div
                        class="absolute inset-0 rounded-lg bg-transparent scale-125"
                    ></div>
                    <Settings class="w-5 h-5 text-secondary relative z-10" />
                </div>
            </div>
            <span
                class="pointer-events-none absolute top-1/2 left-full ml-2 inline-flex items-center gap-1 whitespace-nowrap rounded-md border border-divider bg-background px-2 py-1 text-xs text-white opacity-0 -translate-x-1 -translate-y-1/2 transition-all duration-200 group-hover:opacity-100 group-hover:translate-x-0 group-focus-within:opacity-100 group-focus-within:translate-x-0 z-30"
            >
                <span>Settings</span>
            </span>
        </div>
    </div>
</div>
