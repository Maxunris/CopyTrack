import type { HistoryItem } from "../types/history";

export type SortMode = "recent" | "oldest" | "favorites" | "type";

export function typeLabel(contentType: string) {
  switch (contentType) {
    case "link":
      return "Link";
    case "image":
      return "Image";
    case "file":
      return "File";
    default:
      return "Text";
  }
}

export function relativeDateLabel(value: string) {
  const createdAt = new Date(value).getTime();
  const deltaMinutes = Math.max(0, Math.floor((Date.now() - createdAt) / 60000));

  if (deltaMinutes < 1) {
    return "just now";
  }
  if (deltaMinutes < 60) {
    return `${deltaMinutes}m ago`;
  }

  const deltaHours = Math.floor(deltaMinutes / 60);
  if (deltaHours < 24) {
    return `${deltaHours}h ago`;
  }

  const deltaDays = Math.floor(deltaHours / 24);
  if (deltaDays < 7) {
    return `${deltaDays}d ago`;
  }

  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
  }).format(new Date(value));
}

export function formatBytes(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function getHistoryStats(entries: HistoryItem[]) {
  return entries.reduce(
    (stats, entry) => {
      stats.total += 1;
      stats[entry.contentType as "text" | "link" | "image" | "file"] += 1;
      if (entry.favorite) {
        stats.favorites += 1;
      }
      if (entry.pinned) {
        stats.pinned += 1;
      }
      return stats;
    },
    {
      total: 0,
      text: 0,
      link: 0,
      image: 0,
      file: 0,
      favorites: 0,
      pinned: 0,
    },
  );
}

export function filterEntries(
  entries: HistoryItem[],
  search: string,
  onlyFavorites: boolean,
  onlyPinned: boolean,
  contentType: string,
  selectedTag = "",
) {
  const normalizedSearch = search.trim().toLowerCase();
  const normalizedTag = selectedTag.trim().toLowerCase();

  return entries.filter((entry) => {
    if (onlyFavorites && !entry.favorite) {
      return false;
    }
    if (onlyPinned && !entry.pinned) {
      return false;
    }
    if (contentType !== "all" && entry.contentType !== contentType) {
      return false;
    }
    if (normalizedTag && !entry.tags.map((tag) => tag.toLowerCase()).includes(normalizedTag)) {
      return false;
    }
    if (!normalizedSearch) {
      return true;
    }

    const haystack = [
      entry.previewText,
      entry.fullText ?? "",
      entry.sourceApp ?? "",
      entry.filePaths.join(" "),
      entry.tags.join(" "),
    ]
      .join(" ")
      .toLowerCase();

    return haystack.includes(normalizedSearch);
  });
}

export function sortEntries(entries: HistoryItem[], sortMode: SortMode) {
  return [...entries].sort((left, right) => {
    if (sortMode === "oldest") {
      return new Date(left.createdAt).getTime() - new Date(right.createdAt).getTime();
    }
    if (sortMode === "favorites") {
      return Number(right.favorite) - Number(left.favorite) || new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime();
    }
    if (sortMode === "type") {
      return left.contentType.localeCompare(right.contentType) || new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime();
    }
    return new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime();
  });
}

export function collectTags(entries: HistoryItem[]) {
  return [...new Set(entries.flatMap((entry) => entry.tags.map((tag) => tag.trim()).filter(Boolean)))].sort((left, right) =>
    left.localeCompare(right),
  );
}
