import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppBarModelStatus } from "../AppBarModelStatus";

const invoke = vi.fn();
const listeners = new Map<string, (event: { payload: unknown }) => void>();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: (event: string, handler: (e: { payload: unknown }) => void) => {
    listeners.set(event, handler);
    return Promise.resolve(() => listeners.delete(event));
  },
}));

const INSTALLED = {
  id: "1b-int4",
  name: "Gemma 3 1B INT4",
  size_bytes: 879_776_749,
  variant: "1b-int4",
  family: "Gemma 3",
  description: "軽量・高速。",
  status: "installed",
  active: true,
};
const NOT_INSTALLED = {
  ...INSTALLED,
  id: "4-e4b-int4",
  variant: "4-e4b-int4",
  name: "Gemma 4 E4B INT4",
  status: "available",
};

afterEach(cleanup);
beforeEach(() => {
  invoke.mockReset();
  listeners.clear();
  Object.defineProperty(window, "__TAURI_INTERNALS__", {
    value: {},
    configurable: true,
  });
});

const emit = (event: string, payload: unknown) =>
  act(async () => {
    listeners.get(event)?.({ payload });
  });

describe("AppBarModelStatus", () => {
  it("names the active model and says it is usable", async () => {
    invoke.mockResolvedValue([INSTALLED]);
    render(<AppBarModelStatus onOpenSettings={() => {}} />);

    await screen.findByText("Gemma 3 1B INT4");
    expect(screen.getByText("使用できます")).toBeDefined();
    // Nothing is loading, so there is no bar to show.
    expect(screen.queryByRole("progressbar")).toBeNull();
  });

  it("flags a model that is not installed yet", async () => {
    invoke.mockResolvedValue([NOT_INSTALLED]);
    render(<AppBarModelStatus onOpenSettings={() => {}} />);
    expect(await screen.findByText(/未導入/)).toBeDefined();
  });

  it("shows load progress from any screen, then clears it when ready", async () => {
    invoke.mockResolvedValue([INSTALLED]);
    render(<AppBarModelStatus onOpenSettings={() => {}} />);
    await screen.findByText("Gemma 3 1B INT4");

    await emit("model-load-progress", {
      variant: "1b-int4",
      modelName: "Gemma 3 1B INT4",
      stage: "decoder",
      label: "モデル本体をメモリに読み込み中…",
      percent: 40,
      done: false,
    });

    const bar = screen.getByRole("progressbar", { name: "モデルの読み込み" });
    expect(bar.getAttribute("aria-valuenow")).toBe("40");
    expect(
      screen.getByText(/モデル本体をメモリに読み込み中… 40%/),
    ).toBeDefined();

    await emit("model-load-progress", {
      variant: "1b-int4",
      modelName: "Gemma 3 1B INT4",
      stage: "ready",
      label: "モデルの準備ができました",
      percent: 100,
      done: true,
    });
    expect(screen.queryByRole("progressbar")).toBeNull();
    expect(screen.getByText("使用できます")).toBeDefined();
  });

  it("follows a model switch made elsewhere", async () => {
    invoke.mockResolvedValue([INSTALLED]);
    render(<AppBarModelStatus onOpenSettings={() => {}} />);
    await screen.findByText("Gemma 3 1B INT4");

    await emit("active-model-changed", {
      ...NOT_INSTALLED,
      status: "installed",
      name: "Gemma 4 E2B INT4",
    });
    expect(screen.getByText("Gemma 4 E2B INT4")).toBeDefined();
  });

  it("opens settings when used", async () => {
    const onOpenSettings = vi.fn();
    invoke.mockResolvedValue([INSTALLED]);
    render(<AppBarModelStatus onOpenSettings={onOpenSettings} />);

    const chip = await screen.findByRole("button", { name: /使用中のモデル/ });
    chip.click();
    await waitFor(() => expect(onOpenSettings).toHaveBeenCalledOnce());
  });
});
