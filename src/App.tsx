import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { useCallback, useDeferredValue, useEffect, useState } from "react";

import {
  bootstrapApp,
  clearUnpinnedHistory,
  copyEntry,
  isTauriRuntime,
  deleteHistoryItems,
  exportHistory,
  hideQuickAccess,
  importHistory,
  listHistory,
  openQuickAccess,
  saveSettings,
  saveTags,
  toggleFavorite,
  togglePin,
} from "./app/api";
import {
  collectTags,
  filterEntries,
  formatBytes,
  getHistoryStats,
  relativeDateLabel,
  sortEntries,
  typeLabel,
  type SortMode,
} from "./shared/lib/history";
import { messages, resolveUiLanguage } from "./shared/i18n";
import type { AppSettings, HistoryItem, HistoryQuery, ImportMode } from "./shared/types/history";
import "./App.css";

type HistoryChangedPayload = {
  reason: string;
};

type AppNavigationPayload = {
  destination: string;
};

const contentFilters = ["all", "text", "link", "image", "file"];
const isTauri = isTauriRuntime();

function getRuntimeWindow() {
  return isTauriRuntime() ? getCurrentWindow() : null;
}

function readPreviewOptions() {
  if (typeof window === "undefined") {
    return {
      panel: "",
      language: "",
    };
  }

  const params = new URLSearchParams(window.location.search);
  return {
    panel: params.get("panel") ?? "",
    language: params.get("lang") ?? "",
  };
}

const emptySettings: AppSettings = {
  captureEnabled: true,
  historyLimit: 100,
  shortcut: "CommandOrControl+Shift+V",
  theme: "system",
  language: "system",
  onboardingCompleted: false,
  excludedApps: [],
  launchAtLogin: false,
};

