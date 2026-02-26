<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onboardingOpen } from "../stores/app";
    import { onMount } from "svelte";

    type AppConfig = {
        onboarding_played: boolean;
        launch_at_startup: boolean;
        volume_normalization: boolean;
        autoplay: boolean;
        normalize_by_album: boolean;
        discord_rpc: boolean;
        online_requests: boolean;
        automatic_updates: boolean;
        server_url: string;
    };

    let launchAtStartupEnabled = $state(false);
    let onboardingPlayed = $state(false);
    let volumeNormalizationEnabled = $state(false);
    let autoplayEnabled = $state(true);
    let normalizeByAlbumEnabled = $state(false);
    let discordRpcEnabled = $state(true);
    let onlineRequestsEnabled = $state(true);
    let autoUpdateEnabled = $state(true);
    let serverUrl = $state("https://example.com");

    let isConfigReady = false;
    let saveTimer: ReturnType<typeof setTimeout> | null = null;

    function buildConfig(): AppConfig {
        return {
            onboarding_played: onboardingPlayed,
            launch_at_startup: launchAtStartupEnabled,
            volume_normalization: volumeNormalizationEnabled,
            autoplay: autoplayEnabled,
            normalize_by_album: normalizeByAlbumEnabled,
            discord_rpc: discordRpcEnabled,
            online_requests: onlineRequestsEnabled,
            automatic_updates: autoUpdateEnabled,
            server_url: serverUrl,
        };
    }

    function applyConfig(config: AppConfig) {
        onboardingPlayed = config.onboarding_played;
        launchAtStartupEnabled = config.launch_at_startup;
        volumeNormalizationEnabled = config.volume_normalization;
        autoplayEnabled = config.autoplay;
        normalizeByAlbumEnabled = config.normalize_by_album;
        discordRpcEnabled = config.discord_rpc;
        onlineRequestsEnabled = config.online_requests;
        autoUpdateEnabled = config.automatic_updates;
        serverUrl = config.server_url;
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

    function openOnboarding() {
        onboardingOpen.set(true);
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

        return () => {
            if (saveTimer) clearTimeout(saveTimer);
        };
    });
</script>

