import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => path,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => undefined),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: vi.fn(() => null),
}));

vi.mock("@tauri-apps/plugin-autostart", () => ({
  disable: vi.fn().mockResolvedValue(undefined),
  enable: vi.fn().mockResolvedValue(undefined),
  isEnabled: vi.fn().mockResolvedValue(false),
}));

import App from "./App";

describe("App", () => {
  beforeEach(() => {
    window.location.hash = "";
  });

  afterEach(() => {
    cleanup();
    window.location.hash = "";
  });

  it("renders the main workspace in browser mock mode", async () => {
    render(<App />);

    expect(await screen.findByRole("heading", { name: "CopyTrack" })).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "Find anything you copied" })).toBeInTheDocument();
    expect(await screen.findByText("Open Quick Access")).toBeInTheDocument();
  });

  it("renders the quick access surface when the quick-access hash is active", async () => {
    window.location.hash = "#quick-access";

    render(<App />);

    expect(await screen.findByRole("heading", { name: "CopyTrack popup" })).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Search copied content")).toBeInTheDocument();
    expect(screen.getByText("Arrow keys to navigate")).toBeInTheDocument();
  });
});
