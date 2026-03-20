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
