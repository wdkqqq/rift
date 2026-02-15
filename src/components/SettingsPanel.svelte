<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { settingsPanelOpen, activeSettingsTab } from "../stores/app.ts";
    import { onMount } from "svelte";
    type AppConfig = {
        volume_normalization: boolean;
        autoplay: boolean;
        crossfade: boolean;
        gapless_playback: boolean;
        normalize_by_album: boolean;
        discord_rpc: boolean;
        online_requests: boolean;
        automatic_updates: boolean;
        plausible_analytics: boolean;
        dark_theme: boolean;
        native_decorations: boolean;
    };

    let volumeNormalizationEnabled = false;
    let autoplayEnabled = true;
    let crossfadeEnabled = false;
    let gaplessPlaybackEnabled = true;
    let normalizeByAlbumEnabled = false;
    let discordRpcEnabled = true;
    let onlineRequestsEnabled = true;
    let autoUpdateEnabled = true;
    let plausibleAnalyticsEnabled = true;
    let darkThemeEnabled = true;
    let nativeDecorationsEnabled = false;

    let currentTab = $activeSettingsTab;
    let isAnimating = false;
    let isConfigReady = false;
    let saveTimer: ReturnType<typeof setTimeout> | null = null;

    function buildConfig(): AppConfig {
        return {
            volume_normalization: volumeNormalizationEnabled,
            autoplay: autoplayEnabled,
            crossfade: crossfadeEnabled,
            gapless_playback: gaplessPlaybackEnabled,
            normalize_by_album: normalizeByAlbumEnabled,
            discord_rpc: discordRpcEnabled,
            online_requests: onlineRequestsEnabled,
            automatic_updates: autoUpdateEnabled,
            plausible_analytics: plausibleAnalyticsEnabled,
            dark_theme: darkThemeEnabled,
            native_decorations: nativeDecorationsEnabled,
        };
    }

    function applyConfig(config: AppConfig) {
        volumeNormalizationEnabled = config.volume_normalization;
        autoplayEnabled = config.autoplay;
        crossfadeEnabled = config.crossfade;
        gaplessPlaybackEnabled = config.gapless_playback;
        normalizeByAlbumEnabled = config.normalize_by_album;
        discordRpcEnabled = config.discord_rpc;
        onlineRequestsEnabled = config.online_requests;
        autoUpdateEnabled = config.automatic_updates;
        plausibleAnalyticsEnabled = config.plausible_analytics;
        darkThemeEnabled = config.dark_theme;
        nativeDecorationsEnabled = config.native_decorations;
    }

    async function persistConfig() {
        try {
            const saved = await invoke<AppConfig>("set_app_config", {
                config: buildConfig(),
            });
            applyConfig(saved);
        } catch (error) {
            console.error("Failed to save config:", error);
        }
    }

    function queuePersist() {
        if (!isConfigReady) return;
        if (saveTimer) clearTimeout(saveTimer);
        saveTimer = setTimeout(() => {
            void persistConfig();
        }, 120);
    }

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
            document
                .querySelectorAll(".settings-content")
                .forEach((content) => {
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
        void (async () => {
            try {
                const config = await invoke<AppConfig>("get_app_config");
                applyConfig(config);
            } catch (error) {
                console.error("Failed to load config:", error);
            } finally {
                isConfigReady = true;
            }
        })();

        const initialContent = document.getElementById(
            `${$activeSettingsTab}-content`,
        );
        if (initialContent) {
            initialContent.classList.add("active");
            initialContent.style.transform = "translateY(0)";
            initialContent.style.opacity = "1";
        }

        return () => {
            if (saveTimer) clearTimeout(saveTimer);
        };
    });
</script>

<div
    class="settings-panel fixed top-0 left-0 right-0 bottom-0 m-auto w-2/5 min-w-170 h-[28rem] bg-background border border-border rounded-2xl box-shadow z-50 duration-300 overflow-hidden {$settingsPanelOpen
        ? 'active'
        : ''}"
