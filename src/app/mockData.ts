import type { AppSettings, BootstrapPayload, HistoryItem } from "../shared/types/history";

const createdAt = new Date();

function minutesAgo(minutes: number) {
  return new Date(createdAt.getTime() - minutes * 60_000).toISOString();
}

export const mockSettings: AppSettings = {
  captureEnabled: true,
  historyLimit: 500,
  shortcut: "CommandOrControl+Shift+V",
  theme: "system",
  language: "system",
  excludedApps: ["com.1password.1password"],
  launchAtLogin: true,
};

export const mockEntries: HistoryItem[] = [
  {
    id: "design-spec",
    contentType: "text",
    previewText: "Finalize CopyTrack onboarding checklist",
    fullText: "Finalize CopyTrack onboarding checklist\n- Explain clipboard permission\n- Explain menu bar access\n- Show quick access shortcut",
    imagePath: null,
    filePaths: [],
    sourceApp: "Notes",
    createdAt: minutesAgo(2),
    favorite: true,
    pinned: true,
    tags: ["onboarding", "product"],
    sizeBytes: 124,
  },
  {
    id: "link-docs",
    contentType: "link",
    previewText: "https://tauri.app/start/",
    fullText: "https://tauri.app/start/",
    imagePath: null,
    filePaths: [],
    sourceApp: "Safari",
    createdAt: minutesAgo(7),
    favorite: false,
    pinned: false,
    tags: ["docs"],
    sizeBytes: 25,
  },
  {
    id: "snippet",
    contentType: "text",
    previewText: "npm run tauri build -- --debug",
    fullText: "npm run tauri build -- --debug",
    imagePath: null,
    filePaths: [],
    sourceApp: "Terminal",
    createdAt: minutesAgo(15),
    favorite: true,
    pinned: false,
    tags: ["build", "reusable"],
    sizeBytes: 31,
  },
  {
    id: "file-copy",
    contentType: "file",
    previewText: "File: CopyTrack_0.1.0_aarch64.dmg",
    fullText: "/Users/max/PycharmProjects/CopyTrack/src-tauri/target/debug/bundle/dmg/CopyTrack_0.1.0_aarch64.dmg",
    imagePath: null,
    filePaths: ["/Users/max/PycharmProjects/CopyTrack/src-tauri/target/debug/bundle/dmg/CopyTrack_0.1.0_aarch64.dmg"],
    sourceApp: "Finder",
    createdAt: minutesAgo(35),
    favorite: false,
    pinned: false,
    tags: ["release"],
    sizeBytes: 103,
  },
];

export const mockBootstrap: BootstrapPayload = {
  entries: mockEntries,
  settings: mockSettings,
  supportedHistoryLimits: [50, 100, 500, 1000, 10000],
  defaultShortcut: "CommandOrControl+Shift+V",
};
