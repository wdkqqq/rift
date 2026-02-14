import { writable } from "svelte/store";

export const commandPaletteOpen = writable(false);
export const settingsPanelOpen = writable(false);
export const activeSettingsTab = writable("audio");
export const activeLibraryView = writable<"songs" | "library" | "detail">(
  "songs",
);
export const favoritesOpenRequest = writable(0);
export const playlistsRefreshToken = writable(0);

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

export type NotificationType = "info" | "success" | "error";

export type AppNotification = {
  id: number;
  type: NotificationType;
  message: string;
};

export const activeNotification = writable<AppNotification | null>(null);

let notificationTimeout: ReturnType<typeof setTimeout> | null = null;

export function hideNotification() {
  activeNotification.set(null);
}

export function showNotification(
  message: string,
  type: NotificationType = "info",
  durationMs = 3500,
) {
  if (notificationTimeout) {
    clearTimeout(notificationTimeout);
  }

  activeNotification.set({
    id: Date.now(),
    type,
    message,
  });

  notificationTimeout = setTimeout(() => {
    hideNotification();
    notificationTimeout = null;
  }, durationMs);
}

export function notifyInfo(message: string, durationMs?: number) {
  showNotification(message, "info", durationMs);
}

export function notifySuccess(message: string, durationMs?: number) {
  showNotification(message, "success", durationMs);
}

export function notifyError(message: string, durationMs?: number) {
  showNotification(message, "error", durationMs);
}

export function refreshPlaylists() {
  playlistsRefreshToken.update((value) => value + 1);
}