<div class="w-full">
    <div class="pb-4">
        <h1 class="text-2xl font-semibold text-white">Settings</h1>
    </div>

    <div class="w-full max-w-[760px] space-y-4 pb-8">
        <section>
            <p class="mb-2 text-sm text-secondary">General</p>
            <div
                class="border border-border rounded-xl overflow-hidden bg-[#0d0d0d]"
            >
                <div class="setting-item px-4 py-3.5 border-b border-border">
                    <div class="flex items-center justify-between gap-4">
                        <div class="flex-1 mr-4">
                            <p
                                class="text-sm leading-5 font-medium text-white mb-1"
                            >
                                Launch at Startup
                            </p>
                            <p class="text-sm leading-5 text-secondary">
                                Start Rift automatically when you sign in
                            </p>
                        </div>
                        <label class="checkbox-container">
                            <input
                                type="checkbox"
                                class="checkbox-input"
                                bind:checked={launchAtStartupEnabled}
                                onchange={queuePersist}
                            />
                            <span class="checkbox-slider"></span>
                        </label>
                    </div>
                </div>
                <div class="setting-item px-4 py-3.5 border-b border-border">
                    <div class="flex items-center justify-between gap-4">
                        <div class="flex-1 mr-4">
                            <p
                                class="text-sm leading-5 font-medium text-white mb-1"
                            >
                                Automatic Updates
                            </p>
                            <p class="text-sm leading-5 text-secondary">
                                Check and install app updates automatically
                            </p>
                        </div>
                        <label class="checkbox-container">
                            <input
                                type="checkbox"
                                class="checkbox-input"
                                bind:checked={autoUpdateEnabled}
                                disabled={!onlineRequestsEnabled}
                                onchange={queuePersist}
                            />
                            <span class="checkbox-slider"></span>
                        </label>
                    </div>
                </div>
            </div>
        </section>

        <section>
            <p class="mb-2 text-sm text-secondary">Audio</p>
            <div
                class="border border-border rounded-xl overflow-hidden bg-[#0d0d0d]"
            >
                <div class="setting-item px-4 py-3.5 border-b border-border">
                    <div class="flex items-center justify-between gap-4">
                        <div class="flex-1 mr-4">
                            <p
                                class="text-sm leading-5 font-medium text-white mb-1"
                            >
                                Volume Normalization
                            </p>
                            <p class="text-sm leading-5 text-secondary">
                                Automatically adjust all tracks to consistent
                                volume levels
                            </p>
                        </div>
                        <label class="checkbox-container">
                            <input
                                type="checkbox"
                                class="checkbox-input"
                                bind:checked={volumeNormalizationEnabled}
                                onchange={queuePersist}
                            />
                            <span class="checkbox-slider"></span>
                        </label>
                    </div>
                </div>
                <div class="setting-item px-4 py-3.5 border-b border-border">
                    <div class="flex items-center justify-between gap-4">
                        <div class="flex-1 mr-4">
                            <p
                                class="text-sm leading-5 font-medium text-white mb-1"
                            >
                                Autoplay
                            </p>
                            <p class="text-sm leading-5 text-secondary">
                                Automatically play recommended tracks after
                                queue ends
                            </p>
                        </div>
                        <label class="checkbox-container">
                            <input
                                type="checkbox"
                                class="checkbox-input"
                                bind:checked={autoplayEnabled}
                                onchange={queuePersist}
                            />
                            <span class="checkbox-slider"></span>
                        </label>
                    </div>
                </div>
                <div class="setting-item px-4 py-3.5">
                    <div class="flex items-center justify-between gap-4">
                        <div class="flex-1 mr-4">
                            <p
                                class="text-sm leading-5 font-medium text-white mb-1"
                            >
                                Normalize by Album
                            </p>
                            <p class="text-sm leading-5 text-secondary">
                                Keep volume differences inside albums while
                                normalizing overall loudness
                            </p>
                        </div>
                        <label class="checkbox-container">
                            <input
                                type="checkbox"
                                class="checkbox-input"
                                bind:checked={normalizeByAlbumEnabled}
                                onchange={queuePersist}
                            />
                            <span class="checkbox-slider"></span>
                        </label>
                    </div>
                </div>
            </div>
        </section>

        <section>
            <p class="mb-2 text-sm text-secondary">Privacy</p>
            <div
                class="border border-border rounded-xl overflow-hidden bg-[#0d0d0d]"
            >
                <div class="setting-item px-4 py-3.5">
                    <div class="flex items-center justify-between gap-4">
                        <div class="flex-1 mr-4">
                            <p
                                class="text-sm leading-5 font-medium text-white mb-1"
                            >
                                Discord Rich Presence
                            </p>
                            <p class="text-sm leading-5 text-secondary">
                                Share your listening activity on Discord profile
                            </p>
                        </div>
                        <label class="checkbox-container">
                            <input
                                type="checkbox"
                                class="checkbox-input"
                                bind:checked={discordRpcEnabled}
                                onchange={queuePersist}
                            />
                            <span class="checkbox-slider"></span>
                        </label>
                    </div>
                </div>
            </div>
        </section>

        <section>
            <p class="mb-2 text-sm text-secondary">Network</p>
            <div
                class="border border-border rounded-xl overflow-hidden bg-[#0d0d0d]"
            >
                <div class="setting-item px-4 py-3.5 border-b border-border">
                    <div class="flex items-center justify-between gap-4">
                        <div class="flex-1 mr-4">
                            <p
                                class="text-sm leading-5 font-medium text-white mb-1"
                            >
                                Online Requests
                            </p>
                            <p class="text-sm leading-5 text-secondary">
                                Allow the app to send requests to external
                                servers
                            </p>
                        </div>
                        <label class="checkbox-container">
                            <input
                                type="checkbox"
                                class="checkbox-input"
                                bind:checked={onlineRequestsEnabled}
                                onchange={queuePersist}
                            />
                            <span class="checkbox-slider"></span>
                        </label>
                    </div>
                </div>
                <div class="setting-item px-4 py-3.5">
                    <div class="flex items-center justify-between gap-4">
                        <div class="flex-1 mr-4">
                            <p
                                class="text-sm leading-5 font-medium text-white mb-1"
                            >
                                Server URL
                            </p>
                            <p class="text-sm leading-5 text-secondary">
                                Do not change this unless you know what you are
                                doing
                            </p>
                        </div>
                        <input
                            type="text"
                            bind:value={serverUrl}
                            oninput={queuePersist}
                            class="h-9 w-64 rounded-lg border border-border bg-hover px-3 text-sm text-white placeholder-secondary outline-none"
                        />
                    </div>
                </div>
            </div>
        </section>

        <section>
            <p class="mb-2 text-sm text-secondary">Support</p>
            <div
                class="border border-border rounded-xl overflow-hidden bg-[#0d0d0d]"
            >
                <div class="setting-item px-4 py-3.5">
                    <div class="flex items-center justify-between gap-4">
                        <div class="min-w-0">
                            <p class="text-sm font-medium text-white">
                                Onboarding
                            </p>
                            <p class="text-xs text-secondary">
                                Replay onboarding and tips
                            </p>
                        </div>
                        <button
                            type="button"
                            class="h-8 rounded-md border border-border bg-hover px-3 text-xs text-white hover:bg-[#252525] [transition:background-color_0.2s_ease]"
                            onclick={openOnboarding}
                        >
                            Replay
                        </button>
                    </div>
                </div>
            </div>
        </section>
    </div>
</div>
