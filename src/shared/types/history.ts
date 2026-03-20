export type HistoryItem = {
  id: string;
  contentType: string;
  previewText: string;
  fullText: string | null;
  imagePath: string | null;
  filePaths: string[];
  sourceApp: string | null;
  createdAt: string;
  favorite: boolean;
  pinned: boolean;
  tags: string[];
  sizeBytes: number;
};

export type AppSettings = {
  captureEnabled: boolean;
  historyLimit: number;
  shortcut: string;
  theme: string;
  language: string;
  onboardingCompleted: boolean;
  excludedApps: string[];
  launchAtLogin: boolean;
};

export type HistoryQuery = {
  search?: string;
  contentType?: string;
  onlyFavorites?: boolean;
  onlyPinned?: boolean;
};

export type BootstrapPayload = {
  entries: HistoryItem[];
  settings: AppSettings;
  supportedHistoryLimits: number[];
  defaultShortcut: string;
};

export type ImportMode = "merge" | "replace";

export type ExportSummary = {
  path: string;
  entryCount: number;
};

export type ImportSummary = {
  path: string;
  importedCount: number;
  skippedCount: number;
  mode: ImportMode;
};
