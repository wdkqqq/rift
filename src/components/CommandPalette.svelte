<script>
  import { onMount } from "svelte";
  import { commandPaletteOpen } from "../stores/app.ts";
  import { Search, Music, Loader } from "lucide-svelte";
  import { writable } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import { readFile, BaseDirectory } from "@tauri-apps/plugin-fs";
  import { appCacheDir } from "@tauri-apps/api/path";

  let search = "";
  let items = writable([]);
  let isLoading = writable(false);
  let activeIndex = writable(-1);

  $: hasSearchText = search.length > 0;

  $: if (search.length > 0) {
    performSearch(search);
  }

  async function getCoverUrl(coverFilename) {
    if (!coverFilename) return null;

    try {
      const cacheDir = await appCacheDir();

      const path = `${cacheDir}/covers/${coverFilename}`;

      const data = await readFile(path, {
        dir: BaseDirectory.Cache,
        encoding: null,
      });

      const blob = new Blob([data]);
      return URL.createObjectURL(blob);
    } catch (err) {
      console.error("Cannot load cover:", err);
      return null;
    }
  }

  async function performSearch(query) {
    if (query.length < 1) return;

    isLoading.set(true);
    try {
      console.log("Searching for:", query);
      const results = await invoke("search_music", { query });

      const processedResults = await Promise.all(
        results.map(async (song) => {
          let coverUrl = null;
          if (song.cover) {
            coverUrl = await getCoverUrl(song.cover);
          }
          return {
            ...song,
            coverUrl,
          };
        }),
      );

      console.log("Processed search results:", processedResults);
      items.set(processedResults);
    } catch (error) {
      console.error("Search error:", error);
      items.set([]);
    } finally {
      isLoading.set(false);
    }
  }

  function handleKeydown(event) {
    if (!$commandPaletteOpen) return;

    let currentItems;
    const unsubscribe = items.subscribe((value) => {
      currentItems = value;
    });
    unsubscribe();

    activeIndex.update((i) => {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        return i < currentItems.length - 1 ? i + 1 : 0;
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        return i > 0 ? i - 1 : currentItems.length - 1;
      } else if (event.key === "Enter") {
        event.preventDefault();
        if (i >= 0 && currentItems[i]) selectItem(i);
      } else if (event.key === "Escape") {
        commandPaletteOpen.set(false);
      }
      return i;
    });
  }

  function selectItem(index) {
    let currentItem;
    const unsubscribe = items.subscribe((value) => {
      if (value[index]) {
        currentItem = value[index];
      }
    });
    unsubscribe();

    if (currentItem) {
      console.log("Selected song:", currentItem);
      alert(`Selected: ${currentItem.title} by ${currentItem.subtitle}`);
      commandPaletteOpen.set(false);
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div
  class="command-palette fixed top-0 left-0 right-0 bottom-0 m-auto w-1/3 min-w-150 h-fit bg-background border border-border rounded-2xl box-shadow z-50 transition-all duration-300 {$commandPaletteOpen
    ? 'active'
    : ''}"
>
  <div class="p-2">
    <div class="relative" class:mb-2={hasSearchText}>
      <input
        type="text"
        bind:value={search}
        id="command-search"
        placeholder="Search songs, artists, albums..."
        class="w-full bg-[bg-background] py-2.5 pl-10 text-white placeholder-secondary focus:outline-none"
      />
      <Search
        class="w-5 h-5 text-secondary absolute left-3 top-1/2 transform -translate-y-1/2"
      />
    </div>

    <div
      class="border-b border-border transition-all duration-300 overflow-hidden"
      class:mb-3={hasSearchText}
      class:opacity-0={!hasSearchText}
      class:opacity-100={hasSearchText}
      class:h-0={!hasSearchText}
      class:h-auto={hasSearchText}
    ></div>

    <div
      class="max-h-80 overflow-y-auto transition-all duration-300"
      class:opacity-0={!hasSearchText}
      class:opacity-100={hasSearchText}
      class:max-h-0={!hasSearchText}
      class:max-h-80={hasSearchText}
    >
      {#if $isLoading}
        <div class="flex justify-center items-center py-8">
          <Loader class="w-6 h-6 text-secondary animate-spin" />
        </div>
      {:else if $items.length === 0 && hasSearchText}
        <div class="text-center py-8 text-secondary">
          No results found for "{search}"
        </div>
      {:else}
        {#each $items as item, index}
          <div
            class="p-3 rounded-lg [transition:all_0.1s_ease] hover:bg-hover flex items-center {index ===
            $activeIndex
              ? 'bg-hover'
              : ''}"
            on:click={() => selectItem(index)}
          >
            <div
              class="w-10 h-10 bg-surface rounded mr-3 shrink-0 flex items-center justify-center overflow-hidden"
            >
              {#if item.coverUrl}
                <img
                  src={item.coverUrl}
                  alt={item.title}
                  class="w-full h-full object-cover"
                />
              {:else}
                <Music class="h-5 w-5 text-tertiary" />
              {/if}
            </div>
            <div class="flex-1 min-w-0">
              <div class="text-white truncate">{item.title}</div>
              <div class="text-sm text-secondary truncate">
                {item.subtitle}
              </div>
            </div>
            <div class="text-secondary text-sm ml-2 shrink-0">
              {item.duration}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
