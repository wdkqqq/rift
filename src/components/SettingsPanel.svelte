<script>
  import { settingsPanelOpen, activeSettingsTab } from "../stores/app.ts";
  import { onMount } from "svelte";

  let currentTab = $activeSettingsTab;
  let isAnimating = false;

  function switchTab(tab) {
    if (isAnimating || tab === currentTab) return;

    isAnimating = true;
    activeSettingsTab.set(tab);

    const currentContent = document.getElementById(`${currentTab}-content`);
    if (currentContent) {
      currentContent.style.transform = "translateY(-20px)";
      currentContent.style.opacity = "0";
    }

    setTimeout(() => {
      document
        .querySelectorAll(".settings-tab")
        .forEach((t) => t.classList.remove("active"));
      document.querySelectorAll(".settings-content").forEach((content) => {
        content.classList.remove("active");
        content.style.transform = "translateY(20px)";
        content.style.opacity = "0";
      });

      document
        .querySelector(`[data-settings-tab="${tab}"]`)
        .classList.add("active");

      const newContent = document.getElementById(`${tab}-content`);
      if (newContent) {
        newContent.classList.add("active");
        setTimeout(() => {
          newContent.style.transform = "translateY(0)";
          newContent.style.opacity = "1";
        }, 50);
      }

      currentTab = tab;
      isAnimating = false;
    }, 120);
  }

  function handleTabClick(event) {
    const tab = event.currentTarget.getAttribute("data-settings-tab");
    switchTab(tab);
  }

  onMount(() => {
    const initialContent = document.getElementById(
      `${$activeSettingsTab}-content`,
    );
    if (initialContent) {
      initialContent.classList.add("active");
      initialContent.style.transform = "translateY(0)";
      initialContent.style.opacity = "1";
    }
  });
</script>

<div
  class="settings-panel fixed top-0 left-0 right-0 bottom-0 m-auto w-2/5 min-w-170 h-2/4 max-h-[90vh] bg-background border border-border rounded-2xl box-shadow z-50 duration-300 overflow-y-auto {$settingsPanelOpen
    ? 'active'
    : ''}"
>
  <div class="flex h-full">
    <!-- Sidebar -->
    <div class="w-1/3 bg-background p-4 rounded-l-2xl border-r border-border">
      <div
        class="active:scale-95 [transition:all_0.2s_ease] settings-tab mb-1 flex items-center p-2.5 rounded-lg active {activeSettingsTab ===
        'audio'
          ? 'active bg-hover text-white'
          : 'text-secondary hover:text-white hover:bg-hover'}"
        data-settings-tab="audio"
        on:click={handleTabClick}
      >
        <span>Audio</span>
      </div>
      <div
        class="active:scale-95 [transition:all_0.2s_ease] settings-tab mb-1 flex items-center p-2.5 rounded-lg {activeSettingsTab ===
        'privacy'
          ? 'active bg-hover text-white'
          : 'text-secondary hover:text-white hover:bg-hover'}"
        data-settings-tab="privacy"
        on:click={handleTabClick}
      >
        <span>Privacy</span>
      </div>
      <div
        class="active:scale-95 [transition:all_0.2s_ease] settings-tab mb-1 flex items-center p-2.5 rounded-lg {activeSettingsTab ===
        'appearance'
          ? 'active bg-hover text-white'
          : 'text-secondary hover:text-white hover:bg-hover'}"
        data-settings-tab="appearance"
        on:click={handleTabClick}
      >
        <span>Appearance</span>
      </div>
    </div>
    <!-- Content Area -->
    <div class="w-2/3 p-8 overflow-y-auto">
      <!-- Audio Settings -->
      <div
        class="settings-content {activeSettingsTab === 'audio' ? 'active' : ''}"
        id="audio-content"
      >
        <h3
          class="text-lg font-medium mb-6 text-secondary border-b border-border pb-3"
        >
          Audio Preferences
        </h3>
        <div class="space-y-8">
          <div class="setting-item border-b border-border pb-6">
            <div class="flex items-center justify-between">
              <div class="flex-1 mr-6">
                <p class="font-medium text-white mb-1">Volume Normalization</p>
                <p class="text-sm text-secondary">
                  Automatically adjust all tracks to consistent volume levels
                </p>
              </div>
              <label class="checkbox-container">
                <input type="checkbox" class="checkbox-input" />
                <span class="checkbox-slider"></span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <!-- Privacy Settings -->
      <div
        class="settings-content {activeSettingsTab === 'privacy'
          ? 'active'
          : ''}"
        id="privacy-content"
      >
        <h3
          class="text-lg font-medium mb-6 text-secondary border-b border-border pb-3"
        >
          Privacy
        </h3>
        <div class="space-y-8">
          <div class="setting-item border-b border-border pb-6">
            <div class="flex items-center justify-between">
              <div class="flex-1 mr-6">
                <p class="font-medium text-white mb-1">Discord Rich Presence</p>
                <p class="text-sm text-secondary">
                  Share your listening activity on Discord profile
                </p>
              </div>
              <label class="checkbox-container">
                <input type="checkbox" class="checkbox-input" checked />
                <span class="checkbox-slider"></span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <!-- Appearance Settings -->
      <div
        class="settings-content {activeSettingsTab === 'appearance'
          ? 'active'
          : ''}"
        id="appearance-content"
      >
        <h3
          class="text-lg font-medium mb-6 text-secondary border-b border-border pb-3"
        >
          Appearance
        </h3>
        <div class="space-y-8">
          <div class="setting-item border-b border-border pb-6">
            <div class="flex items-center justify-between">
              <div class="flex-1 mr-6">
                <p class="font-medium text-white mb-1">Dark Theme</p>
                <p class="text-sm text-secondary">
                  Use dark color scheme throughout the application
                </p>
              </div>
              <label class="checkbox-container">
                <input type="checkbox" class="checkbox-input" checked />
                <span class="checkbox-slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-item border-b border-border pb-6">
            <div class="flex items-center justify-between">
              <div class="flex-1 mr-6">
                <p class="font-medium text-white mb-1">Native Decorations</p>
                <p class="text-sm text-[#CCCCCC]">
                  Use native window decorations
                </p>
              </div>
              <label class="checkbox-container">
                <input type="checkbox" class="checkbox-input" />
                <span class="checkbox-slider"></span>
              </label>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