export default function App() {
  const [routeHash, setRouteHash] = useState(() => (typeof window !== "undefined" ? window.location.hash : ""));
  const [previewPanel, setPreviewPanel] = useState(() => readPreviewOptions().panel);
  const [previewLanguage, setPreviewLanguage] = useState(() => readPreviewOptions().language);
  const [entries, setEntries] = useState<HistoryItem[]>([]);
  const [allEntries, setAllEntries] = useState<HistoryItem[]>([]);
  const [settings, setSettings] = useState<AppSettings>(emptySettings);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [contentType, setContentType] = useState("all");
  const [onlyFavorites, setOnlyFavorites] = useState(false);
  const [onlyPinned, setOnlyPinned] = useState(false);
  const [selectedTag, setSelectedTag] = useState("");
  const [sortMode, setSortMode] = useState<SortMode>("recent");
  const [tagDraft, setTagDraft] = useState("");
  const [supportedLimits, setSupportedLimits] = useState<number[]>([50, 100, 500, 1000, 10000]);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [onboardingOpen, setOnboardingOpen] = useState(false);
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [importMode, setImportMode] = useState<ImportMode>("merge");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isPortingData, setIsPortingData] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string>(messages.en.watching);
  const deferredSearch = useDeferredValue(search);
  const currentWindow = getRuntimeWindow();
  const isQuickAccess = currentWindow?.label === "quick-access" || routeHash === "#quick-access";
  const settingsForced = previewPanel === "settings";
  const onboardingForced = previewPanel === "onboarding";
  const uiLanguage = resolveUiLanguage(settings.language);
  const copy = messages[uiLanguage];
  const themeLabel = settings.theme === "light" ? copy.light : settings.theme === "dark" ? copy.dark : copy.system;

  const scopedEntries = filterEntries(entries, "", false, false, "all", selectedTag);
  const visibleEntries = sortEntries(scopedEntries, sortMode);
  const selectedEntry =
    visibleEntries.find((entry) => entry.id === selectedId) ??
    visibleEntries[0] ??
    allEntries.find((entry) => entry.id === selectedId) ??
    null;
  const stats = getHistoryStats(allEntries);
  const availableTags = collectTags(allEntries);

  const refreshHistory = useCallback(async () => {
    const nextQuery: HistoryQuery = {
      search: deferredSearch,
      contentType,
      onlyFavorites,
      onlyPinned,
    };

    const nextEntries = await listHistory(nextQuery);
    setEntries(nextEntries);
    setSelectedId((current) => {
      if (current && nextEntries.some((entry) => entry.id === current)) {
        return current;
      }
      return nextEntries[0]?.id ?? null;
    });
  }, [contentType, deferredSearch, onlyFavorites, onlyPinned]);

  const refreshAllEntries = useCallback(async () => {
    const history = await listHistory({});
    setAllEntries(history);
  }, []);

  const hydrate = useCallback(async () => {
    setIsLoading(true);
    const bootstrap = await bootstrapApp();
    const nextSettings =
      !isTauri && (previewLanguage === "en" || previewLanguage === "ru")
        ? { ...bootstrap.settings, language: previewLanguage }
        : bootstrap.settings;
    setSettings(nextSettings);
    setEntries(bootstrap.entries);
    setAllEntries(bootstrap.entries);
    setSupportedLimits(bootstrap.supportedHistoryLimits);
    setSelectedId(bootstrap.entries[0]?.id ?? null);
    setAutostartEnabled(isTauri ? await isEnabled().catch(() => false) : nextSettings.launchAtLogin);
    setSettingsOpen(settingsForced);
    setOnboardingOpen(onboardingForced || !nextSettings.onboardingCompleted);
    setStatusMessage(messages[resolveUiLanguage(nextSettings.language)].watching);
    setIsLoading(false);
  }, [onboardingForced, previewLanguage, settingsForced]);

  useEffect(() => {
    void hydrate();
  }, [hydrate]);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    const syncLocation = () => {
      setRouteHash(window.location.hash);
      const nextPreview = readPreviewOptions();
      setPreviewPanel(nextPreview.panel);
      setPreviewLanguage(nextPreview.language);
    };

    window.addEventListener("hashchange", syncLocation);
    window.addEventListener("popstate", syncLocation);
    return () => {
      window.removeEventListener("hashchange", syncLocation);
      window.removeEventListener("popstate", syncLocation);
    };
  }, []);

  useEffect(() => {
    if (isLoading) {
      return;
    }
    void refreshHistory();
  }, [isLoading, refreshHistory]);

  useEffect(() => {
    if (selectedEntry) {
      setTagDraft(selectedEntry.tags.join(", "));
    } else {
      setTagDraft("");
    }
  }, [selectedEntry?.id]);

  useEffect(() => {
    if (isLoading || !isTauri) {
      return;
    }

    let unlistenHistory: (() => void) | null = null;
    let unlistenNavigation: (() => void) | null = null;
    void (async () => {
      unlistenHistory = await listen<HistoryChangedPayload>("history-changed", async () => {
        await refreshHistory();
        await refreshAllEntries();
      });
      unlistenNavigation = await listen<AppNavigationPayload>("app-navigation", async (event) => {
        if (event.payload.destination === "settings") {
          setSettingsOpen(true);
        }
      });
    })();

    return () => {
      unlistenHistory?.();
      unlistenNavigation?.();
    };
  }, [isLoading, refreshAllEntries, refreshHistory]);

  useEffect(() => {
    if (isLoading) {
      return;
    }

    if (settingsForced) {
      setSettingsOpen(true);
    }
    if (onboardingForced) {
      setOnboardingOpen(true);
    }
  }, [isLoading, onboardingForced, settingsForced]);

  useEffect(() => {
    if (!isQuickAccess || isLoading) {
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const isTypingTarget =
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target instanceof HTMLSelectElement;

      if (event.key === "Escape") {
        event.preventDefault();
        void closeQuickAccessSurface();
        return;
      }

      if (isTypingTarget) {
        return;
      }

      const currentIndex = Math.max(
        0,
        visibleEntries.findIndex((entry) => entry.id === selectedEntry?.id),
      );

      if (event.key === "ArrowDown" && visibleEntries.length > 0) {
        event.preventDefault();
        const nextIndex = Math.min(visibleEntries.length - 1, currentIndex + 1);
        setSelectedId(visibleEntries[nextIndex]?.id ?? null);
      }

      if (event.key === "ArrowUp" && visibleEntries.length > 0) {
        event.preventDefault();
        const nextIndex = Math.max(0, currentIndex - 1);
        setSelectedId(visibleEntries[nextIndex]?.id ?? null);
      }

      if (event.key === "Enter" && selectedEntry) {
        event.preventDefault();
        void handleCopy(selectedEntry);
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [closeQuickAccessSurface, isLoading, selectedEntry, visibleEntries]);

  async function handleSettingsSave(patch: Partial<AppSettings>) {
    setIsSaving(true);
    try {
      if (patch.launchAtLogin !== undefined && patch.launchAtLogin !== autostartEnabled) {
        if (isTauri && patch.launchAtLogin) {
          await enable();
        } else if (isTauri) {
          await disable();
        }
        setAutostartEnabled(patch.launchAtLogin);
      }

      const nextSettings = await saveSettings(patch);
      setSettings(nextSettings);
      setStatusMessage(messages[resolveUiLanguage(nextSettings.language)].settingsSaved);
      await refreshAllEntries();
      await refreshHistory();
    } finally {
      setIsSaving(false);
    }
  }

  async function handleOnboardingComplete() {
    if (!settings.onboardingCompleted) {
      await handleSettingsSave({ onboardingCompleted: true });
    }
    setOnboardingOpen(false);
  }

  async function closeQuickAccessSurface() {
    await hideQuickAccess();
    if (typeof window !== "undefined") {
      window.location.hash = "";
    }
    setRouteHash("");
  }

  async function handleCopy(entry: HistoryItem) {
    await copyEntry(entry.id);
    setSelectedId(entry.id);
    setStatusMessage(copy.copiedItem(typeLabel(entry.contentType, uiLanguage)));
    if (isQuickAccess) {
      await closeQuickAccessSurface();
    }
  }

  async function handleFavorite(entry: HistoryItem) {
    await toggleFavorite(entry.id, !entry.favorite);
    setStatusMessage(entry.favorite ? copy.favoriteOff : copy.favoriteOn);
    await refreshAllEntries();
    await refreshHistory();
  }

  async function handlePin(entry: HistoryItem) {
    await togglePin(entry.id, !entry.pinned);
    setStatusMessage(entry.pinned ? copy.pinOff : copy.pinOn);
    await refreshAllEntries();
    await refreshHistory();
  }

  async function handleDelete(entry: HistoryItem) {
    await deleteHistoryItems([entry.id]);
    setStatusMessage(copy.entryDeleted);
    await refreshAllEntries();
    await refreshHistory();
  }

  async function handleClear() {
    await clearUnpinnedHistory();
    setStatusMessage(copy.cleared);
    await refreshAllEntries();
    await refreshHistory();
  }

  async function handleTagsSave() {
    if (!selectedEntry) {
      return;
    }

    const nextTags = tagDraft
      .split(",")
      .map((tag) => tag.trim())
      .filter(Boolean);
    await saveTags(selectedEntry.id, nextTags);
    setStatusMessage(copy.tagsUpdated);
    await refreshAllEntries();
    await refreshHistory();
  }

  async function handleExport() {
    setIsPortingData(true);
    try {
      const result = await exportHistory();
      if (!result) {
        setStatusMessage(copy.exportCanceled);
        return;
      }
      setStatusMessage(copy.exportSummary(result.entryCount, result.path.split("/").pop() ?? result.path));
    } finally {
      setIsPortingData(false);
    }
  }

  async function handleImport() {
    setIsPortingData(true);
    try {
      const result = await importHistory(importMode);
      if (!result) {
        setStatusMessage(copy.importCanceled);
        return;
      }
      await refreshAllEntries();
      await refreshHistory();
      setStatusMessage(copy.importSummary(result.mode, result.importedCount, result.skippedCount));
    } finally {
      setIsPortingData(false);
    }
  }

  if (isQuickAccess) {
    return (
      <div className="quick-access-shell">
        <WindowDragStrip className="window-drag-strip" />
        <div className="quick-access-header">
          <div>
            <p className="eyebrow">{copy.quickAccessEyebrow}</p>
            <h2>{copy.quickAccessTitle}</h2>
          </div>
          <button className="ghost-button" onClick={() => void closeQuickAccessSurface()} type="button">
            {copy.close}
          </button>
        </div>

        <div className="quick-search-row">
          <input
            autoFocus
            className="quick-search"
            onChange={(event) => setSearch(event.target.value)}
            placeholder={copy.searchQuickPlaceholder}
            value={search}
          />
          <select className="quick-select" onChange={(event) => setContentType(event.target.value)} value={contentType}>
            {contentFilters.map((filter) => (
              <option key={filter} value={filter}>
                {filter === "all" ? copy.allTypes : typeLabel(filter, uiLanguage)}
              </option>
            ))}
          </select>
        </div>

        <div className="quick-hint-row">
          <span>{copy.quickHintOpen(settings.shortcut)}</span>
          <span>{copy.quickHintNavigate}</span>
          <span>{copy.quickHintEnter}</span>
          <span>{copy.quickHintEsc}</span>
        </div>

        <div className="quick-access-list">
          {visibleEntries.length === 0 ? (
            <div className="empty-state compact">
              <p>{copy.noMatching}</p>
              <span>{copy.widenSearch}</span>
            </div>
          ) : (
            visibleEntries.map((entry) => (
              <button
                className={`quick-row ${selectedEntry?.id === entry.id ? "selected" : ""}`}
                key={entry.id}
                onClick={() => void handleCopy(entry)}
                onFocus={() => setSelectedId(entry.id)}
                type="button"
              >
                <div className="quick-row-top">
                  <span className={`type-pill type-${entry.contentType}`}>{typeLabel(entry.contentType, uiLanguage)}</span>
                  <span className="meta-text">{relativeDateLabel(entry.createdAt, uiLanguage)}</span>
                </div>
                <div className="quick-row-preview">{entry.previewText}</div>
                <div className="quick-row-footer">
                  <span>{entry.tags.length > 0 ? `#${entry.tags.join(" #")}` : copy.noTags}</span>
                  <span>{formatBytes(entry.sizeBytes)}</span>
                </div>
              </button>
            ))
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <div className="app-background" />
      <WindowDragStrip className="window-drag-strip app-drag-strip" />
      <aside className="hero-column glass-panel">
        <div className="brand-lockup">
          <div className="brand-icon">
            <img alt={copy.appIconAlt} src="/app-icon.svg" />
          </div>
          <div>
            <p className="eyebrow">{copy.appEyebrow}</p>
            <h1>CopyTrack</h1>
            <p className="lede">{copy.appDescription}</p>
          </div>
        </div>

        <div className="status-strip">
          <span className={`status-dot ${settings.captureEnabled ? "live" : "paused"}`} />
          <span>{statusMessage}</span>
        </div>

        <div className="stats-grid">
          <StatCard label={copy.saved} value={stats.total} />
          <StatCard label={copy.pinned} value={stats.pinned} />
          <StatCard label={copy.favorites} value={stats.favorites} />
          <StatCard label={copy.limit} value={settings.historyLimit} />
        </div>

        <div className="hero-actions">
          <button className="primary-button" onClick={() => void openQuickAccess()} type="button">
            {copy.openQuickAccess}
          </button>
          <button className="secondary-button" onClick={() => void setSettingsOpen(true)} type="button">
            {copy.openSettings}
          </button>
        </div>

        <div className="feature-list">
          <FeatureLine title={copy.featureShortcut} value={settings.shortcut} />
          <FeatureLine title={copy.featureLaunch} value={autostartEnabled ? copy.enabled : copy.disabled} />
          <FeatureLine title={copy.featureCapture} value={settings.captureEnabled ? copy.recording : copy.paused} />
          <FeatureLine title={copy.featureTheme} value={themeLabel} />
        </div>
      </aside>

      <main className="content-column glass-panel">
        <header className="toolbar">
          <div className="toolbar-copy">
            <p className="eyebrow">{copy.historyEyebrow}</p>
            <h2>{copy.historyTitle}</h2>
          </div>

          <div className="toolbar-actions">
            <label className="search-field" htmlFor="history-search">
              <span>{copy.search}</span>
              <input
                id="history-search"
                onChange={(event) => setSearch(event.target.value)}
                placeholder={copy.searchPlaceholder}
                value={search}
              />
            </label>
            <label className="sort-field" htmlFor="sort-mode">
              <span>{copy.sort}</span>
              <select id="sort-mode" onChange={(event) => setSortMode(event.target.value as SortMode)} value={sortMode}>
                <option value="recent">{copy.sortRecent}</option>
                <option value="favorites">{copy.sortFavorites}</option>
                <option value="type">{copy.sortType}</option>
                <option value="oldest">{copy.sortOldest}</option>
              </select>
            </label>
            <button className="ghost-button" onClick={() => void handleClear()} type="button">
              {copy.clearUnpinned}
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
              {filter === "all" ? copy.all : typeLabel(filter, uiLanguage)}
            </button>
          ))}
          <button
            className={`toggle-chip ${onlyFavorites ? "active" : ""}`}
            onClick={() => setOnlyFavorites((current) => !current)}
            type="button"
          >
            {copy.favorites}
          </button>
          <button
            className={`toggle-chip ${onlyPinned ? "active" : ""}`}
            onClick={() => setOnlyPinned((current) => !current)}
            type="button"
          >
            {copy.pinned}
          </button>
          {availableTags.map((tag) => (
            <button
              className={`tag-chip ${selectedTag === tag ? "active" : ""}`}
              key={tag}
              onClick={() => setSelectedTag((current) => (current === tag ? "" : tag))}
              type="button"
            >
              #{tag}
            </button>
          ))}
        </div>

        <div className="workspace-grid">
          <section className="history-list">
            {isLoading ? (
              <div className="empty-state">{copy.preparing}</div>
            ) : visibleEntries.length === 0 ? (
              <div className="empty-state">
                <p>{copy.noEntries}</p>
                <span>{copy.noEntriesHint}</span>
              </div>
            ) : (
              visibleEntries.map((entry) => (
                <button
                  className={`history-row ${selectedEntry?.id === entry.id ? "selected" : ""}`}
                  key={entry.id}
                  onClick={() => void handleCopy(entry)}
                  type="button"
                >
                  <div className="history-row-top">
                    <span className={`type-pill type-${entry.contentType}`}>{typeLabel(entry.contentType, uiLanguage)}</span>
                    <span className="meta-text">{relativeDateLabel(entry.createdAt, uiLanguage)}</span>
                  </div>
                  <div className="history-row-preview">{entry.previewText}</div>
                  <div className="history-row-footer">
                    <span>{entry.tags.length > 0 ? `#${entry.tags.join(" #")}` : formatBytes(entry.sizeBytes)}</span>
                    <div className="history-row-actions">
                      <ActionBadge
                        active={entry.favorite}
                        label={entry.favorite ? copy.removeFavorite : copy.addFavorite}
                        onClick={() => void handleFavorite(entry)}
                      />
                      <ActionBadge
                        active={entry.pinned}
                        label={entry.pinned ? copy.unpinAction : copy.pinAction}
                        onClick={() => void handlePin(entry)}
                      />
                      <ActionBadge active={false} label={copy.deleteAction} onClick={() => void handleDelete(entry)} />
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
                    <p className="eyebrow">{copy.preview}</p>
                    <h3>{selectedEntry.previewText}</h3>
                  </div>
                  <span className={`type-pill type-${selectedEntry.contentType}`}>{typeLabel(selectedEntry.contentType, uiLanguage)}</span>
                </div>

                {selectedEntry.imagePath ? (
                  <div className="image-preview">
                    <img alt={selectedEntry.previewText} src={convertFileSrc(selectedEntry.imagePath)} />
                  </div>
                ) : (
                  <pre className="preview-code">{selectedEntry.fullText ?? selectedEntry.filePaths.join("\n")}</pre>
                )}

                <div className="preview-metadata">
                  <MetaLine label={copy.copied} value={relativeDateLabel(selectedEntry.createdAt, uiLanguage)} />
                  <MetaLine label={copy.type} value={typeLabel(selectedEntry.contentType, uiLanguage)} />
                  <MetaLine label={copy.sourceApp} value={selectedEntry.sourceApp ?? copy.unavailable} />
                  <MetaLine label={copy.size} value={formatBytes(selectedEntry.sizeBytes)} />
                </div>

                <label className="tag-editor" htmlFor="tag-editor">
                  <span>{copy.tags}</span>
                  <input
                    id="tag-editor"
                    onBlur={() => void handleTagsSave()}
                    onChange={(event) => setTagDraft(event.target.value)}
                    placeholder={copy.tagPlaceholder}
                    value={tagDraft}
                  />
                </label>

                <div className="preview-actions">
                  <button className="primary-button" onClick={() => void handleCopy(selectedEntry)} type="button">
                    {copy.copyAgain}
                  </button>
                  <button className="secondary-button" onClick={() => void handleFavorite(selectedEntry)} type="button">
                    {selectedEntry.favorite ? copy.removeFavorite : copy.addFavorite}
                  </button>
                </div>
              </>
            ) : (
              <div className="empty-state">
                <p>{copy.nothingSelected}</p>
                <span>{copy.nothingSelectedHint}</span>
              </div>
            )}
          </section>
        </div>
      </main>

      {settingsOpen ? (
        <div className="settings-sheet">
          <div className="settings-card glass-panel">
            <WindowDragStrip className="window-drag-strip modal-drag-strip" />
            <div className="settings-header">
              <div>
                <p className="eyebrow">{copy.preferencesEyebrow}</p>
                <h3>{copy.preferencesTitle}</h3>
              </div>
              <button className="ghost-button" onClick={() => setSettingsOpen(false)} type="button">
                {copy.close}
              </button>
            </div>

            <div className="settings-grid">
              <label>
                <span>{copy.captureStatus}</span>
                <select
                  onChange={(event) => void handleSettingsSave({ captureEnabled: event.target.value === "enabled" })}
                  value={settings.captureEnabled ? "enabled" : "paused"}
                >
                  <option value="enabled">{copy.enabled}</option>
                  <option value="paused">{copy.paused}</option>
                </select>
              </label>

              <label>
                <span>{copy.historyLimit}</span>
                <select
                  onChange={(event) => void handleSettingsSave({ historyLimit: Number(event.target.value) })}
                  value={settings.historyLimit}
                >
                  {supportedLimits.map((value) => (
                    <option key={value} value={value}>
                      {copy.itemsCount(value)}
                    </option>
                  ))}
                </select>
              </label>

              <label>
                <span>{copy.shortcut}</span>
                <input
                  onBlur={(event) => void handleSettingsSave({ shortcut: event.target.value || emptySettings.shortcut })}
                  onChange={(event) => setSettings((current) => ({ ...current, shortcut: event.target.value }))}
                  placeholder={emptySettings.shortcut}
                  type="text"
                  value={settings.shortcut}
                />
              </label>

              <label>
                <span>{copy.launchAtLogin}</span>
                <select
                  onChange={(event) => void handleSettingsSave({ launchAtLogin: event.target.value === "enabled" })}
                  value={autostartEnabled ? "enabled" : "disabled"}
                >
                  <option value="enabled">{copy.enabled}</option>
                  <option value="disabled">{copy.disabled}</option>
                </select>
              </label>

              <label>
                <span>{copy.theme}</span>
                <select onChange={(event) => void handleSettingsSave({ theme: event.target.value })} value={settings.theme}>
                  <option value="system">{copy.system}</option>
                  <option value="light">{copy.light}</option>
                  <option value="dark">{copy.dark}</option>
                </select>
              </label>

              <label>
                <span>{copy.language}</span>
                <select onChange={(event) => void handleSettingsSave({ language: event.target.value })} value={settings.language}>
                  <option value="system">{copy.system}</option>
                  <option value="en">{copy.english}</option>
                  <option value="ru">{copy.russian}</option>
                </select>
              </label>

              <label className="full-width">
                <span>{copy.excludedApps}</span>
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
                  placeholder={copy.excludedPlaceholder}
                  value={settings.excludedApps.join("\n")}
                />
              </label>
            </div>

            <div className="settings-guidance">
              <div className="settings-guidance-card">
                <strong>{copy.portabilityTitle}</strong>
                <span>{copy.portabilityBody}</span>
              </div>
              <div className="settings-guidance-card">
                <strong>{copy.clipboardAccessTitle}</strong>
                <span>{copy.clipboardAccessBody}</span>
              </div>
              <div className="settings-guidance-card">
                <strong>{copy.menuBarTitle}</strong>
                <span>{copy.menuBarBody}</span>
              </div>
              <div className="settings-guidance-card">
                <strong>{copy.loginItemTitle}</strong>
                <span>{copy.loginItemBody}</span>
              </div>
            </div>

            <div className="portability-panel">
              <div className="portability-copy">
                <strong>{copy.portabilityTitle}</strong>
                <span>{copy.portabilityBody}</span>
              </div>
              <div className="portability-actions">
                <label className="portability-select" htmlFor="import-mode">
                  <span>{copy.importMode}</span>
                  <select id="import-mode" onChange={(event) => setImportMode(event.target.value as ImportMode)} value={importMode}>
                    <option value="merge">{copy.importMerge}</option>
                    <option value="replace">{copy.importReplace}</option>
                  </select>
                </label>
                <button className="secondary-button" disabled={isPortingData} onClick={() => void handleExport()} type="button">
                  {copy.exportHistory}
                </button>
                <button className="primary-button" disabled={isPortingData} onClick={() => void handleImport()} type="button">
                  {isPortingData ? copy.working : copy.importHistory}
                </button>
              </div>
            </div>

            <div className="settings-actions-row">
              <button className="secondary-button" onClick={() => setOnboardingOpen(true)} type="button">
                {copy.showOnboarding}
              </button>
            </div>

            <p className="settings-note">{copy.settingsNote(isSaving)}</p>
          </div>
        </div>
      ) : null}

      {onboardingOpen ? (
        <div className="settings-sheet onboarding-sheet">
          <div className="onboarding-card glass-panel">
            <WindowDragStrip className="window-drag-strip modal-drag-strip" />
            <div className="settings-header">
              <div>
                <p className="eyebrow">{copy.onboardingEyebrow}</p>
                <h3>{copy.onboardingTitle}</h3>
                <p className="lede onboarding-lede">{copy.onboardingBody}</p>
              </div>
              <button className="ghost-button" onClick={() => void handleOnboardingComplete()} type="button">
                {copy.close}
              </button>
            </div>

            <div className="onboarding-grid">
              <OnboardingCard title={copy.onboardingPermissionTitle} body={copy.onboardingPermissionBody} />
              <OnboardingCard title={copy.onboardingMenuBarStepTitle} body={copy.onboardingMenuBarStepBody} />
              <OnboardingCard title={copy.onboardingShortcutStepTitle} body={copy.onboardingShortcutStepBody(settings.shortcut)} />
            </div>

            <div className="onboarding-actions">
              <button className="secondary-button" onClick={() => void openQuickAccess()} type="button">
                {copy.onboardingOpenQuickAccess}
              </button>
              <button
                className="secondary-button"
                onClick={() => {
                  setSettingsOpen(true);
                  setOnboardingOpen(false);
                }}
                type="button"
              >
                {copy.onboardingOpenSettings}
              </button>
              <button className="primary-button" onClick={() => void handleOnboardingComplete()} type="button">
                {copy.onboardingFinish}
              </button>
            </div>
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

function OnboardingCard({ title, body }: { title: string; body: string }) {
  return (
    <div className="onboarding-step">
      <strong>{title}</strong>
      <span>{body}</span>
    </div>
  );
}

function WindowDragStrip({ className = "" }: { className?: string }) {
  return (
    <div
      className={`window-drag-strip-handle ${className}`.trim()}
      data-tauri-drag-region
      onMouseDown={() => {
        const windowHandle = getRuntimeWindow();
        if (windowHandle) {
          void windowHandle.startDragging().catch(() => undefined);
        }
      }}
    />
  );
}
