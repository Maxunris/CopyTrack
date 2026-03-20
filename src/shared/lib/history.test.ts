import { describe, expect, it } from "vitest";

import { collectTags, filterEntries, getHistoryStats, sortEntries, typeLabel } from "./history";
import type { HistoryItem } from "../types/history";

const items: HistoryItem[] = [
  {
    id: "1",
    contentType: "text",
    previewText: "Release notes",
    fullText: "Prepare release notes for v1",
    imagePath: null,
    filePaths: [],
    sourceApp: "Notes",
    createdAt: "2026-03-20T10:00:00.000Z",
    favorite: true,
    pinned: false,
    tags: ["release"],
    sizeBytes: 120,
  },
  {
    id: "2",
    contentType: "image",
    previewText: "Image 1440x900",
    fullText: null,
    imagePath: "/tmp/example.png",
    filePaths: [],
    sourceApp: "Preview",
    createdAt: "2026-03-20T10:05:00.000Z",
    favorite: false,
    pinned: true,
    tags: [],
    sizeBytes: 2048,
  },
];

describe("history utilities", () => {
  it("filters by search and favorite flags", () => {
    const result = filterEntries(items, "release", true, false, "all");
    expect(result).toHaveLength(1);
    expect(result[0]?.id).toBe("1");
  });

  it("filters by selected tag", () => {
    const result = filterEntries(items, "", false, false, "all", "release");
    expect(result).toHaveLength(1);
    expect(result[0]?.id).toBe("1");
  });

  it("computes entry stats", () => {
    expect(getHistoryStats(items)).toEqual({
      total: 2,
      text: 1,
      link: 0,
      image: 1,
      file: 0,
      favorites: 1,
      pinned: 1,
    });
  });

  it("maps type labels", () => {
    expect(typeLabel("image")).toBe("Image");
    expect(typeLabel("link")).toBe("Link");
    expect(typeLabel("text")).toBe("Text");
  });

  it("sorts by favorites and collects tags", () => {
    expect(sortEntries(items, "favorites")[0]?.id).toBe("1");
    expect(collectTags(items)).toEqual(["release"]);
  });
});
