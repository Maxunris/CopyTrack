import { invoke } from "@tauri-apps/api/core";

import { mockBootstrap, mockEntries, mockSettings } from "./mockData";
import type { AppSettings, BootstrapPayload, HistoryItem, HistoryQuery } from "../shared/types/history";

const mockState = {
  entries: [...mockEntries],
  settings: { ...mockSettings },
};

export function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function bootstrapApp() {
  if (!isTauriRuntime()) {
    return Promise.resolve({
      ...mockBootstrap,
      entries: [...mockState.entries],
      settings: { ...mockState.settings },
    });
  }
  return invoke<BootstrapPayload>("bootstrap_app");
}

export function listHistory(query: HistoryQuery) {
  if (!isTauriRuntime()) {
    return Promise.resolve(
      mockState.entries.filter((entry) => {
        if (query.onlyFavorites && !entry.favorite) {
          return false;
        }
        if (query.onlyPinned && !entry.pinned) {
          return false;
        }
        if (query.contentType && query.contentType !== "all" && entry.contentType !== query.contentType) {
          return false;
        }
        if (!query.search) {
          return true;
        }
        const haystack = [entry.previewText, entry.fullText ?? "", entry.tags.join(" "), entry.filePaths.join(" ")]
          .join(" ")
          .toLowerCase();
        return haystack.includes(query.search.toLowerCase());
      }),
    );
  }
  return invoke<HistoryItem[]>("list_history", { query });
}

export function saveSettings(patch: Partial<AppSettings>) {
  if (!isTauriRuntime()) {
    mockState.settings = { ...mockState.settings, ...patch };
    return Promise.resolve({ ...mockState.settings });
  }
  return invoke<AppSettings>("save_settings", { patch });
}

export function togglePin(id: string, pinned: boolean) {
  if (!isTauriRuntime()) {
    mockState.entries = mockState.entries.map((entry) => (entry.id === id ? { ...entry, pinned } : entry));
    return Promise.resolve();
  }
  return invoke<void>("toggle_pin", { id, pinned });
}

export function toggleFavorite(id: string, favorite: boolean) {
  if (!isTauriRuntime()) {
    mockState.entries = mockState.entries.map((entry) => (entry.id === id ? { ...entry, favorite } : entry));
    return Promise.resolve();
  }
  return invoke<void>("toggle_favorite", { id, favorite });
}

export function copyEntry(id: string) {
  if (!isTauriRuntime()) {
    return Promise.resolve();
  }
  return invoke<void>("copy_entry", { id });
}

export function deleteHistoryItems(ids: string[]) {
  if (!isTauriRuntime()) {
    mockState.entries = mockState.entries.filter((entry) => !ids.includes(entry.id));
    return Promise.resolve();
  }
  return invoke<void>("delete_history_items", { ids });
}

export function clearUnpinnedHistory() {
  if (!isTauriRuntime()) {
    mockState.entries = mockState.entries.filter((entry) => entry.pinned);
    return Promise.resolve();
  }
  return invoke<void>("clear_unpinned_history");
}

export function saveTags(id: string, tags: string[]) {
  if (!isTauriRuntime()) {
    mockState.entries = mockState.entries.map((entry) => (entry.id === id ? { ...entry, tags } : entry));
    return Promise.resolve();
  }
  return invoke<void>("save_tags", { patch: { id, tags } });
}

export function openQuickAccess() {
  if (!isTauriRuntime()) {
    if (typeof window !== "undefined") {
      window.location.hash = "quick-access";
    }
    return Promise.resolve();
  }
  return invoke<void>("open_quick_access");
}
