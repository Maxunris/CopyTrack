import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { register, unregister } from "@tauri-apps/plugin-global-shortcut";
import { useDeferredValue, useEffect, useEffectEvent, useRef, useState } from "react";

import {
  bootstrapApp,
  clearUnpinnedHistory,
  copyEntry,
  deleteHistoryItems,
  listHistory,
  saveSettings,
  toggleFavorite,
  togglePin,
} from "./app/api";
import { formatBytes, getHistoryStats, relativeDateLabel, typeLabel } from "./shared/lib/history";
import type { AppSettings, HistoryItem, HistoryQuery } from "./shared/types/history";
import "./App.css";

type HistoryChangedPayload = {
  reason: string;
};

const contentFilters = ["all", "text", "link", "image", "file"];

const emptySettings: AppSettings = {
  captureEnabled: true,
  historyLimit: 100,
  shortcut: "CommandOrControl+Shift+V",
  theme: "system",
  excludedApps: [],
  launchAtLogin: false,
};

export default function App() {
  const [entries, setEntries] = useState<HistoryItem[]>([]);
  const [allEntries, setAllEntries] = useState<HistoryItem[]>([]);
  const [settings, setSettings] = useState<AppSettings>(emptySettings);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [contentType, setContentType] = useState("all");
  const [onlyFavorites, setOnlyFavorites] = useState(false);
  const [onlyPinned, setOnlyPinned] = useState(false);
  const [supportedLimits, setSupportedLimits] = useState<number[]>([50, 100, 500, 1000, 10000]);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [statusMessage, setStatusMessage] = useState("Watching your clipboard locally");
  const deferredSearch = useDeferredValue(search);
  const registeredShortcutRef = useRef<string | null>(null);

  const selectedEntry =
    entries.find((entry) => entry.id === selectedId) ?? entries[0] ?? allEntries.find((entry) => entry.id === selectedId) ?? null;
  const stats = getHistoryStats(allEntries);

  const refreshHistory = useEffectEvent(async (queryOverride?: Partial<HistoryQuery>) => {
    const nextQuery: HistoryQuery = {
      search: deferredSearch,
      contentType,
      onlyFavorites,
      onlyPinned,
      ...queryOverride,
    };

    const nextEntries = await listHistory(nextQuery);
    setEntries(nextEntries);
    setSelectedId((current) => {
      if (current && nextEntries.some((entry) => entry.id === current)) {
        return current;
      }
      return nextEntries[0]?.id ?? null;
    });
  });

  const refreshAllEntries = useEffectEvent(async () => {
    const history = await listHistory({});
    setAllEntries(history);
  });

  const hydrate = useEffectEvent(async () => {
    setIsLoading(true);
    const bootstrap = await bootstrapApp();
    setSettings(bootstrap.settings);
    setEntries(bootstrap.entries);
    setAllEntries(bootstrap.entries);
    setSupportedLimits(bootstrap.supportedHistoryLimits);
    setSelectedId(bootstrap.entries[0]?.id ?? null);
    setAutostartEnabled(await isEnabled().catch(() => false));
    setIsLoading(false);
  });

  useEffect(() => {
    void hydrate();
  }, [hydrate]);

  useEffect(() => {
    if (isLoading) {
      return;
    }
    void refreshHistory();
  }, [contentType, deferredSearch, onlyFavorites, onlyPinned, isLoading, refreshHistory]);

  useEffect(() => {
    if (isLoading) {
      return;
    }

    let unlisten: (() => void) | null = null;
    void (async () => {
      unlisten = await listen<HistoryChangedPayload>("history-changed", async () => {
        await refreshHistory();
        await refreshAllEntries();
      });
    })();

    return () => {
      unlisten?.();
    };
  }, [isLoading, refreshAllEntries, refreshHistory]);

  useEffect(() => {
    if (isLoading) {
      return;
    }

    const shortcut = settings.shortcut || emptySettings.shortcut;
    void (async () => {
      const previousShortcut = registeredShortcutRef.current;
      if (previousShortcut && previousShortcut !== shortcut) {
        await unregister(previousShortcut).catch(() => undefined);
      }

      await register(shortcut, async () => {
        const window = getCurrentWindow();
        await window.show();
        await window.unminimize();
        await window.setFocus();
        setStatusMessage(`Quick access opened with ${shortcut}`);
        window.setAlwaysOnTop(true).catch(() => undefined);
        window.setAlwaysOnTop(false).catch(() => undefined);
      }).catch(() => undefined);
      registeredShortcutRef.current = shortcut;
    })();

    return () => {
      if (registeredShortcutRef.current === shortcut) {
        void unregister(shortcut).catch(() => undefined);
      }
    };
  }, [isLoading, settings.shortcut]);

  async function handleSettingsSave(patch: Partial<AppSettings>) {
    setIsSaving(true);
    try {
      if (patch.launchAtLogin !== undefined && patch.launchAtLogin !== autostartEnabled) {
        if (patch.launchAtLogin) {
          await enable();
        } else {
          await disable();
        }
        setAutostartEnabled(patch.launchAtLogin);
      }

      const nextSettings = await saveSettings(patch);
      setSettings(nextSettings);
      setStatusMessage("Settings saved");
      await refreshAllEntries();
      await refreshHistory();
    } finally {
      setIsSaving(false);
    }
  }

  async function handleCopy(entry: HistoryItem) {
    await copyEntry(entry.id);
    setSelectedId(entry.id);
    setStatusMessage(`Copied ${typeLabel(entry.contentType).toLowerCase()} item back to clipboard`);
  }

  async function handleFavorite(entry: HistoryItem) {
    await toggleFavorite(entry.id, !entry.favorite);
    setStatusMessage(entry.favorite ? "Removed from favorites" : "Added to favorites");
    await refreshAllEntries();
    await refreshHistory();
  }

  async function handlePin(entry: HistoryItem) {
    await togglePin(entry.id, !entry.pinned);
    setStatusMessage(entry.pinned ? "Removed pin" : "Pinned for quick reuse");
    await refreshAllEntries();
    await refreshHistory();
  }

  async function handleDelete(entry: HistoryItem) {
    await deleteHistoryItems([entry.id]);
    setStatusMessage("Entry deleted");
    await refreshAllEntries();
    await refreshHistory();
  }

  async function handleClear() {
    await clearUnpinnedHistory();
    setStatusMessage("Cleared unpinned history");
    await refreshAllEntries();
    await refreshHistory();
  }

  return (
    <div className="app-shell">
      <div className="app-background" />
      <aside className="hero-column glass-panel">
        <div className="brand-lockup">
          <div className="brand-icon">
            <img alt="CopyTrack icon" src="/app-icon.svg" />
          </div>
          <div>
            <p className="eyebrow">Clipboard history for macOS</p>
            <h1>CopyTrack</h1>
            <p className="lede">
              A local-first clipboard utility with quick recall, one-click re-copy, menu bar access, and a polished
              glass-inspired interface.
            </p>
          </div>
        </div>

        <div className="status-strip">
          <span className={`status-dot ${settings.captureEnabled ? "live" : "paused"}`} />
          <span>{statusMessage}</span>
        </div>

        <div className="stats-grid">
          <StatCard label="Saved" value={stats.total} />
          <StatCard label="Pinned" value={stats.pinned} />
          <StatCard label="Favorites" value={stats.favorites} />
          <StatCard label="Limit" value={settings.historyLimit} />
        </div>

        <div className="hero-actions">
          <button className="primary-button" onClick={() => void setSettingsOpen(true)} type="button">
            Open Settings
          </button>
          <button className="secondary-button" onClick={() => void handleClear()} type="button">
            Clear Unpinned
          </button>
        </div>

        <div className="feature-list">
          <FeatureLine title="Quick access shortcut" value={settings.shortcut} />
          <FeatureLine title="Launch at login" value={autostartEnabled ? "Enabled" : "Disabled"} />
          <FeatureLine title="Capture mode" value={settings.captureEnabled ? "Recording" : "Paused"} />
          <FeatureLine title="Theme" value={settings.theme} />
        </div>
      </aside>

      <main className="content-column glass-panel">
        <header className="toolbar">
          <div className="toolbar-copy">
            <p className="eyebrow">History</p>
            <h2>Find anything you copied</h2>
          </div>

          <div className="toolbar-actions">
            <label className="search-field" htmlFor="history-search">
              <span>Search</span>
              <input
                id="history-search"
                onChange={(event) => setSearch(event.target.value)}
                placeholder="Search text, links, file paths, tags"
                value={search}
              />
            </label>
            <button className="ghost-button" onClick={() => void setSettingsOpen(true)} type="button">
              Settings
            </button>
          </div>
        </header>

        <div className="filter-row">
          {contentFilters.map((filter) => (
            <button
              className={`filter-chip ${contentType === filter ? "active" : ""}`}
              key={filter}
              onClick={() => setContentType(filter)}
              type="button"
            >
              {filter === "all" ? "All" : typeLabel(filter)}
            </button>
          ))}
          <button
            className={`toggle-chip ${onlyFavorites ? "active" : ""}`}
            onClick={() => setOnlyFavorites((current) => !current)}
            type="button"
          >
            Favorites
          </button>
          <button
            className={`toggle-chip ${onlyPinned ? "active" : ""}`}
            onClick={() => setOnlyPinned((current) => !current)}
            type="button"
          >
            Pinned
          </button>
        </div>

        <div className="workspace-grid">
          <section className="history-list">
            {isLoading ? (
              <div className="empty-state">Preparing your clipboard workspace…</div>
            ) : entries.length === 0 ? (
              <div className="empty-state">
                <p>No entries match the current view.</p>
                <span>Copy some content or change the filters to see more history.</span>
              </div>
            ) : (
              entries.map((entry) => (
                <button
                  className={`history-row ${selectedEntry?.id === entry.id ? "selected" : ""}`}
                  key={entry.id}
                  onClick={() => void handleCopy(entry)}
                  type="button"
                >
                  <div className="history-row-top">
                    <span className={`type-pill type-${entry.contentType}`}>{typeLabel(entry.contentType)}</span>
                    <span className="meta-text">{relativeDateLabel(entry.createdAt)}</span>
                  </div>
                  <div className="history-row-preview">{entry.previewText}</div>
                  <div className="history-row-footer">
                    <span>{formatBytes(entry.sizeBytes)}</span>
                    <div className="history-row-actions">
                      <ActionBadge active={entry.favorite} label="Favorite" onClick={() => void handleFavorite(entry)} />
                      <ActionBadge active={entry.pinned} label="Pin" onClick={() => void handlePin(entry)} />
                      <ActionBadge active={false} label="Delete" onClick={() => void handleDelete(entry)} />
                    </div>
                  </div>
                </button>
              ))
            )}
          </section>

          <section className="preview-panel">
            {selectedEntry ? (
              <>
                <div className="preview-header">
                  <div>
                    <p className="eyebrow">Preview</p>
                    <h3>{selectedEntry.previewText}</h3>
                  </div>
                  <span className={`type-pill type-${selectedEntry.contentType}`}>{typeLabel(selectedEntry.contentType)}</span>
                </div>

                {selectedEntry.imagePath ? (
                  <div className="image-preview">
                    <img alt={selectedEntry.previewText} src={convertFileSrc(selectedEntry.imagePath)} />
                  </div>
                ) : (
                  <pre className="preview-code">{selectedEntry.fullText ?? selectedEntry.filePaths.join("\n")}</pre>
                )}

                <div className="preview-metadata">
                  <MetaLine label="Copied" value={relativeDateLabel(selectedEntry.createdAt)} />
                  <MetaLine label="Type" value={typeLabel(selectedEntry.contentType)} />
                  <MetaLine label="Source App" value={selectedEntry.sourceApp ?? "Unavailable"} />
                  <MetaLine label="Size" value={formatBytes(selectedEntry.sizeBytes)} />
                </div>

                <div className="preview-actions">
                  <button className="primary-button" onClick={() => void handleCopy(selectedEntry)} type="button">
                    Copy Again
                  </button>
                  <button className="secondary-button" onClick={() => void handleFavorite(selectedEntry)} type="button">
                    {selectedEntry.favorite ? "Remove Favorite" : "Add Favorite"}
                  </button>
                </div>
              </>
            ) : (
              <div className="empty-state">
                <p>Nothing selected yet.</p>
                <span>Choose an entry from your history to inspect it and copy it again.</span>
              </div>
            )}
          </section>
        </div>
      </main>

      {settingsOpen ? (
        <div className="settings-sheet">
          <div className="settings-card glass-panel">
            <div className="settings-header">
              <div>
                <p className="eyebrow">Preferences</p>
                <h3>Shape CopyTrack to your workflow</h3>
              </div>
              <button className="ghost-button" onClick={() => setSettingsOpen(false)} type="button">
                Close
              </button>
            </div>

            <div className="settings-grid">
              <label>
                <span>Capture status</span>
                <select
                  onChange={(event) =>
                    void handleSettingsSave({ captureEnabled: event.target.value === "enabled" })
                  }
                  value={settings.captureEnabled ? "enabled" : "paused"}
                >
                  <option value="enabled">Enabled</option>
                  <option value="paused">Paused</option>
                </select>
              </label>

              <label>
                <span>History limit</span>
                <select
                  onChange={(event) => void handleSettingsSave({ historyLimit: Number(event.target.value) })}
                  value={settings.historyLimit}
                >
                  {supportedLimits.map((value) => (
                    <option key={value} value={value}>
                      {value} items
                    </option>
                  ))}
                </select>
              </label>

              <label>
                <span>Quick access shortcut</span>
                <input
                  onBlur={(event) =>
                    void handleSettingsSave({ shortcut: event.target.value || emptySettings.shortcut })
                  }
                  placeholder={emptySettings.shortcut}
                  type="text"
                  value={settings.shortcut}
                  onChange={(event) => setSettings((current) => ({ ...current, shortcut: event.target.value }))}
                />
              </label>

              <label>
                <span>Launch at login</span>
                <select
                  onChange={(event) =>
                    void handleSettingsSave({ launchAtLogin: event.target.value === "enabled" })
                  }
                  value={autostartEnabled ? "enabled" : "disabled"}
                >
                  <option value="enabled">Enabled</option>
                  <option value="disabled">Disabled</option>
                </select>
              </label>

              <label>
                <span>Theme</span>
                <select onChange={(event) => void handleSettingsSave({ theme: event.target.value })} value={settings.theme}>
                  <option value="system">System</option>
                  <option value="light">Light</option>
                  <option value="dark">Dark</option>
                </select>
              </label>

              <label className="full-width">
                <span>Excluded apps</span>
                <textarea
                  onBlur={(event) =>
                    void handleSettingsSave({
                      excludedApps: event.target.value
                        .split("\n")
                        .map((value) => value.trim())
                        .filter(Boolean),
                    })
                  }
                  onChange={(event) =>
                    setSettings((current) => ({
                      ...current,
                      excludedApps: event.target.value
                        .split("\n")
                        .map((value) => value.trim())
                        .filter(Boolean),
                    }))
                  }
                  placeholder={"com.1password.1password\ncom.apple.keychainaccess"}
                  value={settings.excludedApps.join("\n")}
                />
              </label>
            </div>

            <p className="settings-note">
              Settings save on change. Shortcut updates after the field loses focus. {isSaving ? "Saving…" : "All changes are local only."}
            </p>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <div className="stat-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function FeatureLine({ title, value }: { title: string; value: string }) {
  return (
    <div className="feature-line">
      <span>{title}</span>
      <strong>{value}</strong>
    </div>
  );
}

function MetaLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="meta-line">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function ActionBadge({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <span
      className={`action-badge ${active ? "active" : ""}`}
      onClick={(event) => {
        event.stopPropagation();
        onClick();
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
      role="button"
      tabIndex={0}
    >
      {label}
    </span>
  );
}
