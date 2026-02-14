<script lang="ts">
    import {
        activeLibraryView,
        commandPaletteOpen,
        favoritesOpenRequest,
        libraryHomeRequest,
        settingsPanelOpen,
    } from "../stores/app";
    import { Search, Menu, Music, Settings } from "lucide-svelte";

    function openCommandPalette() {
        commandPaletteOpen.set(true);
    }

    function openSettingsPanel() {
        settingsPanelOpen.set(true);
    }

    function openLibraryView() {
        if ($activeLibraryView === "library") {
            libraryHomeRequest.update((value) => value + 1);
            return;
        }
        activeLibraryView.set("library");
    }

    function openSongsView() {
        favoritesOpenRequest.update((value) => value + 1);
        activeLibraryView.set("songs");
    }
</script>

<div
    class="pt-5 shrink-0 flex flex-col items-start justify-between border-r border-divider h-full bg-background relative transition-all group"
>
    <div class="flex flex-col w-full px-4 space-y-1">
        <!-- Library Button -->
        <div
            on:click={openLibraryView}
            class="active:scale-90 [transition:all_0.2s_ease] group flex items-center space-x-3 p-1 rounded-lg hover:bg-hover"
            class:bg-hover={$activeLibraryView === "library"}
        >
            <div class="w-10 h-10 flex items-center justify-center relative">
                <div
                    class="absolute inset-0 rounded-lg bg-transparent scale-125"
                ></div>
                <Menu
                    class="w-5 h-5 relative z-10 {$activeLibraryView ===
                    'library'
                        ? 'text-white'
                        : 'text-secondary'}"
                />
            </div>
        </div>

        <!-- Search Button -->
        <div
            on:click={openCommandPalette}
            class="active:scale-90 [transition:all_0.2s_ease] group flex items-center space-x-3 p-1 rounded-lg hover:bg-hover"
        >
            <div class="w-10 h-10 flex items-center justify-center relative">
                <div
                    class="absolute inset-0 rounded-lg bg-transparent scale-125"
                ></div>
                <Search class="w-5 h-5 text-secondary relative z-10" />
            </div>
        </div>

        <!-- Menu Button -->
        <div
            on:click={openSongsView}
            class="active:scale-90 [transition:all_0.2s_ease] group flex items-center space-x-3 p-1 rounded-lg hover:bg-hover"
            class:bg-hover={$activeLibraryView === "songs"}
        >
            <div class="w-10 h-10 flex items-center justify-center relative">
                <div
                    class="absolute inset-0 rounded-lg bg-whit scale-125"
                ></div>
                <Music
                    class="w-5 h-5 relative z-10 {$activeLibraryView === 'songs'
                        ? 'text-white'
                        : 'text-secondary'}"
                />
            </div>
        </div>
    </div>

    <!-- Settings Button -->
    <div class="flex flex-col w-full px-4 space-y-5 pb-5">
        <div
            on:click={openSettingsPanel}
            class="active:scale-90 [transition:all_0.2s_ease] group flex items-center space-x-3 p-1 rounded-lg hover:bg-hover"
        >
            <div class="w-10 h-10 flex items-center justify-center relative">
                <div
                    class="absolute inset-0 rounded-lg bg-transparent scale-125"
                ></div>
                <Settings class="w-5 h-5 text-secondary relative z-10" />
            </div>
        </div>
    </div>
</div>
