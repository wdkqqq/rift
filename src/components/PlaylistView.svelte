<script lang="ts">
    import { Heart, Music, Play, Plus } from "lucide-svelte";
    import { onDestroy, tick } from "svelte";
    import { activeLibraryView } from "../stores/app";

    export let title = "Favorite songs";
    export let description = "12 minutes • 125 songs";
    export let loading = false;

    const songs = [
        {
            rank: 1,
            title: "Bad for Business",
            artist: "Sabrina Carpenter",
            album: "emails i can't send",
            duration: "3:20",
        },
        {
            rank: 2,
            title: "Tears",
            artist: "Sabrina Carpenter",
            album: "Man's Best Friend",
            duration: "3:35",
        },
    ];

    const albums = [
        {
            title: "Short n' Sweet",
            artist: "Sabrina Carpenter",
        },
        {
            title: "emails i can't send",
            artist: "Sabrina Carpenter",
        },
        {
            title: "BRAT",
            artist: "Charli XCX",
        },
        {
            title: "Future Nostalgia",
            artist: "Dua Lipa",
        },
        {
            title: "HOW I'M FEELING NOW",
            artist: "Charli XCX",
        },
        {
            title: "OIL OF EVERY PEARL'S UN-INSIDES",
            artist: "SOPHIE",
        },
        {
            title: "Britpop",
            artist: "A. G. Cook",
        },
        {
            title: "Happier Than Ever",
            artist: "Billie Eilish",
        },
    ];

    let displayedView: "songs" | "library" = "songs";
    let viewStyle =
        "opacity: 1; transform: translateY(0); transition: all 0.3s ease-in-out;";
    let isAnimating = false;
    let queuedView: "songs" | "library" | null = null;
    let transitionTimer: ReturnType<typeof setTimeout> | null = null;

    function wait(ms: number) {
        return new Promise<void>((resolve) => {
            transitionTimer = setTimeout(resolve, ms);
        });
    }

    async function animateViewSwitch(nextView: "songs" | "library") {
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

    $: if ($activeLibraryView !== displayedView) {
        animateViewSwitch($activeLibraryView);
    }

    onDestroy(() => {
        if (transitionTimer) clearTimeout(transitionTimer);
    });
</script>

<div class="flex-1 overflow-auto m-4 ml-0 p-6">
    <div class="playlist-view-switch" style={viewStyle}>
        {#if displayedView === "library"}
            <div class="max-w-6xl space-y-10">
                <section>
                    <div class="mb-4 flex items-center justify-between">
                        <h2 class="text-xl font-semibold">Playlists</h2>
                        <button
                            type="button"
                            class="h-9 w-9 rounded-lg border border-border bg-surface text-secondary hover:text-white hover:bg-hover flex items-center justify-center [transition:all_0.15s_ease]"
                            aria-label="Add playlist"
                        >
                            <Plus class="h-4 w-4" />
                        </button>
                    </div>

                    <div
                        class="grid grid-cols-[repeat(auto-fill,minmax(220px,220px))] gap-4"
                    >
                        <button
                            type="button"
                            class="w-[220px] text-left rounded-xl border border-divider bg-card p-3 hover:bg-hover [transition:all_0.2s_ease]"
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
                                Favorite tracks
                            </p>
                            <p class="text-xs text-secondary">125 tracks</p>
                        </button>
                    </div>
                </section>

                <section>
                    <h2 class="mb-4 text-xl font-semibold">Albums</h2>
                    <div
                        class="grid grid-cols-[repeat(auto-fill,minmax(220px,220px))] gap-4"
                    >
                        {#each albums as album}
                            <button
                                type="button"
                                class="w-[220px] text-left rounded-xl border border-divider bg-card p-3 hover:bg-hover [transition:all_0.2s_ease]"
                            >
                                <div
                                    class="mb-2 w-full aspect-square rounded-xl bg-hover flex items-center justify-center"
                                >
                                    <Music class="h-17 w-17 text-tertiary" />
                                </div>
                                <p
                                    class="text-sm font-medium text-white truncate"
                                >
                                    {album.title}
                                </p>
                                <p class="text-xs text-secondary truncate">
                                    {album.artist}
                                </p>
                            </button>
                        {/each}
                    </div>
                </section>
            </div>
        {:else}
            <div>
                <div class="flex items-end mb-8">
                    <div
                        class="w-[220px] h-[220px] bg-hover rounded-xl flex items-center justify-center mr-6"
                    >
                        <Heart
                            fill="currentColor"
                            class="h-17 w-17 text-tertiary"
                        />
                    </div>
                    <div>
                        <p class="text-sm font-medium text-secondary">
                            Playlist
                        </p>
                        <h1 class="text-7xl font-bold mt-2 mb-4">
                            {title}
                        </h1>
                        <p class="text-secondary">{description}</p>
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

                {#if loading}
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
                {:else}
                    <div class="space-y-1">
                        {#each songs as song (song.rank)}
                            <div
                                class="grid grid-cols-12 gap-4 px-4 py-3 rounded-lg hover:bg-hover group [transition:all_0.1s_ease]"
                            >
                                <div class="col-span-8 flex items-center">
                                    <div
                                        class="w-16 flex items-center justify-center"
                                    >
                                        <span
                                            class="text-secondary group-hover:hidden"
                                            >{song.rank}</span
                                        >
                                        <Play
                                            class="h-4 w-4 hidden group-hover:block text-white"
                                        />
                                    </div>
                                    <div class="flex items-center flex-1">
                                        <div
                                            class="w-11 h-11 bg-surface rounded mr-3 shrink-0 flex items-center justify-center"
                                        >
                                            <Music
                                                class="h-5 w-5 text-tertiary"
                                            />
                                        </div>
                                        <div>
                                            <div class="text-white">
                                                {song.title}
                                            </div>
                                            <div class="text-sm text-secondary">
                                                {song.artist}
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
