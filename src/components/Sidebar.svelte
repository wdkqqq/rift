<script lang="ts">
  import { commandPaletteOpen, settingsPanelOpen } from "../stores/app";
  import { Search, Menu, Music, Settings, MicVocal } from "lucide-svelte";

  function openCommandPalette() {
    commandPaletteOpen.set(true);
  }

  function openSettingsPanel() {
    settingsPanelOpen.set(true);
  }
</script>

<div
  class="pt-5 shrink-0 flex flex-col items-start justify-between border-r border-divider min-h-screen bg-background relative transition-all group"
>
  <div class="flex flex-col w-full px-4 space-y-1">
    <!-- Songs Button -->
    <div
      class="active:scale-90 [transition:all_0.2s_ease] group flex items-center space-x-3 p-1 rounded-lg hover:bg-hover"
    >
      <div class="w-10 h-10 flex items-center justify-center relative">
        <div class="absolute inset-0 rounded-lg bg-transparent scale-125"></div>
        <Menu class="w-5 h-5 text-secondary relative z-10" />
      </div>
    </div>

    <!-- Search Button -->
    <div
      on:click={openCommandPalette}
      class="active:scale-90 [transition:all_0.2s_ease] group flex items-center space-x-3 p-1 rounded-lg hover:bg-hover"
    >
      <div class="w-10 h-10 flex items-center justify-center relative">
        <div class="absolute inset-0 rounded-lg bg-transparent scale-125"></div>
        <Search class="w-5 h-5 text-secondary relative z-10" />
      </div>
    </div>

    <!-- Menu Button -->
    <div
      class="active:scale-90 [transition:all_0.2s_ease] group flex items-center space-x-3 p-1 rounded-lg bg-hover"
    >
      <div class="w-10 h-10 flex items-center justify-center relative">
        <div class="absolute inset-0 rounded-lg bg-whit scale-125"></div>
        <Music class="w-5 h-5 text-white relative z-10" />
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
        <div class="absolute inset-0 rounded-lg bg-transparent scale-125"></div>
        <Settings class="w-5 h-5 text-secondary relative z-10" />
      </div>
    </div>
  </div>
</div>
