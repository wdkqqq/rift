<script lang="ts">
    import { run } from "svelte/legacy";

    import { invoke } from "@tauri-apps/api/core";
    import { onDestroy, onMount } from "svelte";
    import { cubicOut } from "svelte/easing";
    import { scale } from "svelte/transition";
    import {
        Shuffle,
        SkipBack,
        Play,
        Pause,
        SkipForward,
        Repeat,
        Repeat1,
        PlusCircle,
        Search,
        Heart,
        Check,
        Volume2,
        VolumeX,
    } from "lucide-svelte";
    import {
        notifyError,
        notifyInfo,
        notifySuccess,
        type PlaybackSource,
        playbackIndex,
        playbackIsPlaying,
        playbackQueue,
        refreshListeningInsights,
        refreshPlaylists,
    } from "../stores/app";

    type PlaybackState = {
        is_loaded: boolean;
        is_playing: boolean;
        current_time: number;
        duration: number;
        volume: number;
        is_muted: boolean;
    };

    let currentTime = $state(0);
    let duration = $state(0);
    let volume = $state(70);
    let isMuted = $state(false);
    let isPlaying = $state(false);
    let lastLoadedPath: string | null = $state(null);
    let pollTimer: ReturnType<typeof setInterval> | null = null;
    let isSeeking = $state(false);
    let seekPreview = $state(0);
    let sliderValue = $state(0);
    let isAdvancing = false;
    let isSyncing = false;
    let repeatMode: "off" | "all" | "one" = $state("off");
    let playlistPopoverOpen = $state(false);
    let playlistSearch = $state("");
    let playlistPopoverElement: HTMLDivElement | null = $state(null);
    let playlistMemberships = $state(new Set<string>());
    let membershipsLoadedForPath: string | null = $state(null);

    type Playlist = {
        slug: string;
        name: string;
    };

    let playlists: Playlist[] = $state([]);

    async function loadAndPlay(path: string) {
        try {
            const state = await invoke<PlaybackState>(
                "playback_load_and_play",
                {
                    path,
                    source:
                        currentTrack?.source ??
                        ({ kind: "other" } satisfies PlaybackSource),
                },
            );
            lastLoadedPath = path;
            applyState(state);
            refreshListeningInsights();
        } catch (error) {
            console.error("Failed to load track:", error);
            isPlaying = false;
        }
    }

    function applyState(state: PlaybackState) {
        isPlaying = state.is_playing;
        playbackIsPlaying.set(state.is_playing);
        currentTime = state.current_time || 0;
        duration = state.duration || 0;
        volume = Math.round((state.volume || 0) * 100);
        isMuted = state.is_muted;
    }

    async function syncState() {
        if (isSyncing || isSeeking) return;

        if (!currentTrack) {
            isPlaying = false;
            playbackIsPlaying.set(false);
            currentTime = 0;
            duration = 0;
            return;
        }

        isSyncing = true;
        try {
            const state = await invoke<PlaybackState>("playback_get_state");
            applyState(state);
            maybeAdvanceQueue(state);
        } catch (error) {
            console.error("Failed to sync playback state:", error);
        } finally {
            isSyncing = false;
        }
    }

    function maybeAdvanceQueue(state: PlaybackState) {
        if (isAdvancing) return;
        if (state.is_playing) return;
        if (state.duration <= 0) return;
        if (state.current_time < state.duration - 0.05) return;

        if (repeatMode === "one" && currentTrack) {
            isAdvancing = true;
            void loadAndPlay(currentTrack.path).finally(() => {
                isAdvancing = false;
            });
            return;
        }

        if (
            repeatMode === "all" &&
            $playbackQueue.length === 1 &&
            currentTrack
        ) {
            isAdvancing = true;
            void loadAndPlay(currentTrack.path).finally(() => {
                isAdvancing = false;
            });
            return;
        }

        if ($playbackIndex < $playbackQueue.length - 1) {
            isAdvancing = true;
            playbackIndex.set($playbackIndex + 1);
            setTimeout(() => {
                isAdvancing = false;
            }, 100);
            return;
        }

        if (repeatMode === "all" && $playbackQueue.length > 0) {
            isAdvancing = true;
            playbackIndex.set(0);
            setTimeout(() => {
                isAdvancing = false;
            }, 100);
            return;
        }

        void playRandomTrack();
    }

    async function playRandomTrack() {
        try {
            const randomTrack = await invoke<{
                title: string;
                subtitle: string;
                album: string;
                duration: string;
                cover: string;
                path: string;
            } | null>("get_random_track");

            if (!randomTrack) {
                console.warn("No tracks available in library");
                return;
            }

            const track = {
                title: randomTrack.title,
                subtitle: randomTrack.subtitle,
                album: randomTrack.album,
                duration: randomTrack.duration,
                coverUrl: randomTrack.cover ? randomTrack.cover : null,
                path: randomTrack.path,
                source: { kind: "other" } satisfies PlaybackSource,
            };

            playbackQueue.set([track]);
            playbackIndex.set(0);
        } catch (error) {
            console.error("Failed to get random track:", error);
        }
    }

    async function togglePlayPause() {
        if (!currentTrack) return;

        try {
            const state = await invoke<PlaybackState>(
                isPlaying ? "playback_pause" : "playback_play",
            );
            applyState(state);
        } catch (error) {
            console.error("Failed to toggle playback:", error);
        }
    }

    function playPrevious() {
        if ($playbackQueue.length === 0) return;
        if ($playbackIndex > 0) {
            playbackIndex.set($playbackIndex - 1);
        }
    }

    function playNext() {
        if ($playbackQueue.length === 0) return;
        if ($playbackIndex < $playbackQueue.length - 1) {
            playbackIndex.set($playbackIndex + 1);
        }
    }

    function startSeek(event: Event) {
        const target = event.currentTarget as HTMLInputElement;
        seekPreview = Number(target.value);
        isSeeking = true;
    }

    async function commitSeek(event: Event) {
        if (!currentTrack) return;
        const target = event.currentTarget as HTMLInputElement;
        const nextValue = Number(target.value);
        seekPreview = nextValue;

        try {
            const state = await invoke<PlaybackState>("playback_seek", {
                positionSeconds: nextValue,
            });
            applyState(state);
        } catch (error) {
            console.error("Failed to seek playback:", error);
        } finally {
            isSeeking = false;
        }
    }

    async function setVolume(event: Event) {
        const target = event.currentTarget as HTMLInputElement;
        const nextVolume = Number(target.value);
        volume = nextVolume;

        try {
            const state = await invoke<PlaybackState>("playback_set_volume", {
                volume: nextVolume / 100,
            });
            applyState(state);
        } catch (error) {
            console.error("Failed to set volume:", error);
        }
    }

    async function toggleMute() {
        try {
            const state = await invoke<PlaybackState>("playback_toggle_mute");
            applyState(state);
        } catch (error) {
            console.error("Failed to toggle mute:", error);
        }
    }

    function cycleRepeatMode() {
        if (repeatMode === "off") {
            repeatMode = "all";
            return;
        }

        if (repeatMode === "all") {
            repeatMode = "one";
            return;
        }

        repeatMode = "off";
    }

    function formatTime(seconds: number): string {
        if (!Number.isFinite(seconds) || seconds < 0) return "0:00";
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${String(secs).padStart(2, "0")}`;
    }

    function togglePlaylistPopover() {
        playlistPopoverOpen = !playlistPopoverOpen;
        if (!playlistPopoverOpen) {
            playlistSearch = "";
        }
    }

    async function loadPlaylists() {
        try {
            playlists = await invoke<Playlist[]>("get_playlists");
        } catch (error) {
            console.error("Failed to load playlists:", error);
            playlists = [];
        }
    }

    async function handlePlaylistPick(playlist: Playlist) {
        if (!currentTrack) return;
        const isInPlaylist = playlistMemberships.has(playlist.slug);

        try {
            if (isInPlaylist) {
                const removed = await invoke<boolean>(
                    "remove_track_from_playlist",
                    {
                        playlistSlug: playlist.slug,
                        trackPath: currentTrack.path,
                    },
                );
                if (removed) {
                    playlistMemberships = new Set(
                        [...playlistMemberships].filter(
                            (slug) => slug !== playlist.slug,
                        ),
                    );
                    notifySuccess(`Removed from ${playlist.name}`);
                    refreshPlaylists();
                } else {
                    notifyInfo(`Track is not in ${playlist.name}`);
                }
            } else {
                const added = await invoke<boolean>("add_track_to_playlist", {
                    playlistSlug: playlist.slug,
                    trackPath: currentTrack.path,
                });

                if (added) {
                    playlistMemberships = new Set([
                        ...playlistMemberships,
                        playlist.slug,
                    ]);
                    notifySuccess(`Added to ${playlist.name}`);
                    refreshPlaylists();
                } else {
                    notifyInfo(`Track is already in ${playlist.name}`);
                }
            }
        } catch (error) {
            console.error("Failed to add track to playlist:", error);
            notifyError("Failed to update playlist");
        }
    }

    async function loadTrackMemberships(trackPath: string) {
        try {
            const slugs = await invoke<string[]>(
                "get_track_playlist_memberships",
                {
                    trackPath,
                },
            );
            if (currentTrack?.path !== trackPath) return;
            playlistMemberships = new Set(slugs);
        } catch (error) {
            console.error("Failed to load playlist memberships:", error);
            if (currentTrack?.path !== trackPath) return;
            playlistMemberships = new Set();
        }
    }

    function handleGlobalPointerDown(event: MouseEvent) {
        if (!playlistPopoverOpen) return;
        const target = event.target;
        if (!(target instanceof Node)) return;

        if (playlistPopoverElement?.contains(target)) return;
        playlistPopoverOpen = false;
        playlistSearch = "";
    }

    function handleGlobalKeydown(event: KeyboardEvent) {
        if (event.key === "Escape") {
            playlistPopoverOpen = false;
            playlistSearch = "";
        }
    }

    onMount(() => {
        void loadPlaylists();
        pollTimer = setInterval(() => {
            void syncState();
        }, 500);

        document.addEventListener("mousedown", handleGlobalPointerDown);
        document.addEventListener("keydown", handleGlobalKeydown);

        return () => {
            if (pollTimer) clearInterval(pollTimer);
            document.removeEventListener("mousedown", handleGlobalPointerDown);
            document.removeEventListener("keydown", handleGlobalKeydown);
        };
    });

    onDestroy(() => {
        if (pollTimer) clearInterval(pollTimer);
        document.removeEventListener("mousedown", handleGlobalPointerDown);
        document.removeEventListener("keydown", handleGlobalKeydown);
    });
    let currentTrack = $derived($playbackQueue[$playbackIndex] ?? null);
    let effectiveCurrentTime = $derived(isSeeking ? seekPreview : currentTime);
    run(() => {
        if (!isSeeking) {
            sliderValue = effectiveCurrentTime;
        }
    });
    let progressPercent = $derived(
        duration > 0 ? `${(sliderValue / duration) * 100}%` : "0%",
    );
    let volumePercent = $derived(`${volume}%`);
    let filteredPlaylists = $derived(
        playlists.filter((playlist) =>
            playlist.name
                .toLowerCase()
                .includes(playlistSearch.trim().toLowerCase()),
        ),
    );
    run(() => {
        if (currentTrack?.path && currentTrack.path !== lastLoadedPath) {
            void loadAndPlay(currentTrack.path);
        }
    });
    run(() => {
        if (
            currentTrack?.path &&
            currentTrack.path !== membershipsLoadedForPath
        ) {
            membershipsLoadedForPath = currentTrack.path;
            void loadTrackMemberships(currentTrack.path);
        }
    });
    run(() => {
        if (!currentTrack?.path) {
            membershipsLoadedForPath = null;
            playlistMemberships = new Set();
        }
    });
</script>

<div class="p-3 border-t border-divider">
    <div class="flex items-center justify-between pl-2">
        <div class="flex items-center flex-1 min-w-0">
            {#if currentTrack}
                <div
                    class="w-11 h-11 bg-card rounded mr-3 shrink-0 flex items-center justify-center overflow-hidden"
                >
                    {#if currentTrack.coverUrl}
                        <img
                            src={currentTrack.coverUrl}
                            alt={currentTrack.title}
                            class="w-full h-full object-cover"
                        />
                    {:else}
                        <svg
                            class="h-5 w-5 text-tertiary"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
                            />
                        </svg>
                    {/if}
                </div>
                <div class="flex items-center min-w-0">
                    <div class="mr-3 min-w-0">
                        <div class="text-white truncate max-w-56">
                            {currentTrack.title}
                        </div>
                        <div class="text-sm text-secondary truncate max-w-56">
                            {currentTrack.subtitle}
                        </div>
                    </div>
                    <div class="relative" bind:this={playlistPopoverElement}>
                        <button
                            class="p-1 transition-colors [transition:all_0.2s_ease] text-[#818181] hover:text-white"
                            aria-label="Add to playlist"
                            aria-haspopup="menu"
                            aria-expanded={playlistPopoverOpen}
                            onclick={togglePlaylistPopover}
                        >
                            {#if playlistMemberships.size > 0}
                                <span
                                    class="inline-flex h-5 w-5 items-center justify-center rounded-full border border-[var(--color-border)] bg-[var(--color-hover)] text-current"
                                >
                                    <Check class="h-3 w-3" strokeWidth={3} />
                                </span>
                            {:else}
                                <PlusCircle class="h-5 w-5 text-current" />
                            {/if}
                        </button>

                        {#if playlistPopoverOpen}
                            <div
                                class="absolute left-0 bottom-full mb-2 w-64 origin-bottom-left rounded-lg border border-border bg-background [box-shadow:0_12px_28px_rgba(0,0,0,0.35)] p-2 z-30"
                                role="menu"
                                aria-label="Choose playlist"
                                transition:scale={{
                                    duration: 180,
                                    easing: cubicOut,
                                    start: 0.96,
                                }}
                            >
                                <div
                                    class="flex items-center gap-2 rounded-md border border-divider px-2 py-1.5 mb-2"
                                >
                                    <Search
                                        class="h-3.5 w-3.5 text-secondary"
                                    />
                                    <input
                                        type="text"
                                        class="w-full bg-transparent text-sm text-white placeholder:text-[#8a8a8a] focus:outline-none"
                                        placeholder="Search playlists"
                                        bind:value={playlistSearch}
                                    />
                                </div>

                                <div class="max-h-48 overflow-y-auto">
                                    {#if filteredPlaylists.length > 0}
                                        {#each filteredPlaylists as playlist}
                                            <button
                                                type="button"
                                                class="w-full text-left px-2 py-2 rounded-md text-sm text-white hover:bg-white/10 transition-colors flex items-center gap-2"
                                                role="menuitemcheckbox"
                                                aria-checked={playlistMemberships.has(
                                                    playlist.slug,
                                                )}
                                                onclick={() =>
                                                    handlePlaylistPick(
                                                        playlist,
                                                    )}
                                            >
                                                <div
                                                    class="h-8 w-8 rounded bg-hover shrink-0 flex items-center justify-center"
                                                >
                                                    <Heart
                                                        fill="currentColor"
                                                        class="h-4 w-4 text-tertiary"
                                                    />
                                                </div>
                                                <span class="truncate"
                                                    >{playlist.name}</span
                                                >
                                                {#if playlistMemberships.has(playlist.slug)}
                                                    <Check
                                                        class="h-4 w-4 ml-auto text-white"
                                                    />
                                                {/if}
                                            </button>
                                        {/each}
                                    {:else}
                                        <div
                                            class="px-2 py-2 text-sm text-secondary"
                                        >
                                            No playlists found
                                        </div>
                                    {/if}
                                </div>
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}
        </div>

        <div class="flex flex-col items-center space-y-2 w-1/3">
            <div class="flex items-center space-x-6">
                <button
                    class="text-[#818181] transition-colors hover:text-white [transition:all_0.2s_ease] disabled:text-[#5f5f5f] disabled:cursor-not-allowed disabled:hover:text-[#5f5f5f]"
                    disabled={!currentTrack}
                >
                    <Shuffle class="h-5 w-5" />
                </button>

                <button
                    class="text-[#818181] transition-colors hover:text-white [transition:all_0.2s_ease] disabled:text-[#5f5f5f] disabled:cursor-not-allowed disabled:hover:text-[#5f5f5f]"
                    onclick={playPrevious}
                    disabled={!currentTrack}
                >
                    <SkipBack class="h-5 w-5" />
                </button>

                <button
                    class="text-[#818181] p-2 transition-colors [transition:all_0.2s_ease] hover:text-white disabled:text-[#5f5f5f] disabled:cursor-not-allowed disabled:hover:text-[#5f5f5f]"
                    onclick={togglePlayPause}
                    disabled={!currentTrack}
                >
                    {#if isPlaying}
                        <Pause class="h-5 w-5" />
                    {:else}
                        <Play class="h-5 w-5" />
                    {/if}
                </button>

                <button
                    class="text-[#818181] transition-colors hover:text-white [transition:all_0.2s_ease] disabled:text-[#5f5f5f] disabled:cursor-not-allowed disabled:hover:text-[#5f5f5f]"
                    onclick={playNext}
                    disabled={!currentTrack}
                >
                    <SkipForward class="h-5 w-5" />
                </button>

                <button
                    class="transition-colors [transition:all_0.2s_ease] disabled:text-[#5f5f5f] disabled:cursor-not-allowed disabled:hover:text-[#5f5f5f] {repeatMode ===
                    'off'
                        ? 'text-[#818181] hover:text-white'
                        : 'text-white'}"
                    onclick={cycleRepeatMode}
                    disabled={!currentTrack}
                >
                    {#if repeatMode === "one"}
                        <Repeat1 class="h-5 w-5" />
                    {:else}
                        <Repeat class="h-5 w-5" />
                    {/if}
                </button>
            </div>

            <div
                class="flex items-center space-x-2 text-xs text-secondary w-full max-w-md"
            >
                <span class="w-10 text-right shrink-0"
                    >{formatTime(effectiveCurrentTime)}</span
                >
                <div
                    class="group relative flex-1 h-1 bg-[#404040] rounded-full cursor-default"
                >
                    <div
                        class="absolute inset-y-0 left-0 bg-white rounded-full pointer-events-none"
                        style:width={progressPercent}
                    >
                        <div
                            class="absolute right-0 top-1/2 h-3 w-3 -translate-y-1/2 translate-x-1/2 rounded-full bg-white opacity-0 scale-100 transition-all duration-200 ease-out group-hover:opacity-100 group-active:scale-125"
                        ></div>
                    </div>
                    <input
                        class="absolute inset-0 w-full h-full opacity-0 cursor-default"
                        type="range"
                        min="0"
                        max={duration > 0 ? duration : 0}
                        step="0.1"
                        bind:value={sliderValue}
                        oninput={startSeek}
                        onchange={commitSeek}
                    />
                </div>
                <span class="w-10 shrink-0"
                    >{duration > 0
                        ? formatTime(duration)
                        : (currentTrack?.duration ?? "3:35")}</span
                >
            </div>
        </div>

        <div class="flex items-center justify-end space-x-3 w-1/3 pr-2">
            <button
                class="text-[#818181] transition-colors hover:text-white [transition:all_0.2s_ease]"
                onclick={toggleMute}
            >
                {#if isMuted}
                    <VolumeX class="h-5 w-5" />
                {:else}
                    <Volume2 class="h-5 w-5" />
                {/if}
            </button>

            <div
                class="group w-24 relative h-1 bg-[#404040] rounded-full cursor-default"
            >
                <div
                    class="absolute inset-y-0 left-0 bg-white rounded-full pointer-events-none"
                    style:width={volumePercent}
                >
                    <div
                        class="absolute right-0 top-1/2 h-3 w-3 -translate-y-1/2 translate-x-1/2 rounded-full bg-white opacity-0 scale-100 transition-all duration-200 ease-out group-hover:opacity-100 group-active:scale-125"
                    ></div>
                </div>
                <input
                    class="absolute inset-0 w-full h-full opacity-0 cursor-default"
                    type="range"
                    min="0"
                    max="100"
                    step="1"
                    value={volume}
                    oninput={setVolume}
                />
            </div>
        </div>
    </div>
</div>
