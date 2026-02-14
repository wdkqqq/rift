import { writable } from "svelte/store";

export const commandPaletteOpen = writable(false);
export const settingsPanelOpen = writable(false);
export const activeSettingsTab = writable("audio");
export const activeLibraryView = writable<"songs" | "library">("songs");