>
    <div class="flex h-full flex-col">
        <div class="px-6 py-4 border-b border-border">
            <h2 class="text-base font-semibold text-white">Settings</h2>
        </div>
        <div class="flex flex-1 min-h-0">
            <!-- Sidebar -->
            <div
                class="w-1/3 bg-background p-4 border-r border-border overflow-y-auto"
            >
                <div class="space-y-1">
                    <div
                        class="active:scale-95 [transition:all_0.2s_ease] settings-tab flex items-center p-2.5 rounded-lg active {activeSettingsTab ===
                        'audio'
                            ? 'active bg-hover text-white'
                            : 'text-secondary hover:text-white hover:bg-hover'}"
                        data-settings-tab="audio"
                        on:click={handleTabClick}
                    >
                        <span>Audio</span>
                    </div>
                    <div
                        class="active:scale-95 [transition:all_0.2s_ease] settings-tab flex items-center p-2.5 rounded-lg {activeSettingsTab ===
                        'privacy'
                            ? 'active bg-hover text-white'
                            : 'text-secondary hover:text-white hover:bg-hover'}"
                        data-settings-tab="privacy"
                        on:click={handleTabClick}
                    >
                        <span>Privacy</span>
                    </div>
                    <div
                        class="active:scale-95 [transition:all_0.2s_ease] settings-tab flex items-center p-2.5 rounded-lg {activeSettingsTab ===
                        'appearance'
                            ? 'active bg-hover text-white'
                            : 'text-secondary hover:text-white hover:bg-hover'}"
                        data-settings-tab="appearance"
                        on:click={handleTabClick}
                    >
                        <span>Appearance</span>
                    </div>
                </div>
            </div>
            <!-- Content Area -->
            <div class="w-2/3 px-6 py-5 overflow-y-auto">
                <!-- Audio Settings -->
                <div
                    class="settings-content {activeSettingsTab === 'audio'
                        ? 'active'
                        : ''}"
                    id="audio-content"
                >
                    <h3 class="text-sm font-semibold text-secondary mb-4">
                        Audio
                    </h3>
                    <div
                        class="border border-border rounded-xl overflow-hidden"
                    >
                        <div
                            class="setting-item px-4 py-4 border-b border-border"
                        >
                            <div
                                class="flex items-center justify-between gap-6"
                            >
                                <div class="flex-1 mr-6">
                                    <p class="font-medium text-white mb-1">
                                        Volume Normalization
                                    </p>
                                    <p class="text-sm text-secondary">
                                        Automatically adjust all tracks to
                                        consistent volume levels
                                    </p>
                                </div>
                                <label class="checkbox-container">
                                    <input
                                        type="checkbox"
                                        class="checkbox-input"
                                        bind:checked={
                                            volumeNormalizationEnabled
                                        }
                                        on:change={queuePersist}
                                    />
                                    <span class="checkbox-slider"></span>
                                </label>
                            </div>
                        </div>
                        <div
                            class="setting-item px-4 py-4 border-b border-border"
                        >
                            <div
                                class="flex items-center justify-between gap-6"
                            >
                                <div class="flex-1 mr-6">
                                    <p class="font-medium text-white mb-1">
                                        Autoplay
                                    </p>
                                    <p class="text-sm text-secondary">
                                        Automatically play recommended tracks
                                        after queue ends
                                    </p>
                                </div>
                                <label class="checkbox-container">
                                    <input
                                        type="checkbox"
                                        class="checkbox-input"
                                        bind:checked={autoplayEnabled}
                                        on:change={queuePersist}
                                    />
                                    <span class="checkbox-slider"></span>
                                </label>
                            </div>
                        </div>
                        <div
                            class="setting-item px-4 py-4 border-b border-border"
                        >
                            <div
                                class="flex items-center justify-between gap-6"
                            >
                                <div class="flex-1 mr-6">
                                    <p class="font-medium text-white mb-1">
                                        Crossfade
                                    </p>
                                    <p class="text-sm text-secondary">
                                        Smoothly blend track endings into the
                                        next track
                                    </p>
                                </div>
                                <label class="checkbox-container">
                                    <input
                                        type="checkbox"
                                        class="checkbox-input"
                                        bind:checked={crossfadeEnabled}
                                        on:change={queuePersist}
                                    />
                                    <span class="checkbox-slider"></span>
                                </label>
                            </div>
                        </div>
                        <div
                            class="setting-item px-4 py-4 border-b border-border"
                        >
                            <div
                                class="flex items-center justify-between gap-6"
                            >
                                <div class="flex-1 mr-6">
                                    <p class="font-medium text-white mb-1">
                                        Gapless Playback
                                    </p>
                                    <p class="text-sm text-secondary">
                                        Remove silence between consecutive
                                        tracks when possible
                                    </p>
                                </div>
                                <label class="checkbox-container">
                                    <input
                                        type="checkbox"
                                        class="checkbox-input"
                                        bind:checked={gaplessPlaybackEnabled}
                                        on:change={queuePersist}
                                    />
                                    <span class="checkbox-slider"></span>
                                </label>
                            </div>
                        </div>
                        <div class="setting-item px-4 py-4">
                            <div
                                class="flex items-center justify-between gap-6"
                            >
                                <div class="flex-1 mr-6">
                                    <p class="font-medium text-white mb-1">
                                        Normalize by Album
                                    </p>
                                    <p class="text-sm text-secondary">
                                        Keep volume differences inside albums
                                        while normalizing overall loudness
                                    </p>
                                </div>
                                <label class="checkbox-container">
                                    <input
                                        type="checkbox"
                                        class="checkbox-input"
                                        bind:checked={normalizeByAlbumEnabled}
                                        on:change={queuePersist}
                                    />
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
                    <h3 class="text-sm font-semibold text-secondary mb-4">
                        Privacy
                    </h3>
                    <div class="space-y-4">
                        <div
                            class="border border-border rounded-xl overflow-hidden"
                        >
                            <div class="setting-item px-4 py-4">
                                <div
                                    class="flex items-center justify-between gap-6"
                                >
                                    <div class="flex-1 mr-6">
                                        <p class="font-medium text-white mb-1">
                                            Discord Rich Presence
                                        </p>
                                        <p class="text-sm text-secondary">
                                            Share your listening activity on
                                            Discord profile
                                        </p>
                                    </div>
                                    <label class="checkbox-container">
                                        <input
                                            type="checkbox"
                                            class="checkbox-input"
                                            bind:checked={discordRpcEnabled}
                                            on:change={queuePersist}
                                        />
                                        <span class="checkbox-slider"></span>
                                    </label>
                                </div>
                            </div>
                        </div>

                        <div>
                            <div
                                class="border border-border rounded-xl overflow-hidden"
                            >
                                <div
                                    class="setting-item px-4 py-4 border-b border-border"
                                >
                                    <div
                                        class="flex items-center justify-between gap-6"
                                    >
                                        <div class="flex-1 mr-6">
                                            <p
                                                class="font-medium text-white mb-1"
                                            >
                                                Online Requests
                                            </p>
                                            <p class="text-sm text-secondary">
                                                Allow the app to send any
                                                requests to external servers
                                            </p>
                                        </div>
                                        <label class="checkbox-container">
                                            <input
                                                type="checkbox"
                                                class="checkbox-input"
                                                bind:checked={
                                                    onlineRequestsEnabled
                                                }
                                                on:change={queuePersist}
                                            />
                                            <span class="checkbox-slider"
                                            ></span>
                                        </label>
                                    </div>
                                </div>

                                <div
                                    class="setting-item px-4 py-4 border-b border-border"
                                >
                                    <div
                                        class="flex items-center justify-between gap-6"
                                    >
                                        <div class="flex-1 mr-6">
                                            <p
                                                class="font-medium text-white mb-1"
                                            >
                                                Automatic Updates
                                            </p>
                                            <p class="text-sm text-secondary">
                                                Check and install app updates
                                                automatically
                                            </p>
                                        </div>
                                        <label class="checkbox-container">
                                            <input
                                                type="checkbox"
                                                class="checkbox-input"
                                                bind:checked={autoUpdateEnabled}
                                                disabled={!onlineRequestsEnabled}
                                                on:change={queuePersist}
                                            />
                                            <span class="checkbox-slider"
                                            ></span>
                                        </label>
                                    </div>
                                </div>

                                <div class="setting-item px-4 py-4">
                                    <div
                                        class="flex items-center justify-between gap-6"
                                    >
                                        <div class="flex-1 mr-6">
                                            <p
                                                class="font-medium text-white mb-1"
                                            >
                                                Usage Analytics
                                            </p>
                                            <p class="text-sm text-secondary">
                                                Share anonymous traffic metrics
                                                via Plausible
                                            </p>
                                        </div>
                                        <label class="checkbox-container">
                                            <input
                                                type="checkbox"
                                                class="checkbox-input"
                                                bind:checked={
                                                    plausibleAnalyticsEnabled
                                                }
                                                disabled={!onlineRequestsEnabled}
                                                on:change={queuePersist}
                                            />
                                            <span class="checkbox-slider"
                                            ></span>
                                        </label>
                                    </div>
                                </div>
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
                    <h3 class="text-sm font-semibold text-secondary mb-4">
                        Appearance
                    </h3>
                    <div
                        class="border border-border rounded-xl overflow-hidden"
                    >
                        <div
                            class="setting-item px-4 py-4 border-b border-border"
                        >
                            <div
                                class="flex items-center justify-between gap-6"
                            >
                                <div class="flex-1 mr-6">
                                    <p class="font-medium text-white mb-1">
                                        Dark Theme
                                    </p>
                                    <p class="text-sm text-secondary">
                                        Use dark color scheme throughout the
                                        application
                                    </p>
                                </div>
                                <label class="checkbox-container">
                                    <input
                                        type="checkbox"
                                        class="checkbox-input"
                                        bind:checked={darkThemeEnabled}
                                        on:change={queuePersist}
                                    />
                                    <span class="checkbox-slider"></span>
                                </label>
                            </div>
                        </div>
                        <div class="setting-item px-4 py-4">
                            <div
                                class="flex items-center justify-between gap-6"
                            >
                                <div class="flex-1 mr-6">
                                    <p class="font-medium text-white mb-1">
                                        Native Decorations
                                    </p>
                                    <p class="text-sm text-secondary">
                                        Use native window decorations
                                    </p>
                                </div>
                                <label class="checkbox-container">
                                    <input
                                        type="checkbox"
                                        class="checkbox-input"
                                        bind:checked={nativeDecorationsEnabled}
                                        on:change={queuePersist}
                                    />
                                    <span class="checkbox-slider"></span>
                                </label>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
