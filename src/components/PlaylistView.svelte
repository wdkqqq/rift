<script lang="ts">
    import {
        ChevronLeft,
        ChevronRight,
        Heart,
        Music,
        Pause,
        Play,
        UserRound,
    } from "lucide-svelte";
    import { onDestroy, onMount, tick } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { appCacheDir } from "@tauri-apps/api/path";
    import { BaseDirectory, readFile } from "@tauri-apps/plugin-fs";
    import {
        activeLibraryView,
        favoritesOpenRequest,
        listeningInsightsRefreshToken,
        playlistsRefreshToken,
        playbackIndex,
        playbackQueue,
    } from "../stores/app";

    type LibrarySong = {
        title: string;
        subtitle: string;
        album: string;
        added_at: number;
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

    type ArtistGroup = {
        key: string;
        name: string;
        tracks: number;
        imageFilename: string | null;
        imageUrl: string | null;
    };

    type SongsLibraryTab = "tracks" | "albums" | "playlists";

    type HomeSectionId = "continue" | "recently-added" | "most-played-week";

    type HomeAlbumSection = {
        id: HomeSectionId;
        title: string;
        emptyText: string;
        albums: AlbumGroup[];
    };

    type HomeInsights = {
        continue_listening_paths: string[];
        most_played_week_paths: string[];
    };

    const FAVORITES_SLUG = "favorites-tracks";
    const SONGS_PAGE_SIZE = 10;
    const HOME_SECTION_SIZE = 12;
    const ALBUM_NAME_MAX_CHARS = 28;
    const ALBUM_TITLE_MAX_CHARS = 28;

    const coverUrlCache = new Map<string, string>();
    const artistUrlCache = new Map<string, string>();

    let albums: AlbumGroup[] = [];
    let continueListeningAlbums: AlbumGroup[] = [];
    let recentlyAddedAlbums: AlbumGroup[] = [];
    let mostPlayedWeekAlbums: AlbumGroup[] = [];
    let artists: ArtistGroup[] = [];
    let allSongs: SongWithCover[] = [];
    let favoritesTracks: SongWithCover[] = [];
    let libraryMode: "home" | "album" = "home";
    let activeAlbumId: string | null = null;
    let isLibraryLoading = false;

    let displayedView: "songs" | "library" | "detail" = "library";
    let viewStyle =
        "opacity: 1; transform: translateY(0); transition: all 0.3s ease-in-out;";
    let isAnimating = false;
    let queuedView: "songs" | "library" | "detail" | null = null;
    let transitionTimer: ReturnType<typeof setTimeout> | null = null;
    let lastFavoritesRequest = 0;
    let lastPlaylistRefreshToken = 0;
    let lastListeningInsightsToken = 0;
    let hoveredTrackPath: string | null = null;
    let pausedTrackPath: string | null = null;
    let activeAlbumPlaybackKey: string | null = null;
    let activeAlbumPlaybackPaused = false;
    let libraryContentStyle =
        "opacity: 1; transform: translateY(0); transition: opacity 240ms ease, transform 240ms ease;";
    let isLibraryAnimating = false;
    let queuedLibraryTarget: {
        mode: "home" | "album";
        albumId: string | null;
    } | null = null;
    let sectionRowElements: Record<HomeSectionId, HTMLDivElement | null> = {
        continue: null,
        "recently-added": null,
        "most-played-week": null,
    };
    let artistRowElement: HTMLDivElement | null = null;
    let contentElement: HTMLDivElement | null = null;
    let lastActiveLibraryView: "songs" | "library" | "detail" = "songs";
    let isSongsView = false;
    let visibleSongsCount = SONGS_PAGE_SIZE;
    let songsLibraryTab: SongsLibraryTab = "playlists";
    let displayedSongsLibraryTab: SongsLibraryTab = "playlists";
    let songsTabSlideDirection: 1 | -1 = 1;
    let songsTabContentStyle =
        "opacity: 1; transform: translateX(0); transition: opacity 220ms ease, transform 220ms ease;";
    let songsTabTransitionTimer: ReturnType<typeof setTimeout> | null = null;
    let isSongsTabAnimating = false;
    let queuedSongsTab: SongsLibraryTab | null = null;

    $: selectedAlbum =
        libraryMode === "album" && activeAlbumId
            ? (albums.find((album) => album.key === activeAlbumId) ?? null)
            : null;
    $: isSongsView = displayedView === "songs";
    $: selectedTracks = isSongsView
        ? allSongs.slice(0, visibleSongsCount)
        : libraryMode === "album"
          ? (selectedAlbum?.tracks ?? [])
          : favoritesTracks;
    $: playbackSourceTracks = isSongsView ? allSongs : selectedTracks;
    $: selectedTitle = isSongsView
        ? "Library"
        : libraryMode === "album"
          ? formatAlbumTitle(selectedAlbum?.title ?? "Album")
          : "Favorite Tracks";
    $: selectedSubtitle = isSongsView
        ? `${allSongs.length} songs`
        : libraryMode === "album"
          ? `${selectedAlbum?.artist ?? "Unknown Artist"} • ${selectedTracks.length} tracks`
          : `${sumDurations(selectedTracks)} • ${selectedTracks.length} songs`;
    $: selectedCoverUrl = isSongsView
        ? null
        : libraryMode === "album"
          ? (selectedAlbum?.coverUrl ?? null)
          : null;
    $: if (visibleSongsCount < SONGS_PAGE_SIZE) {
        visibleSongsCount = SONGS_PAGE_SIZE;
    }
    $: if (allSongs.length > 0 && visibleSongsCount > allSongs.length) {
        visibleSongsCount = allSongs.length;
    }
    $: homeAlbumSections = [
        {
            id: "continue",
            title: "Continue listening",
            emptyText: "Start playback to build this section.",
            albums: continueListeningAlbums,
        },
        {
            id: "recently-added",
            title: "Recently added",
            emptyText: "No recently added albums found.",
            albums: recentlyAddedAlbums,
        },
        {
            id: "most-played-week",
            title: "Most played this week",
            emptyText: "No listens in the last 7 days yet.",
            albums: mostPlayedWeekAlbums,
        },
    ] satisfies HomeAlbumSection[];

    function wait(ms: number) {
        return new Promise<void>((resolve) => {
            transitionTimer = setTimeout(resolve, ms);
        });
    }

    function waitSongsTab(ms: number) {
        return new Promise<void>((resolve) => {
            songsTabTransitionTimer = setTimeout(resolve, ms);
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

    function formatAlbumName(album: string): string {
        const normalized = album.trim();
        if (normalized.length <= ALBUM_NAME_MAX_CHARS) return normalized;
        return `${normalized.slice(0, ALBUM_NAME_MAX_CHARS - 1)}...`;
    }

    function formatAlbumTitle(title: string): string {
        const normalized = title.trim();
        if (normalized.length <= ALBUM_TITLE_MAX_CHARS) return normalized;
        return `${normalized.slice(0, ALBUM_TITLE_MAX_CHARS - 1)}...`;
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

    async function getArtistImageUrl(
        imageFilename: string,
    ): Promise<string | null> {
        if (!imageFilename) return null;
        if (artistUrlCache.has(imageFilename))
            return artistUrlCache.get(imageFilename)!;

        try {
            const cacheDir = await appCacheDir();
            const path = `${cacheDir}/artists/${imageFilename}`;
            const data = await readFile(path, {
                dir: BaseDirectory.Cache,
                encoding: null,
            });
            const blob = new Blob([data]);
            const url = URL.createObjectURL(blob);
            artistUrlCache.set(imageFilename, url);
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

    function buildArtists(songs: SongWithCover[]): ArtistGroup[] {
        const map = new Map<string, ArtistGroup>();

        for (const song of songs) {
            const rawArtist = song.subtitle?.trim() || "Unknown Artist";
            const splitArtists = rawArtist
                .split(",")
                .map((name) => name.trim())
                .filter((name) => name.length > 0);
            const names =
                splitArtists.length > 0 ? splitArtists : ["Unknown Artist"];

            for (const name of names) {
                const key = name.toLowerCase();
                const existing = map.get(key);
                if (existing) {
                    existing.tracks += 1;
                    continue;
                }
                map.set(key, {
                    key,
                    name,
                    tracks: 1,
                    imageFilename: null,
                    imageUrl: null,
                });
            }
        }

        return Array.from(map.values()).sort((a, b) => {
            if (b.tracks !== a.tracks) return b.tracks - a.tracks;
            return a.name.localeCompare(b.name);
        });
    }

    async function loadArtists(songs: SongWithCover[]) {
        const base = buildArtists(songs);
        if (base.length === 0) {
            artists = [];
            return;
        }

        try {
            const imageEntries = await invoke<
                Array<{ name: string; image_filename: string | null }>
            >("get_artist_images", {
                artistNames: base.map((artist) => artist.name),
            });
            const byName = new Map(
                imageEntries.map((entry) => [entry.name, entry.image_filename]),
            );

            artists = await Promise.all(
                base.map(async (artist) => {
                    const imageFilename = byName.get(artist.name) ?? null;
                    return {
                        ...artist,
                        imageFilename,
                        imageUrl: imageFilename
                            ? await getArtistImageUrl(imageFilename)
                            : null,
                    };
                }),
            );
        } catch {
            artists = base;
        }
    }

    function pickAlbumsByTrackPaths(
        paths: string[],
        trackAlbumKeys: Map<string, string>,
        albumsByKey: Map<string, AlbumGroup>,
    ): AlbumGroup[] {
        const picked: AlbumGroup[] = [];
        const seen = new Set<string>();

        for (const path of paths) {
            const key = trackAlbumKeys.get(path);
            if (!key || seen.has(key)) continue;
            const album = albumsByKey.get(key);
            if (!album) continue;
            seen.add(key);
            picked.push(album);
            if (picked.length >= HOME_SECTION_SIZE) break;
        }

        return picked;
    }

    function mapTracksToAlbums(albumGroups: AlbumGroup[]) {
        const albumsByKey = new Map(
            albumGroups.map((album) => [album.key, album]),
        );
        const trackAlbumKeys = new Map<string, string>();

        for (const album of albumGroups) {
            for (const track of album.tracks) {
                trackAlbumKeys.set(track.path, album.key);
            }
        }

        return { albumsByKey, trackAlbumKeys };
    }

    async function refreshListeningSections(albumGroups: AlbumGroup[]) {
        if (albumGroups.length === 0) {
            continueListeningAlbums = [];
            mostPlayedWeekAlbums = [];
            return;
        }

        const { albumsByKey, trackAlbumKeys } = mapTracksToAlbums(albumGroups);

        try {
            const insights = await invoke<HomeInsights>("get_home_insights");
            continueListeningAlbums = pickAlbumsByTrackPaths(
                insights.continue_listening_paths,
                trackAlbumKeys,
                albumsByKey,
            );
            mostPlayedWeekAlbums = pickAlbumsByTrackPaths(
                insights.most_played_week_paths,
                trackAlbumKeys,
                albumsByKey,
            );
        } catch {
            continueListeningAlbums = [];
            mostPlayedWeekAlbums = [];
        }
    }

    async function loadHomeSections(
        songs: SongWithCover[],
        albumGroups: AlbumGroup[],
    ) {
        const { albumsByKey, trackAlbumKeys } = mapTracksToAlbums(albumGroups);

        const recentlyAddedPaths = [...songs]
            .sort((a, b) => b.added_at - a.added_at)
            .map((song) => song.path);
        recentlyAddedAlbums = pickAlbumsByTrackPaths(
            recentlyAddedPaths,
            trackAlbumKeys,
            albumsByKey,
        );

        await refreshListeningSections(albumGroups);
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

            allSongs = withCover;
            albums = buildAlbums(withCover);
            await loadHomeSections(withCover, albums);
            await loadArtists(withCover);
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
            allSongs = [];
            continueListeningAlbums = [];
            recentlyAddedAlbums = [];
            mostPlayedWeekAlbums = [];
            artists = [];
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

    function scrollLibraryRow(
        row: HomeSectionId | "artists",
        direction: -1 | 1,
    ) {
        const target =
            row === "artists" ? artistRowElement : sectionRowElements[row];
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

    async function toggleAlbumPlayback(event: MouseEvent, album: AlbumGroup) {
        event.stopPropagation();
        if (album.tracks.length === 0) return;

        const isCurrentAlbum = activeAlbumPlaybackKey === album.key;

        if (isCurrentAlbum) {
            if (activeAlbumPlaybackPaused) {
                activeAlbumPlaybackPaused = false;
                await tick();
                await invoke("playback_play");
                return;
            }
            activeAlbumPlaybackPaused = true;
            await tick();
            await invoke("playback_pause");
            return;
        }

        pausedTrackPath = null;
        playTrack(0, album.tracks, album.key);
        await tick();
        await invoke("playback_load_and_play", {
            path: album.tracks[0].path,
        });
    }

    async function toggleFavoritesPlayback(event: MouseEvent) {
        event.stopPropagation();
        if (favoritesTracks.length === 0) return;

        const isCurrentFavorites = activeAlbumPlaybackKey === FAVORITES_SLUG;
        if (isCurrentFavorites) {
            if (activeAlbumPlaybackPaused) {
                activeAlbumPlaybackPaused = false;
                await tick();
                await invoke("playback_play");
                return;
            }

            activeAlbumPlaybackPaused = true;
            await tick();
            await invoke("playback_pause");
            return;
        }

        pausedTrackPath = null;
        playTrack(0, favoritesTracks, FAVORITES_SLUG);
        await tick();
        await invoke("playback_load_and_play", {
            path: favoritesTracks[0].path,
        });
    }

    function playTrack(
        trackIndex: number,
        sourceTracks: SongWithCover[],
        albumKey: string | null = null,
    ) {
        const queue = sourceTracks.map((track) => ({
            title: track.title,
            subtitle: track.subtitle,
            album: track.album,
            duration: track.duration,
            coverUrl: track.coverUrl,
            path: track.path,
        }));

        playbackQueue.set(queue);
        playbackIndex.set(trackIndex);
        activeAlbumPlaybackKey = albumKey;
        activeAlbumPlaybackPaused = false;
    }

    function getSongsTabIndex(tab: SongsLibraryTab): number {
        if (tab === "albums") return 0;
        if (tab === "tracks") return 1;
        return 2;
    }

    function setSongsLibraryTab(nextTab: SongsLibraryTab) {
        if (songsLibraryTab === nextTab && displayedSongsLibraryTab === nextTab)
            return;
        const currentIndex = getSongsTabIndex(songsLibraryTab);
        const nextIndex = getSongsTabIndex(nextTab);
        songsTabSlideDirection = nextIndex > currentIndex ? 1 : -1;
        void animateSongsTabSwitch(nextTab);
    }

    async function animateSongsTabSwitch(nextTab: SongsLibraryTab) {
        if (!isSongsView) {
            songsLibraryTab = nextTab;
            displayedSongsLibraryTab = nextTab;
            return;
        }

        if (isSongsTabAnimating) {
            queuedSongsTab = nextTab;
            songsLibraryTab = nextTab;
            return;
        }

        if (displayedSongsLibraryTab === nextTab) {
            songsLibraryTab = nextTab;
            return;
        }

        songsLibraryTab = nextTab;
        isSongsTabAnimating = true;
        songsTabContentStyle = `opacity: 0; transform: translateX(${songsTabSlideDirection * -14}px); transition: opacity 110ms ease, transform 110ms ease;`;
        await waitSongsTab(110);

        displayedSongsLibraryTab = nextTab;
        await tick();

        songsTabContentStyle = `opacity: 0; transform: translateX(${songsTabSlideDirection * 14}px); transition: none;`;
        await tick();

        requestAnimationFrame(() => {
            songsTabContentStyle =
                "opacity: 1; transform: translateX(0); transition: opacity 240ms cubic-bezier(0.22, 1, 0.36, 1), transform 240ms cubic-bezier(0.22, 1, 0.36, 1);";
        });
        await waitSongsTab(240);

        isSongsTabAnimating = false;
        if (queuedSongsTab && queuedSongsTab !== displayedSongsLibraryTab) {
            const pending = queuedSongsTab;
            queuedSongsTab = null;
            await animateSongsTabSwitch(pending);
        } else {
            queuedSongsTab = null;
        }
    }

    function handleContentScroll() {
        if (!isSongsView || !contentElement) return;
        if (songsLibraryTab !== "tracks") return;
        if (visibleSongsCount >= allSongs.length) return;

        const isNearBottom =
            contentElement.scrollTop + contentElement.clientHeight >=
            contentElement.scrollHeight - 120;
        if (!isNearBottom) return;

        visibleSongsCount = Math.min(
            allSongs.length,
            visibleSongsCount + SONGS_PAGE_SIZE,
        );
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

    $: if ($activeLibraryView !== lastActiveLibraryView) {
        lastActiveLibraryView = $activeLibraryView;
        if ($activeLibraryView === "songs") {
            visibleSongsCount = SONGS_PAGE_SIZE;
            songsLibraryTab = "playlists";
            displayedSongsLibraryTab = "playlists";
            songsTabSlideDirection = 1;
            isSongsTabAnimating = false;
            queuedSongsTab = null;
            songsTabContentStyle =
                "opacity: 1; transform: translateX(0); transition: opacity 220ms ease, transform 220ms ease;";
            requestAnimationFrame(() => {
                contentElement?.scrollTo({ top: 0 });
            });
        }
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

    $: if ($listeningInsightsRefreshToken !== lastListeningInsightsToken) {
        lastListeningInsightsToken = $listeningInsightsRefreshToken;
        void refreshListeningSections(albums);
    }

    onMount(() => {
        loadAlbums();
        void loadFavoriteTracks();
        window.addEventListener("popstate", handlePopState);
    });

    onDestroy(() => {
        if (transitionTimer) clearTimeout(transitionTimer);
        if (songsTabTransitionTimer) clearTimeout(songsTabTransitionTimer);
        window.removeEventListener("popstate", handlePopState);
        for (const url of coverUrlCache.values()) {
            URL.revokeObjectURL(url);
        }
        for (const url of artistUrlCache.values()) {
            URL.revokeObjectURL(url);
        }
    });
</script>

<div
    bind:this={contentElement}
    on:scroll={handleContentScroll}
    class="flex-1 min-w-0 w-full overflow-auto px-6 py-6"
>
    <div class="playlist-view-switch" style={viewStyle}>
        {#if displayedView === "library"}
            <div class="w-full space-y-10">
                <div class="space-y-10" style={libraryContentStyle}>
                    {#if libraryMode === "home"}
                        {#each homeAlbumSections as section (section.id)}
                            <section>
                                <div
                                    class="mb-4 flex items-center justify-between"
                                >
                                    <h2 class="text-xl font-semibold">
                                        {section.title}
                                    </h2>
                                    <div class="flex items-center gap-2">
                                        <button
                                            type="button"
                                            class="h-9 w-9 rounded-lg border border-border bg-surface text-secondary hover:text-white hover:bg-white/15 hover:border-white/60 active:scale-95 active:bg-white/20 flex items-center justify-center [transition:background-color_0.2s_ease,color_0.2s_ease,border-color_0.2s_ease,transform_0.1s_ease]"
                                            aria-label="Scroll section left"
                                            on:click={() =>
                                                scrollLibraryRow(
                                                    section.id,
                                                    -1,
                                                )}
                                        >
                                            <ChevronLeft class="h-4 w-4" />
                                        </button>
                                        <button
                                            type="button"
                                            class="h-9 w-9 rounded-lg border border-border bg-surface text-secondary hover:text-white hover:bg-white/15 hover:border-white/60 active:scale-95 active:bg-white/20 flex items-center justify-center [transition:background-color_0.2s_ease,color_0.2s_ease,border-color_0.2s_ease,transform_0.1s_ease]"
                                            aria-label="Scroll section right"
                                            on:click={() =>
                                                scrollLibraryRow(section.id, 1)}
                                        >
                                            <ChevronRight class="h-4 w-4" />
                                        </button>
                                    </div>
                                </div>
                                {#if section.albums.length === 0}
                                    <div
                                        class="px-1 py-2 text-sm text-secondary"
                                    >
                                        {section.emptyText}
                                    </div>
                                {:else}
                                    <div
                                        bind:this={
                                            sectionRowElements[section.id]
                                        }
                                        class="flex gap-4 overflow-x-auto pb-2 scrollbar-none -mx-6 px-6"
                                    >
                                        {#each section.albums as album}
                                            <div
                                                class="w-[180px] shrink-0 text-left group"
                                                on:click={() =>
                                                    openAlbum(album)}
                                            >
                                                <div
                                                    class="group/cover relative mb-2 w-full aspect-square rounded-xl bg-hover flex items-center justify-center overflow-hidden [transition:background-color_0.2s_ease]"
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
                                                    <div
                                                        class="pointer-events-none absolute inset-0 bg-black/55 opacity-0 [transition:opacity_0.2s_ease] group-hover/cover:opacity-100"
                                                    ></div>
                                                    <div
                                                        class="pointer-events-none absolute inset-0 opacity-0 [transition:opacity_0.2s_ease] group-hover/cover:opacity-100"
                                                    >
                                                        <button
                                                            type="button"
                                                            class="pointer-events-auto absolute left-3 bottom-3 h-12 w-12 rounded-full bg-white flex items-center justify-center"
                                                            aria-label={activeAlbumPlaybackKey ===
                                                                album.key &&
                                                            !activeAlbumPlaybackPaused
                                                                ? "Pause album"
                                                                : "Play album"}
                                                            on:click={(event) =>
                                                                toggleAlbumPlayback(
                                                                    event,
                                                                    album,
                                                                )}
                                                        >
                                                            {#if activeAlbumPlaybackKey === album.key && !activeAlbumPlaybackPaused}
                                                                <span
                                                                    class="flex items-center gap-[3px]"
                                                                >
                                                                    <span
                                                                        class="h-[12px] w-[4px] rounded-[1px] bg-black/70"
                                                                    ></span>
                                                                    <span
                                                                        class="h-[12px] w-[4px] rounded-[1px] bg-black/70"
                                                                    ></span>
                                                                </span>
                                                            {:else}
                                                                <span
                                                                    class="ml-[2px] h-0 w-0 border-y-[7px] border-y-transparent border-l-[12px] border-l-black/70"
                                                                ></span>
                                                            {/if}
                                                        </button>
                                                    </div>
                                                </div>
                                                <p
                                                    class="text-sm font-medium text-white truncate"
                                                    title={album.title}
                                                >
                                                    {formatAlbumTitle(
                                                        album.title,
                                                    )}
                                                </p>
                                                <p
                                                    class="text-xs text-secondary truncate"
                                                >
                                                    {album.artist} • {album
                                                        .tracks.length} tracks
                                                </p>
                                            </div>
                                        {/each}
                                    </div>
                                {/if}
                            </section>
                        {/each}

                        {#if artists.length > 0}
                            <section>
                                <div
                                    class="mb-4 flex items-center justify-between"
                                >
                                    <h2 class="text-xl font-semibold">
                                        Artists
                                    </h2>
                                    <div class="flex items-center gap-2">
                                        <button
                                            type="button"
                                            class="h-9 w-9 rounded-lg border border-border bg-surface text-secondary hover:text-white hover:bg-white/15 hover:border-white/60 active:scale-95 active:bg-white/20 flex items-center justify-center [transition:background-color_0.2s_ease,color_0.2s_ease,border-color_0.2s_ease,transform_0.1s_ease]"
                                            aria-label="Scroll artists left"
                                            on:click={() =>
                                                scrollLibraryRow("artists", -1)}
                                        >
                                            <ChevronLeft class="h-4 w-4" />
                                        </button>
                                        <button
                                            type="button"
                                            class="h-9 w-9 rounded-lg border border-border bg-surface text-secondary hover:text-white hover:bg-white/15 hover:border-white/60 active:scale-95 active:bg-white/20 flex items-center justify-center [transition:background-color_0.2s_ease,color_0.2s_ease,border-color_0.2s_ease,transform_0.1s_ease]"
                                            aria-label="Scroll artists right"
                                            on:click={() =>
                                                scrollLibraryRow("artists", 1)}
                                        >
                                            <ChevronRight class="h-4 w-4" />
                                        </button>
                                    </div>
                                </div>
                                <div
                                    bind:this={artistRowElement}
                                    class="flex gap-4 overflow-x-auto pb-2 scrollbar-none -mx-6 px-6"
                                >
                                    {#each artists as artist}
                                        <div
                                            class="w-[180px] shrink-0 text-left"
                                        >
                                            <div
                                                class="mb-2 w-full aspect-square rounded-full bg-hover flex items-center justify-center overflow-hidden"
                                            >
                                                {#if artist.imageUrl}
                                                    <img
                                                        src={artist.imageUrl}
                                                        alt={artist.name}
                                                        class="w-full h-full object-cover"
                                                    />
                                                {:else}
                                                    <UserRound
                                                        class="h-17 w-17 text-tertiary"
                                                    />
                                                {/if}
                                            </div>
                                            <p
                                                class="text-sm font-medium text-white truncate"
                                            >
                                                {artist.name}
                                            </p>
                                            <p
                                                class="text-xs text-secondary truncate"
                                            >
                                                {artist.tracks} tracks
                                            </p>
                                        </div>
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
                                    <h1
                                        class="text-7xl font-bold mt-2 mb-4 truncate whitespace-nowrap max-w-full"
                                    >
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
                                                        playTrack(
                                                            index,
                                                            playbackSourceTracks,
                                                        );
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
                                            class="col-span-2 flex items-center text-secondary min-w-0"
                                        >
                                            <span
                                                class="block w-full truncate whitespace-nowrap"
                                                title={song.album}
                                            >
                                                {formatAlbumName(song.album)}
                                            </span>
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
                {#if isSongsView}
                    <div class="mb-6">
                        <h1 class="text-2xl font-semibold mb-4">Library</h1>
                        <div
                            class="inline-flex items-center gap-1 rounded-full border border-border bg-surface p-1"
                        >
                            <button
                                type="button"
                                class="px-3 py-1.5 rounded-full text-sm [transition:all_0.2s_ease]"
                                class:bg-card={songsLibraryTab === "playlists"}
                                class:text-white={songsLibraryTab ===
                                    "playlists"}
                                class:text-secondary={songsLibraryTab !==
                                    "playlists"}
                                on:click={() => setSongsLibraryTab("playlists")}
                            >
                                Playlists
                            </button>
                            <button
                                type="button"
                                class="px-3 py-1.5 rounded-full text-sm [transition:all_0.2s_ease]"
                                class:bg-card={songsLibraryTab === "albums"}
                                class:text-white={songsLibraryTab === "albums"}
                                class:text-secondary={songsLibraryTab !==
                                    "albums"}
                                on:click={() => setSongsLibraryTab("albums")}
                            >
                                Albums
                            </button>
                            <button
                                type="button"
                                class="px-3 py-1.5 rounded-full text-sm [transition:all_0.2s_ease]"
                                class:bg-card={songsLibraryTab === "tracks"}
                                class:text-white={songsLibraryTab === "tracks"}
                                class:text-secondary={songsLibraryTab !==
                                    "tracks"}
                                on:click={() => setSongsLibraryTab("tracks")}
                            >
                                Tracks
                            </button>
                        </div>
                    </div>
                {:else}
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
                            <h1
                                class="text-7xl font-bold mt-2 mb-4 truncate whitespace-nowrap max-w-full"
                            >
                                {selectedTitle}
                            </h1>
                            <p class="text-secondary">{selectedSubtitle}</p>
                        </div>
                    </div>
                {/if}

                <div style={isSongsView ? songsTabContentStyle : undefined}>
                    {#if !isSongsView || displayedSongsLibraryTab === "tracks"}
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
                                        <div
                                            class="col-span-8 flex items-center"
                                        >
                                            <div
                                                class="w-16 flex items-center justify-center"
                                            >
                                                <div
                                                    class="h-4 bg-hover rounded w-4 animate-pulse"
                                                ></div>
                                            </div>
                                            <div
                                                class="flex items-center flex-1"
                                            >
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
                                        <div
                                            class="col-span-2 flex items-center"
                                        >
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
                                                        playTrack(
                                                            index,
                                                            playbackSourceTracks,
                                                        );
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
                                            class="col-span-2 flex items-center text-secondary min-w-0"
                                        >
                                            <span
                                                class="block w-full truncate whitespace-nowrap"
                                                title={song.album}
                                            >
                                                {formatAlbumName(song.album)}
                                            </span>
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
                    {:else if displayedSongsLibraryTab === "albums"}
                        {#if isLibraryLoading}
                            <div
                                class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4"
                            >
                                {#each Array(12) as _}
                                    <div>
                                        <div
                                            class="w-full aspect-square rounded-xl bg-hover animate-pulse"
                                        ></div>
                                        <div
                                            class="h-4 bg-hover rounded mt-2 w-3/4 animate-pulse"
                                        ></div>
                                        <div
                                            class="h-3 bg-hover rounded mt-2 w-2/3 animate-pulse"
                                        ></div>
                                    </div>
                                {/each}
                            </div>
                        {:else if albums.length === 0}
                            <div class="px-4 py-8 text-secondary text-sm">
                                No albums found.
                            </div>
                        {:else}
                            <div
                                class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4"
                            >
                                {#each albums as album}
                                    <div
                                        class="text-left group"
                                        on:click={() => openAlbum(album)}
                                    >
                                        <div
                                            class="group/cover relative mb-2 w-full aspect-square rounded-xl bg-hover flex items-center justify-center overflow-hidden [transition:background-color_0.2s_ease]"
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
                                            <div
                                                class="pointer-events-none absolute inset-0 bg-black/55 opacity-0 [transition:opacity_0.2s_ease] group-hover/cover:opacity-100"
                                            ></div>
                                            <div
                                                class="pointer-events-none absolute inset-0 opacity-0 [transition:opacity_0.2s_ease] group-hover/cover:opacity-100"
                                            >
                                                <button
                                                    type="button"
                                                    class="pointer-events-auto absolute left-3 bottom-3 h-12 w-12 rounded-full bg-white flex items-center justify-center"
                                                    aria-label={activeAlbumPlaybackKey ===
                                                        album.key &&
                                                    !activeAlbumPlaybackPaused
                                                        ? "Pause album"
                                                        : "Play album"}
                                                    on:click={(event) =>
                                                        toggleAlbumPlayback(
                                                            event,
                                                            album,
                                                        )}
                                                >
                                                    {#if activeAlbumPlaybackKey === album.key && !activeAlbumPlaybackPaused}
                                                        <span
                                                            class="flex items-center gap-[3px]"
                                                        >
                                                            <span
                                                                class="h-[12px] w-[4px] rounded-[1px] bg-black/70"
                                                            ></span>
                                                            <span
                                                                class="h-[12px] w-[4px] rounded-[1px] bg-black/70"
                                                            ></span>
                                                        </span>
                                                    {:else}
                                                        <span
                                                            class="ml-[2px] h-0 w-0 border-y-[7px] border-y-transparent border-l-[12px] border-l-black/70"
                                                        ></span>
                                                    {/if}
                                                </button>
                                            </div>
                                        </div>
                                        <p
                                            class="text-sm font-medium text-white truncate"
                                            title={album.title}
                                        >
                                            {formatAlbumTitle(album.title)}
                                        </p>
                                        <p
                                            class="text-xs text-secondary truncate"
                                        >
                                            {album.artist} • {album.tracks
                                                .length}
                                            tracks
                                        </p>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    {:else}
                        <div
                            class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4"
                        >
                            <button
                                type="button"
                                class="text-left group"
                                on:click={openFavoriteTracks}
                            >
                                <div
                                    class="group/cover relative mb-2 w-full aspect-square rounded-xl bg-hover flex items-center justify-center overflow-hidden [transition:background-color_0.2s_ease]"
                                >
                                    <Heart
                                        fill="currentColor"
                                        class="h-17 w-17 text-tertiary"
                                    />
                                    <div
                                        class="pointer-events-none absolute inset-0 bg-black/55 opacity-0 [transition:opacity_0.2s_ease] group-hover/cover:opacity-100"
                                    ></div>
                                    <div
                                        class="pointer-events-none absolute inset-0 opacity-0 [transition:opacity_0.2s_ease] group-hover/cover:opacity-100"
                                    >
                                        <button
                                            type="button"
                                            class="pointer-events-auto absolute left-3 bottom-3 h-12 w-12 rounded-full bg-white flex items-center justify-center"
                                            aria-label={activeAlbumPlaybackKey ===
                                                FAVORITES_SLUG &&
                                            !activeAlbumPlaybackPaused
                                                ? "Pause playlist"
                                                : "Play playlist"}
                                            on:click={toggleFavoritesPlayback}
                                        >
                                            {#if activeAlbumPlaybackKey === FAVORITES_SLUG && !activeAlbumPlaybackPaused}
                                                <span
                                                    class="flex items-center gap-[3px]"
                                                >
                                                    <span
                                                        class="h-[12px] w-[4px] rounded-[1px] bg-black/70"
                                                    ></span>
                                                    <span
                                                        class="h-[12px] w-[4px] rounded-[1px] bg-black/70"
                                                    ></span>
                                                </span>
                                            {:else}
                                                <span
                                                    class="ml-[2px] h-0 w-0 border-y-[7px] border-y-transparent border-l-[12px] border-l-black/70"
                                                ></span>
                                            {/if}
                                        </button>
                                    </div>
                                </div>
                                <p
                                    class="text-sm font-medium text-white truncate"
                                >
                                    Favorite Tracks
                                </p>
                                <p class="text-xs text-secondary truncate">
                                    Playlist • {favoritesTracks.length} songs
                                </p>
                            </button>
                        </div>
                    {/if}
                </div>
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
