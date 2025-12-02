<script>
  import { onMount } from "svelte";
  import { commandPaletteOpen, settingsPanelOpen } from "./stores/app.ts";

  import "./main.css";

  import Sidebar from "./components/Sidebar.svelte";
  import CommandPalette from "./components/CommandPalette.svelte";
  import SettingsPanel from "./components/SettingsPanel.svelte";
  import PlaylistView from "./components/PlaylistView.svelte";
  import Player from "./components/Player.svelte";

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

  onMount(() => {
    document.addEventListener("keydown", handleKeydown);

    return () => {
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

<div class="bg-[#0A0A0A] text-white h-screen flex overflow-hidden">
  <Sidebar />

  <div class="flex-1 flex flex-col min-h-0">
    <PlaylistView />

    <Player />
  </div>

  <CommandPalette />
  <SettingsPanel />
</div>
