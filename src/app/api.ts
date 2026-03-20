import { invoke } from "@tauri-apps/api/core";

import type { AppSettings, BootstrapPayload, HistoryItem, HistoryQuery } from "../shared/types/history";

export function bootstrapApp() {
  return invoke<BootstrapPayload>("bootstrap_app");
}

export function listHistory(query: HistoryQuery) {
  return invoke<HistoryItem[]>("list_history", { query });
}

export function saveSettings(patch: Partial<AppSettings>) {
  return invoke<AppSettings>("save_settings", { patch });
}

export function togglePin(id: string, pinned: boolean) {
  return invoke<void>("toggle_pin", { id, pinned });
}

export function toggleFavorite(id: string, favorite: boolean) {
  return invoke<void>("toggle_favorite", { id, favorite });
}

export function copyEntry(id: string) {
  return invoke<void>("copy_entry", { id });
}

export function deleteHistoryItems(ids: string[]) {
  return invoke<void>("delete_history_items", { ids });
}

export function clearUnpinnedHistory() {
  return invoke<void>("clear_unpinned_history");
}
