import { writable } from "svelte/store";

export const commandPaletteOpen = writable(false);
export const settingsPanelOpen = writable(false);
export const onboardingOpen = writable(false);
export const activeSettingsTab = writable("general");
export const activeLibraryView = writable<"songs" | "library" | "detail">(
  "library",
);

export type RiftHistoryState =
  | { riftView: "library-home" }
  | { riftView: "songs" }
  | { riftView: "detail-home" }
  | { riftView: "detail-album"; albumId: string };

export function isRiftHistoryState(value: unknown): value is RiftHistoryState {
  if (!value || typeof value !== "object") return false;
  const state = value as { riftView?: unknown; albumId?: unknown };

  if (state.riftView === "library-home") return true;
  if (state.riftView === "songs") return true;
  if (state.riftView === "detail-home") return true;

  return state.riftView === "detail-album" && typeof state.albumId === "string";
}

function sameRiftHistoryState(a: RiftHistoryState, b: RiftHistoryState) {
  return a.riftView === b.riftView && a.albumId === b.albumId;
}

export function pushRiftHistoryState(state: RiftHistoryState) {
  if (typeof window === "undefined") return;
  const currentState = window.history.state;
  if (
    isRiftHistoryState(currentState) &&
    sameRiftHistoryState(currentState, state)
  ) {
    return;
  }

  window.history.pushState(state, "");
}

export function ensureRiftHistoryState(state: RiftHistoryState) {
  if (typeof window === "undefined") return;
  const currentState = window.history.state;
  if (
    isRiftHistoryState(currentState) &&
    sameRiftHistoryState(currentState, state)
  ) {
    return;
  }

  window.history.replaceState(state, "");
}
export const favoritesOpenRequest = writable(0);
export const playlistsRefreshToken = writable(0);
export const listeningInsightsRefreshToken = writable(0);
export const albumOpenRequest = writable<{
  id: number;
  title: string;
  artist: string;
} | null>(null);

export type PlaybackSourceKind = "album" | "playlist" | "other";

export type PlaybackSource = {
  kind: PlaybackSourceKind;
  id?: string | null;
  name?: string | null;
};

export type PlayerTrack = {
  title: string;
  subtitle: string;
  album: string;
  duration: string;
  coverUrl: string | null;
  path: string;
  source?: PlaybackSource;
};

export const playbackQueue = writable<PlayerTrack[]>([]);
export const playbackIndex = writable(0);
export const playbackIsPlaying = writable(false);

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

export function refreshListeningInsights() {
  listeningInsightsRefreshToken.update((value) => value + 1);
}

export function requestOpenAlbum(title: string, artist: string) {
  albumOpenRequest.set({
    id: Date.now(),
    title,
    artist,
  });
}
