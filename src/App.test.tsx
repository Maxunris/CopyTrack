import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
  save: vi.fn().mockResolvedValue(null),
}));

import App from "./App";

describe("App", () => {
  beforeEach(() => {
    window.location.hash = "";
    window.history.replaceState({}, "", "/");
  });

  afterEach(() => {
    cleanup();
    window.location.hash = "";
    window.history.replaceState({}, "", "/");
  });

  it("renders the main workspace in browser mock mode", async () => {
    render(<App />);

    expect(await screen.findByRole("heading", { name: "CopyTrack" })).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "Find anything you copied" })).toBeInTheDocument();
    expect(await screen.findByText("Open Quick Access")).toBeInTheDocument();
  });

  it("shows import and export actions in settings", async () => {
    render(<App />);

    await userEvent.click(screen.getByRole("button", { name: "Open Settings" }));

    expect(await screen.findByRole("button", { name: "Export History" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import History" })).toBeInTheDocument();
  });

  it("renders the quick access surface when the quick-access hash is active", async () => {
    window.location.hash = "#quick-access";

    render(<App />);

    expect(await screen.findByRole("heading", { name: "CopyTrack popup" })).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Search copied content")).toBeInTheDocument();
    expect(screen.getByText("Arrow keys to navigate")).toBeInTheDocument();
  });

  it("renders russian copy when the preview lang is forced", async () => {
    window.history.replaceState({}, "", "/?lang=ru");

    render(<App />);

    expect(await screen.findByRole("heading", { name: "Найди все, что копировал" })).toBeInTheDocument();
    expect(screen.getByText("Открыть быстрый доступ")).toBeInTheDocument();
  });

  it("opens onboarding when the onboarding preview panel is requested", async () => {
    window.history.replaceState({}, "", "/?panel=onboarding");

    render(<App />);

    expect(await screen.findByRole("heading", { name: "Set up CopyTrack in a minute" })).toBeInTheDocument();
    expect(screen.getByText("Try Quick Access")).toBeInTheDocument();
  });
});
