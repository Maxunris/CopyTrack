import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

import { mockBootstrap, mockEntries, mockSettings } from "./mockData";
import type {
  AppSettings,
  BootstrapPayload,
  ExportSummary,
  HistoryItem,
  HistoryQuery,
  ImportMode,
  ImportSummary,
} from "../shared/types/history";

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

export async function exportHistory() {
  if (!isTauriRuntime()) {
    const archive = JSON.stringify(
      {
        version: 1,
        exportedAt: new Date().toISOString(),
        settings: mockState.settings,
        entries: mockState.entries.map((entry) => ({
          ...entry,
          imageDataBase64: null,
        })),
      },
      null,
      2,
    );

    const blob = new Blob([archive], { type: "application/json" });
    const href = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = href;
    anchor.download = "copytrack-history-export.json";
    anchor.click();
    URL.revokeObjectURL(href);

    return Promise.resolve<ExportSummary>({
      path: "copytrack-history-export.json",
      entryCount: mockState.entries.length,
    });
  }

  const path = await save({
    defaultPath: "copytrack-history-export.json",
    filters: [{ name: "CopyTrack Export", extensions: ["json"] }],
  });

  if (!path) {
    return null;
  }

  return invoke<ExportSummary>("export_history", { path });
}

export async function importHistory(mode: ImportMode) {
  if (!isTauriRuntime()) {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json,application/json";

    const file = await new Promise<File | null>((resolve) => {
      input.onchange = () => resolve(input.files?.[0] ?? null);
      input.click();
    });

    if (!file) {
      return null;
    }

    const raw = await file.text();
    const archive = JSON.parse(raw) as {
      settings?: AppSettings;
      entries?: HistoryItem[];
    };

    const importedEntries = archive.entries ?? [];
    const existingIds = new Set(mockState.entries.map((entry) => entry.id));
    let importedCount = 0;
    let skippedCount = 0;

    if (mode === "replace") {
      mockState.entries = importedEntries.map((entry, index) => ({
        ...entry,
        id: `${entry.id || "imported"}-${index}`,
      }));
      if (archive.settings) {
        mockState.settings = { ...archive.settings };
      }
      importedCount = mockState.entries.length;
    } else {
      const merged = [...mockState.entries];
      for (const entry of importedEntries) {
        if (existingIds.has(entry.id)) {
          skippedCount += 1;
          continue;
        }
        merged.push(entry);
        existingIds.add(entry.id);
        importedCount += 1;
      }
      mockState.entries = merged;
    }

    return Promise.resolve<ImportSummary>({
      path: file.name,
      importedCount,
      skippedCount,
      mode,
    });
  }

  const path = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "CopyTrack Export", extensions: ["json"] }],
  });

  if (!path || Array.isArray(path)) {
    return null;
  }

  return invoke<ImportSummary>("import_history", { path, mode });
}
