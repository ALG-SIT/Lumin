import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../../App";

const invoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invoke(...args),
  Channel: class {
    onmessage: (message: unknown) => void = () => {};
  },
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

const MODELS = [
  {
    id: "1b-int4",
    name: "Gemma 3 1B INT4",
    size_bytes: 879_776_749,
    variant: "1b-int4",
    family: "Gemma 3",
    description: "軽量・高速。",
    status: "installed",
    recommended: true,
    active: true,
  },
];

afterEach(cleanup);
beforeEach(() => {
  invoke.mockReset();
  invoke.mockImplementation((command: string) => {
    if (command === "list_models") return Promise.resolve(MODELS);
    if (command === "get_system_info") {
      return Promise.resolve({
        platform: "macos",
        arch: "aarch64",
        tauri_version: "2",
        ort_available: true,
        model_dir: "/tmp/models",
      });
    }
    return Promise.resolve(null);
  });
  Object.defineProperty(window, "__TAURI_INTERNALS__", {
    value: {},
    configurable: true,
  });
});

describe("settings as an app-wide screen", () => {
  it("is reachable from the bar before a role is chosen", async () => {
    render(<App />);
    expect(screen.getByText("先生として始める")).toBeDefined();

    fireEvent.click(
      await screen.findByRole("button", { name: /使用中のモデル/ }),
    );

    expect(await screen.findByRole("heading", { name: "設定" })).toBeDefined();
    expect(screen.getByText("モデル管理")).toBeDefined();
    // Role selection steps aside while settings is open.
    expect(screen.queryByText("先生として始める")).toBeNull();
  });

  it("returns to the same place on close, keeping the chosen role", async () => {
    render(<App />);
    fireEvent.click(screen.getByText("先生として始める"));
    await waitFor(() =>
      expect(screen.getByText("現在の役割: 先生")).toBeDefined(),
    );

    fireEvent.click(
      await screen.findByRole("button", { name: /使用中のモデル/ }),
    );
    await screen.findByRole("heading", { name: "設定" });
    // The role is not lost while settings is open.
    expect(screen.getByText("現在の役割: 先生")).toBeDefined();

    fireEvent.click(screen.getByText("閉じる"));
    await waitFor(() =>
      expect(screen.queryByRole("heading", { name: "設定" })).toBeNull(),
    );
    expect(screen.getByText("現在の役割: 先生")).toBeDefined();
  });

  it("no longer offers model management as a teacher-only tab", async () => {
    render(<App />);
    fireEvent.click(screen.getByText("先生として始める"));
    await waitFor(() =>
      expect(screen.getByText("現在の役割: 先生")).toBeDefined(),
    );
    // The model is shared with the students' hints, so it is not a tab here.
    expect(screen.queryByRole("button", { name: "モデル管理" })).toBeNull();
  });

  it("leaves the role switch to the app bar alone", async () => {
    render(<App />);
    fireEvent.click(screen.getByText("先生として始める"));
    await waitFor(() =>
      expect(screen.getByText("現在の役割: 先生")).toBeDefined(),
    );

    // One way out of the role, in the bar next to the badge that names it -
    // the sidebar carries only the teacher's own tabs.
    const switches = screen.getAllByRole("button", {
      name: "役割を切り替える",
    });
    expect(switches).toHaveLength(1);
    expect(switches[0].closest("header")).not.toBeNull();

    const nav = document.querySelector("nav") as HTMLElement;
    expect(
      Array.from(nav.querySelectorAll("button")).map((b) => b.textContent),
    ).toEqual(["概要", "セッション", "レッスンプラン", "AIチャット"]);
  });
});
