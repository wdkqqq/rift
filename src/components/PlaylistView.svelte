<script lang="ts">
    import {
        ChevronLeft,
        ChevronRight,
        Heart,
        Music,
        Pause,
        Play,
    } from "lucide-svelte";
    import { onDestroy, onMount, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { appCacheDir } from "@tauri-apps/api/path";
    import { BaseDirectory, readFile } from "@tauri-apps/plugin-fs";
    import {
        activeLibraryView,
        favoritesOpenRequest,
        playlistsRefreshToken,
        playbackIndex,
        playbackQueue,
    } from "../stores/app";

    type LibrarySong = {
        title: string;
        subtitle: string;
        album: string;
        duration: string;
        cover: string;
        path: string;
    };

    type SongWithCover = LibrarySong & {
        coverUrl: string | null;
    };

    type AlbumGroup = {
        key: string;
        title: string;
        artist: string;
        coverUrl: string | null;
        tracks: SongWithCover[];
    };

    const FAVORITES_SLUG = "favorites-tracks";

    const coverUrlCache = new Map<string, string>();

    let albums: AlbumGroup[] = [];
    let favoritesTracks: SongWithCover[] = [];
    let libraryMode: "home" | "album" = "home";
    let activeAlbumId: string | null = null;
    let isLibraryLoading = false;

    let displayedView: "songs" | "library" | "detail" = "songs";
    let viewStyle =
        "opacity: 1; transform: translateY(0); transition: all 0.3s ease-in-out;";
    let isAnimating = false;
    let queuedView: "songs" | "library" | "detail" | null = null;
    let transitionTimer: ReturnType<typeof setTimeout> | null = null;
    let lastFavoritesRequest = 0;
    let lastPlaylistRefreshToken = 0;
    let hoveredTrackPath: string | null = null;
    let pausedTrackPath: string | null = null;
    let libraryContentStyle =
        "opacity: 1; transform: translateY(0); transition: opacity 240ms ease, transform 240ms ease;";
    let isLibraryAnimating = false;
    let queuedLibraryTarget: {
        mode: "home" | "album";
        albumId: string | null;
    } | null = null;
    let playlistRowElement: HTMLDivElement | null = null;
    let albumRowElement: HTMLDivElement | null = null;

    $: selectedAlbum =
        libraryMode === "album" && activeAlbumId
            ? (albums.find((album) => album.key === activeAlbumId) ?? null)
            : null;
    $: selectedTracks =
        libraryMode === "album"
            ? (selectedAlbum?.tracks ?? [])
            : favoritesTracks;
    $: selectedTitle =
        libraryMode === "album"
            ? (selectedAlbum?.title ?? "Album")
            : "Favorite songs";
    $: selectedSubtitle =
        libraryMode === "album"
            ? `${selectedAlbum?.artist ?? "Unknown Artist"} • ${selectedTracks.length} tracks`
            : `${sumDurations(selectedTracks)} • ${selectedTracks.length} songs`;
    $: selectedCoverUrl =
        libraryMode === "album" ? (selectedAlbum?.coverUrl ?? null) : null;

    function wait(ms: number) {
        return new Promise<void>((resolve) => {
            transitionTimer = setTimeout(resolve, ms);
        });
    }

    function sumDurations(tracks: SongWithCover[]): string {
        let totalSeconds = 0;
        for (const track of tracks) {
            totalSeconds += parseDuration(track.duration);
        }
        const minutes = Math.round(totalSeconds / 60);
        return `${minutes} minutes`;
    }

    function parseDuration(duration: string): number {
        const [mins, secs] = duration.split(":").map((value) => Number(value));
        if (!Number.isFinite(mins) || !Number.isFinite(secs)) return 0;
        return mins * 60 + secs;
    }

    async function animateViewSwitch(nextView: "songs" | "library" | "detail") {
        if (isAnimating) {
            queuedView = nextView;
            return;
        }

        isAnimating = true;
        viewStyle =
            "opacity: 0; transform: translateY(-12px); transition: opacity 150ms ease, transform 150ms ease;";
        await wait(120);

        displayedView = nextView;
        await tick();

        viewStyle =
            "opacity: 0; transform: translateY(10px); transition: none;";
        await tick();

        requestAnimationFrame(() => {
            viewStyle =
                "opacity: 1; transform: translateY(0); transition: opacity 240ms ease, transform 240ms ease;";
        });
        await wait(250);

        isAnimating = false;
        if (queuedView && queuedView !== displayedView) {
            const pendingView = queuedView;
            queuedView = null;
            animateViewSwitch(pendingView);
        } else {
            queuedView = null;
        }
    }

    async function animateLibraryModeSwitch(
        nextMode: "home" | "album",
        albumId: string | null,
    ) {
        if (isLibraryAnimating) {
            queuedLibraryTarget = { mode: nextMode, albumId };
            return;
        }

        if (libraryMode === nextMode && activeAlbumId === albumId) return;

        isLibraryAnimating = true;
        libraryContentStyle =
            "opacity: 0; transform: translateY(-12px); transition: opacity 150ms ease, transform 150ms ease;";
        await wait(120);

        libraryMode = nextMode;
        activeAlbumId = albumId;
        await tick();

        libraryContentStyle =
            "opacity: 0; transform: translateY(10px); transition: none;";
        await tick();

        requestAnimationFrame(() => {
            libraryContentStyle =
                "opacity: 1; transform: translateY(0); transition: opacity 240ms ease, transform 240ms ease;";
        });
        await wait(250);

        isLibraryAnimating = false;
        if (queuedLibraryTarget) {
            const queued = queuedLibraryTarget;
            queuedLibraryTarget = null;
            animateLibraryModeSwitch(queued.mode, queued.albumId);
        }
    }

    async function getCoverUrl(coverFilename: string): Promise<string | null> {
        if (!coverFilename) return null;
        if (coverUrlCache.has(coverFilename))
            return coverUrlCache.get(coverFilename)!;

        try {
            const cacheDir = await appCacheDir();
            const path = `${cacheDir}/covers/${coverFilename}`;
            const data = await readFile(path, {
                dir: BaseDirectory.Cache,
                encoding: null,
            });
            const blob = new Blob([data]);
            const url = URL.createObjectURL(blob);
            coverUrlCache.set(coverFilename, url);
            return url;
        } catch {
            return null;
        }
    }

    function buildAlbums(songs: SongWithCover[]): AlbumGroup[] {
        const map = new Map<string, AlbumGroup>();

        for (const song of songs) {
            const albumTitleRaw = song.album?.trim();
            const hasNamedAlbum =
                !!albumTitleRaw && albumTitleRaw !== "Unknown Album";
            const albumTitle = hasNamedAlbum ? albumTitleRaw : "Unknown Album";
            const artist = song.subtitle?.trim() || "Unknown Artist";
            const key = hasNamedAlbum
                ? `${artist.toLowerCase()}::${albumTitle.toLowerCase()}`
                : song.cover
                  ? `cover::${song.cover}`
                  : `path::${song.path.split(/[\\/]/).slice(0, -1).join("/")}`;
            const existing = map.get(key);

            if (existing) {
                existing.tracks.push(song);
                if (!existing.coverUrl && song.coverUrl) {
                    existing.coverUrl = song.coverUrl;
                }
                continue;
            }

            map.set(key, {
                key,
                title: albumTitle,
                artist,
                coverUrl: song.coverUrl,
                tracks: [song],
            });
        }

        return Array.from(map.values()).sort((a, b) =>
            `${a.artist} ${a.title}`.localeCompare(`${b.artist} ${b.title}`),
        );
    }

    async function loadAlbums() {
        isLibraryLoading = true;
        try {
            const songs = await invoke<LibrarySong[]>("search_music", {
                query: "",
            });
            const withCover = await Promise.all(
                songs.map(async (song) => ({
                    ...song,
                    coverUrl: await getCoverUrl(song.cover),
                })),
            );

            albums = buildAlbums(withCover);
            if (
                libraryMode === "album" &&
                activeAlbumId &&
                !albums.some((x) => x.key === activeAlbumId)
            ) {
                libraryMode = "home";
                activeAlbumId = null;
            }
        } catch {
            albums = [];
        } finally {
            isLibraryLoading = false;
        }
    }

    async function loadFavoriteTracks() {
        try {
            const songs = await invoke<LibrarySong[]>("get_playlist_tracks", {
                playlistSlug: FAVORITES_SLUG,
            });
            favoritesTracks = await Promise.all(
                songs.map(async (song) => ({
                    ...song,
                    coverUrl: await getCoverUrl(song.cover),
                })),
            );
        } catch {
            favoritesTracks = [];
        }
    }

    function openFavoriteTracks() {
        libraryMode = "home";
        activeAlbumId = null;
        activeLibraryView.set("detail");
    }

    function scrollLibraryRow(row: "playlists" | "albums", direction: -1 | 1) {
        const target =
            row === "playlists" ? playlistRowElement : albumRowElement;
        if (!target) return;

        const offset = Math.max(240, Math.round(target.clientWidth * 0.82));
        target.scrollBy({
            left: offset * direction,
            behavior: "smooth",
        });
    }

    function openAlbum(album: AlbumGroup) {
        window.history.pushState(
            { riftView: "library-album", albumId: album.key },
            "",
        );
        animateLibraryModeSwitch("album", album.key);
        activeLibraryView.set("detail");
    }

    function playTrack(trackIndex: number) {
        const queue = selectedTracks.map((track) => ({
            title: track.title,
            subtitle: track.subtitle,
            album: track.album,
            duration: track.duration,
            coverUrl: track.coverUrl,
            path: track.path,
        }));

        playbackQueue.set(queue);
        playbackIndex.set(trackIndex);
    }

    function handlePopState(event: PopStateEvent) {
        const state = event.state;
        if (
            state?.riftView === "library-album" &&
            typeof state.albumId === "string"
        ) {
            activeLibraryView.set("detail");
            animateLibraryModeSwitch("album", state.albumId);
            return;
        }

        if (libraryMode === "album") {
            activeLibraryView.set("library");
            animateLibraryModeSwitch("home", null);
        }
    }

    $: if ($activeLibraryView !== displayedView) {
        animateViewSwitch($activeLibraryView);
    }

    $: if ($activeLibraryView === "library" && libraryMode !== "home") {
        libraryMode = "home";
        activeAlbumId = null;
    }

    $: if ($favoritesOpenRequest !== lastFavoritesRequest) {
        lastFavoritesRequest = $favoritesOpenRequest;
        libraryMode = "home";
        activeAlbumId = null;
    }

    $: if ($playlistsRefreshToken !== lastPlaylistRefreshToken) {
        lastPlaylistRefreshToken = $playlistsRefreshToken;
        void loadFavoriteTracks();
    }

    onMount(() => {
        loadAlbums();
        void loadFavoriteTracks();
        window.addEventListener("popstate", handlePopState);
    });

    onDestroy(() => {
        if (transitionTimer) clearTimeout(transitionTimer);
        window.removeEventListener("popstate", handlePopState);
        for (const url of coverUrlCache.values()) {
            URL.revokeObjectURL(url);
        }
    });
</script>

<div class="flex-1 min-w-0 w-full overflow-auto m-4 ml-0 p-6">
    <div class="playlist-view-switch" style={viewStyle}>
        {#if displayedView === "library"}
            <div class="w-full max-w-6xl space-y-10">
                <div class="space-y-10" style={libraryContentStyle}>
                    {#if libraryMode === "home"}
                        <section>
                            <div class="mb-4 flex items-center justify-between">
                                <h2 class="text-xl font-semibold">Playlists</h2>
                                <div class="flex items-center gap-2">
                                    <button
                                        type="button"
                                        class="h-9 w-9 rounded-lg border border-border bg-surface text-secondary hover:text-white hover:bg-white/15 hover:border-white/60 active:scale-95 active:bg-white/20 flex items-center justify-center [transition:background-color_0.2s_ease,color_0.2s_ease,border-color_0.2s_ease,transform_0.1s_ease]"
                                        aria-label="Scroll playlists left"
                                        on:click={() =>
                                            scrollLibraryRow("playlists", -1)}
                                    >
                                        <ChevronLeft class="h-4 w-4" />
                                    </button>
                                    <button
                                        type="button"
                                        class="h-9 w-9 rounded-lg border border-border bg-surface text-secondary hover:text-white hover:bg-white/15 hover:border-white/60 active:scale-95 active:bg-white/20 flex items-center justify-center [transition:background-color_0.2s_ease,color_0.2s_ease,border-color_0.2s_ease,transform_0.1s_ease]"
                                        aria-label="Scroll playlists right"
                                        on:click={() =>
                                            scrollLibraryRow("playlists", 1)}
                                    >
                                        <ChevronRight class="h-4 w-4" />
                                    </button>
                                </div>
                            </div>

                            <div
                                bind:this={playlistRowElement}
                                class="flex gap-4 overflow-x-auto pb-2 scrollbar-none"
                            >
                                <button
                                    type="button"
                                    class="w-[220px] shrink-0 text-left rounded-xl border border-divider bg-card p-3 hover:bg-hover [transition:all_0.2s_ease]"
                                    on:click={openFavoriteTracks}
                                >
                                    <div
                                        class="mb-2 w-full aspect-square rounded-xl bg-hover flex items-center justify-center"
                                    >
                                        <Heart
                                            fill="currentColor"
                                            class="h-17 w-17 text-tertiary"
                                        />
                                    </div>
                                    <p class="text-sm font-medium text-white">
                                        Favorite songs
                                    </p>
                                    <p class="text-xs text-secondary">
                                        {favoritesTracks.length} songs
                                    </p>
                                </button>
                            </div>
                        </section>

                        {#if albums.length > 0}
                            <section>
                                <div
                                    class="mb-4 flex items-center justify-between"
                                >
                                    <h2 class="text-xl font-semibold">
                                        Albums
                                    </h2>
                                    <div class="flex items-center gap-2">
                                        <button
                                            type="button"
                                            class="h-9 w-9 rounded-lg border border-border bg-surface text-secondary hover:text-white hover:bg-white/15 hover:border-white/60 active:scale-95 active:bg-white/20 flex items-center justify-center [transition:background-color_0.2s_ease,color_0.2s_ease,border-color_0.2s_ease,transform_0.1s_ease]"
                                            aria-label="Scroll albums left"
                                            on:click={() =>
                                                scrollLibraryRow("albums", -1)}
                                        >
                                            <ChevronLeft class="h-4 w-4" />
                                        </button>
                                        <button
                                            type="button"
                                            class="h-9 w-9 rounded-lg border border-border bg-surface text-secondary hover:text-white hover:bg-white/15 hover:border-white/60 active:scale-95 active:bg-white/20 flex items-center justify-center [transition:background-color_0.2s_ease,color_0.2s_ease,border-color_0.2s_ease,transform_0.1s_ease]"
                                            aria-label="Scroll albums right"
                                            on:click={() =>
                                                scrollLibraryRow("albums", 1)}
                                        >
                                            <ChevronRight class="h-4 w-4" />
                                        </button>
                                    </div>
                                </div>
                                <div
                                    bind:this={albumRowElement}
                                    class="flex gap-4 overflow-x-auto pb-2 scrollbar-none"
                                >
                                    {#each albums as album}
                                        <button
                                            type="button"
                                            class="w-[220px] shrink-0 text-left rounded-xl border border-divider bg-card p-3 hover:bg-hover [transition:all_0.2s_ease]"
                                            on:click={() => openAlbum(album)}
                                        >
                                            <div
                                                class="mb-2 w-full aspect-square rounded-xl bg-hover flex items-center justify-center overflow-hidden"
                                            >
                                                {#if album.coverUrl}
                                                    <img
                                                        src={album.coverUrl}
                                                        alt={album.title}
                                                        class="w-full h-full object-cover"
                                                    />
                                                {:else}
                                                    <Music
                                                        class="h-17 w-17 text-tertiary"
                                                    />
                                                {/if}
                                            </div>
                                            <p
                                                class="text-sm font-medium text-white truncate"
                                            >
                                                {album.title}
                                            </p>
                                            <p
                                                class="text-xs text-secondary truncate"
                                            >
                                                {album.artist} • {album.tracks
                                                    .length} tracks
                                            </p>
                                        </button>
                                    {/each}
                                </div>
                            </section>
                        {/if}
                    {:else}
                        <div>
                            <div class="flex items-end mb-8">
                                <div
                                    class="w-[220px] h-[220px] bg-hover rounded-xl flex items-center justify-center mr-6 overflow-hidden"
                                >
                                    {#if selectedCoverUrl}
                                        <img
                                            src={selectedCoverUrl}
                                            alt={selectedTitle}
                                            class="w-full h-full object-cover"
                                        />
                                    {:else}
                                        <Music
                                            class="h-17 w-17 text-tertiary"
                                        />
                                    {/if}
                                </div>
                                <div>
                                    <p
                                        class="text-sm font-medium text-secondary"
                                    >
                                        Album
                                    </p>
                                    <h1 class="text-7xl font-bold mt-2 mb-4">
                                        {selectedTitle}
                                    </h1>
                                    <p class="text-secondary">
                                        {selectedSubtitle}
                                    </p>
                                </div>
                            </div>

                            <div
                                class="grid grid-cols-12 gap-4 px-4 py-2 text-secondary text-sm border-b border-border mb-2"
                            >
                                <div class="col-span-8 flex items-center">
                                    <div class="w-16 text-center">#</div>
                                    <div class="flex-1">Title</div>
                                </div>
                                <div class="col-span-2">Album</div>
                                <div class="col-span-2 text-right">
                                    Duration
                                </div>
                            </div>

                            <div class="space-y-1">
                                {#each selectedTracks as song, index (song.path)}
                                    <div
                                        class="grid grid-cols-12 gap-4 px-4 py-3 rounded-lg hover:bg-hover group [transition:all_0.1s_ease]"
                                        on:mouseenter={() =>
                                            (hoveredTrackPath = song.path)}
                                        on:mouseleave={() => {
                                            if (
                                                hoveredTrackPath === song.path
                                            ) {
                                                hoveredTrackPath = null;
                                            }
                                        }}
                                    >
                                        <div
                                            class="col-span-8 flex items-center"
                                        >
                                            <div
                                                class="w-16 h-7 relative flex items-center justify-center"
                                            >
                                                {#if $playbackQueue[$playbackIndex]?.path === song.path && pausedTrackPath !== song.path && hoveredTrackPath !== song.path}
                                                    <div
                                                        class="music-eq"
                                                        aria-hidden="true"
                                                    >
                                                        <span></span>
                                                        <span></span>
                                                        <span></span>
                                                    </div>
                                                {:else}
                                                    <span
                                                        class="text-secondary group-hover:hidden"
                                                    >
                                                        {index + 1}
                                                    </span>
                                                {/if}
                                                <button
                                                    type="button"
                                                    class="absolute inset-0 hidden group-hover:flex items-center justify-center text-white"
                                                    aria-label={$playbackQueue[
                                                        $playbackIndex
                                                    ]?.path === song.path &&
                                                    pausedTrackPath !==
                                                        song.path
                                                        ? "Pause track"
                                                        : "Play track"}
                                                    on:click|stopPropagation={() => {
                                                        const isCurrent =
                                                            $playbackQueue[
                                                                $playbackIndex
                                                            ]?.path ===
                                                            song.path;
                                                        if (
                                                            isCurrent &&
                                                            pausedTrackPath !==
                                                                song.path
                                                        ) {
                                                            pausedTrackPath =
                                                                song.path;
                                                            void invoke(
                                                                "playback_pause",
                                                            );
                                                            return;
                                                        }
                                                        if (
                                                            isCurrent &&
                                                            pausedTrackPath ===
                                                                song.path
                                                        ) {
                                                            pausedTrackPath =
                                                                null;
                                                            void invoke(
                                                                "playback_play",
                                                            );
                                                            return;
                                                        }
                                                        pausedTrackPath = null;
                                                        playTrack(index);
                                                        void invoke(
                                                            "playback_load_and_play",
                                                            {
                                                                path: song.path,
                                                            },
                                                        );
                                                    }}
                                                >
                                                    {#if $playbackQueue[$playbackIndex]?.path === song.path && pausedTrackPath !== song.path}
                                                        <Pause
                                                            class="h-4 w-4"
                                                        />
                                                    {:else}
                                                        <Play class="h-4 w-4" />
                                                    {/if}
                                                </button>
                                            </div>
                                            <div
                                                class="flex items-center flex-1"
                                            >
                                                <div
                                                    class="w-11 h-11 bg-surface rounded mr-3 shrink-0 flex items-center justify-center overflow-hidden"
                                                >
                                                    {#if song.coverUrl}
                                                        <img
                                                            src={song.coverUrl}
                                                            alt={song.title}
                                                            class="w-full h-full object-cover"
                                                        />
                                                    {:else}
                                                        <Music
                                                            class="h-5 w-5 text-tertiary"
                                                        />
                                                    {/if}
                                                </div>
                                                <div>
                                                    <div class="text-white">
                                                        {song.title}
                                                    </div>
                                                    <div
                                                        class="text-sm text-secondary"
                                                    >
                                                        {song.subtitle}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                        <div
                                            class="col-span-2 flex items-center text-secondary"
                                        >
                                            {song.album}
                                        </div>
                                        <div
                                            class="col-span-2 flex items-center justify-end text-secondary"
                                        >
                                            {song.duration}
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}
                </div>
            </div>
        {:else}
            <div>
                <div class="flex items-end mb-8">
                    <div
                        class="w-[220px] h-[220px] bg-hover rounded-xl flex items-center justify-center mr-6 overflow-hidden"
                    >
                        {#if selectedCoverUrl}
                            <img
                                src={selectedCoverUrl}
                                alt={selectedTitle}
                                class="w-full h-full object-cover"
                            />
                        {:else if libraryMode === "album"}
                            <Music class="h-17 w-17 text-tertiary" />
                        {:else}
                            <Heart
                                fill="currentColor"
                                class="h-17 w-17 text-tertiary"
                            />
                        {/if}
                    </div>
                    <div>
                        <p class="text-sm font-medium text-secondary">
                            {libraryMode === "album" ? "Album" : "Playlist"}
                        </p>
                        <h1 class="text-7xl font-bold mt-2 mb-4">
                            {selectedTitle}
                        </h1>
                        <p class="text-secondary">{selectedSubtitle}</p>
                    </div>
                </div>

                <div
                    class="grid grid-cols-12 gap-4 px-4 py-2 text-secondary text-sm border-b border-border mb-2"
                >
                    <div class="col-span-8 flex items-center">
                        <div class="w-16 text-center">#</div>
                        <div class="flex-1">Title</div>
                    </div>
                    <div class="col-span-2">Album</div>
                    <div class="col-span-2 text-right">Duration</div>
                </div>

                {#if isLibraryLoading}
                    <div class="space-y-1">
                        {#each Array(8) as _}
                            <div
                                class="grid grid-cols-12 gap-4 px-4 py-3 rounded-lg"
                            >
                                <div class="col-span-8 flex items-center">
                                    <div
                                        class="w-16 flex items-center justify-center"
                                    >
                                        <div
                                            class="h-4 bg-hover rounded w-4 animate-pulse"
                                        ></div>
                                    </div>
                                    <div class="flex items-center flex-1">
                                        <div
                                            class="w-11 h-11 bg-hover rounded mr-3 shrink-0 animate-pulse"
                                        ></div>
                                        <div class="flex-1">
                                            <div
                                                class="h-4 bg-hover rounded w-32 mb-2 animate-pulse"
                                            ></div>
                                            <div
                                                class="h-3 bg-hover rounded w-24 animate-pulse"
                                            ></div>
                                        </div>
                                    </div>
                                </div>
                                <div class="col-span-2 flex items-center">
                                    <div
                                        class="h-3 bg-hover rounded w-20 animate-pulse"
                                    ></div>
                                </div>
                                <div
                                    class="col-span-2 flex items-center justify-end"
                                >
                                    <div
                                        class="h-3 bg-hover rounded w-12 animate-pulse"
                                    ></div>
                                </div>
                            </div>
                        {/each}
                    </div>
                {:else if selectedTracks.length === 0}
                    <div class="px-4 py-8 text-secondary text-sm">
                        No tracks found for this selection.
                    </div>
                {:else}
                    <div class="space-y-1">
                        {#each selectedTracks as song, index (song.path)}
                            <div
                                class="grid grid-cols-12 gap-4 px-4 py-3 rounded-lg hover:bg-hover group [transition:all_0.1s_ease]"
                                on:mouseenter={() =>
                                    (hoveredTrackPath = song.path)}
                                on:mouseleave={() => {
                                    if (hoveredTrackPath === song.path) {
                                        hoveredTrackPath = null;
                                    }
                                }}
                            >
                                <div class="col-span-8 flex items-center">
                                    <div
                                        class="w-16 h-7 relative flex items-center justify-center"
                                    >
                                        {#if $playbackQueue[$playbackIndex]?.path === song.path && pausedTrackPath !== song.path && hoveredTrackPath !== song.path}
                                            <div
                                                class="music-eq"
                                                aria-hidden="true"
                                            >
                                                <span></span>
                                                <span></span>
                                                <span></span>
                                            </div>
                                        {:else}
                                            <span
                                                class="text-secondary group-hover:hidden"
                                            >
                                                {index + 1}
                                            </span>
                                        {/if}
                                        <button
                                            type="button"
                                            class="absolute inset-0 hidden group-hover:flex items-center justify-center text-white"
                                            aria-label={$playbackQueue[
                                                $playbackIndex
                                            ]?.path === song.path &&
                                            pausedTrackPath !== song.path
                                                ? "Pause track"
                                                : "Play track"}
                                            on:click|stopPropagation={() => {
                                                const isCurrent =
                                                    $playbackQueue[
                                                        $playbackIndex
                                                    ]?.path === song.path;
                                                if (
                                                    isCurrent &&
                                                    pausedTrackPath !==
                                                        song.path
                                                ) {
                                                    pausedTrackPath = song.path;
                                                    void invoke(
                                                        "playback_pause",
                                                    );
                                                    return;
                                                }
                                                if (
                                                    isCurrent &&
                                                    pausedTrackPath ===
                                                        song.path
                                                ) {
                                                    pausedTrackPath = null;
                                                    void invoke(
                                                        "playback_play",
                                                    );
                                                    return;
                                                }
                                                pausedTrackPath = null;
                                                playTrack(index);
                                                void invoke(
                                                    "playback_load_and_play",
                                                    {
                                                        path: song.path,
                                                    },
                                                );
                                            }}
                                        >
                                            {#if $playbackQueue[$playbackIndex]?.path === song.path && pausedTrackPath !== song.path}
                                                <Pause class="h-4 w-4" />
                                            {:else}
                                                <Play class="h-4 w-4" />
                                            {/if}
                                        </button>
                                    </div>
                                    <div class="flex items-center flex-1">
                                        <div
                                            class="w-11 h-11 bg-surface rounded mr-3 shrink-0 flex items-center justify-center overflow-hidden"
                                        >
                                            {#if song.coverUrl}
                                                <img
                                                    src={song.coverUrl}
                                                    alt={song.title}
                                                    class="w-full h-full object-cover"
                                                />
                                            {:else}
                                                <Music
                                                    class="h-5 w-5 text-tertiary"
                                                />
                                            {/if}
                                        </div>
                                        <div>
                                            <div class="text-white">
                                                {song.title}
                                            </div>
                                            <div class="text-sm text-secondary">
                                                {song.subtitle}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div
                                    class="col-span-2 flex items-center text-secondary"
                                >
                                    {song.album}
                                </div>
                                <div
                                    class="col-span-2 flex items-center justify-end text-secondary"
                                >
                                    {song.duration}
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        {/if}
    </div>
</div>

<style>
    .music-eq {
        display: inline-flex;
        align-items: flex-end;
        gap: 2px;
        height: 14px;
    }

    .music-eq span {
        width: 3px;
        border-radius: 999px;
        background: currentColor;
        animation: music-eq 0.95s ease-in-out infinite;
    }

    .music-eq span:nth-child(1) {
        animation-delay: -0.25s;
    }

    .music-eq span:nth-child(2) {
        animation-delay: -0.1s;
    }

    .music-eq span:nth-child(3) {
        animation-delay: -0.35s;
    }

    @keyframes music-eq {
        0%,
        100% {
            height: 4px;
            opacity: 0.55;
        }
        50% {
            height: 13px;
            opacity: 1;
        }
    }

    .scrollbar-none {
        scrollbar-width: none;
    }

    .scrollbar-none::-webkit-scrollbar {
        display: none;
    }
</style>
