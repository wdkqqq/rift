import { writable } from "svelte/store";

export const commandPaletteOpen = writable(false);
export const settingsPanelOpen = writable(false);
export const activeSettingsTab = writable("audio");
export const activeLibraryView = writable<"songs" | "library">("songs");
export const libraryHomeRequest = writable(0);
export const favoritesOpenRequest = writable(0);

export type PlayerTrack = {
  title: string;
  subtitle: string;
  album: string;
  duration: string;
  coverUrl: string | null;
  path: string;
};

export const playbackQueue = writable<PlayerTrack[]>([]);
export const playbackIndex = writable(0);
