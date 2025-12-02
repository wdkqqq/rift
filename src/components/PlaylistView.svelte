<script lang="ts">
  import { Music, Play, Heart } from "lucide-svelte";

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
      title: "Save Your Tears",
      artist: "The Weeknd",
      album: "After Hours",
      duration: "3:35",
    },
  ];
</script>

<div class="flex-1 p-6 overflow-auto m-4 ml-0">
  <!-- Header-->
  <div class="flex items-end mb-8">
    <div
      class="w-[220px] h-[220px] bg-hover shadow-lg rounded-xl flex items-center justify-center mr-6"
    >
      <Heart fill="currentColor" class="h-17 w-17 text-tertiary" />
    </div>
    <div>
      <p class="text-sm font-medium text-secondary">Playlist</p>
      <h1 class="text-7xl font-bold mt-2 mb-4">{title}</h1>
      <p class="text-secondary">{description}</p>
    </div>
  </div>

  <!-- Table Header -->
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
    <!-- Skeleton Songs List -->
    <div class="space-y-1">
      {#each Array(8) as _, i}
        <div class="grid grid-cols-12 gap-4 px-4 py-3 rounded-lg">
          <div class="col-span-8 flex items-center">
            <div class="w-16 flex items-center justify-center">
              <div class="h-4 bg-hover rounded w-4 animate-pulse"></div>
            </div>
            <div class="flex items-center flex-1">
              <div
                class="w-11 h-11 bg-hover rounded mr-3 shrink-0 animate-pulse"
              ></div>
              <div class="flex-1">
                <div
                  class="h-4 bg-hover rounded w-32 mb-2 animate-pulse"
                ></div>
                <div class="h-3 bg-hover rounded w-24 animate-pulse"></div>
              </div>
            </div>
          </div>
          <div class="col-span-2 flex items-center">
            <div class="h-3 bg-hover rounded w-20 animate-pulse"></div>
          </div>
          <div class="col-span-2 flex items-center justify-end">
            <div class="h-3 bg-hover rounded w-12 animate-pulse"></div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <!-- Original Songs List -->
    <div class="space-y-1">
      {#each songs as song (song.rank)}
        <div
          class="grid grid-cols-12 gap-4 px-4 py-3 rounded-lg transition hover:bg-hover group transition"
        >
          <div class="col-span-8 flex items-center">
            <div class="w-16 flex items-center justify-center">
              <span class="text-secondary group-hover:hidden">{song.rank}</span>
              <Play class="h-4 w-4 hidden group-hover:block text-white" />
            </div>
            <div class="flex items-center flex-1">
              <div
                class="w-11 h-11 bg-[#171717] rounded mr-3 shrink-0 flex items-center justify-center"
              >
                <Music class="h-5 w-5 text-tertiary" />
              </div>
              <div>
                <div class="text-white">{song.title}</div>
                <div class="text-sm text-secondary">
                  {song.artist}
                </div>
              </div>
            </div>
          </div>
          <div class="col-span-2 flex items-center text-secondary">
            {song.album}
          </div>
          <div class="col-span-2 flex items-center justify-end text-secondary">
            {song.duration}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
