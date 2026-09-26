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
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
    this.dispatchEvent(new Event("close"));
  };
  invoke.mockReset();
  invoke.mockImplementation((command: string) => {
    if (command === "list_quizzes" || command === "browse_teachers")
      return Promise.resolve([]);
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
  it("loads the AI graphs only after visiting chat", async () => {
    render(<App />);
    fireEvent.click(
      screen.getByRole("button", { name: "先生として始める画面を開く" }),
    );
    await screen.findByRole("button", { name: "AIチャット" });
    expect(
      invoke.mock.calls.some(([name]) => name === "preload_active_model"),
    ).toBe(false);
    fireEvent.click(screen.getByRole("button", { name: "AIチャット" }));
    await waitFor(() =>
      expect(
        invoke.mock.calls.some(([name]) => name === "preload_active_model"),
      ).toBe(true),
    );
  });

  it("is reachable from the bar before a role is chosen", async () => {
    render(<App />);
    expect(screen.getByText("先生として始める")).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "操作" }));
    fireEvent.click(
      await screen.findByRole("button", { name: /使用中のモデル/ }),
    );

    expect(await screen.findByRole("heading", { name: "設定" })).toBeDefined();
    expect(screen.getByText("モデル管理")).toBeDefined();
    // Role selection steps aside while settings is open.
    expect(
      screen.queryByRole("button", { name: "先生として始める画面を開く" }),
    ).toBeNull();
  });

  it("uses a readable model storage label on iPhone", async () => {
    invoke.mockImplementation((command: string) => {
      if (command === "list_quizzes" || command === "browse_teachers")
        return Promise.resolve([]);
      if (command === "list_models") return Promise.resolve(MODELS);
      if (command === "get_system_info") {
        return Promise.resolve({
          platform: "ios",
          arch: "aarch64",
          tauri_version: "2",
          ort_available: true,
          model_dir:
            "/private/var/mobile/Containers/Data/Application/DEVICE-UUID/Library/Application Support/models",
        });
      }
      return Promise.resolve(null);
    });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "操作" }));
    fireEvent.click(
      await screen.findByRole("button", { name: /使用中のモデル/ }),
    );

    const value = await screen.findByText("このiPhone内（アプリ専用領域）");
    expect(value.getAttribute("title")).toBeNull();
    expect(screen.queryByText(/private\/var\/mobile/)).toBeNull();
  });

  it("returns to the same place on close, keeping the chosen role", async () => {
    render(<App />);
    fireEvent.click(screen.getByText("先生として始める"));
    await waitFor(() => expect(screen.getByText("先生")).toBeDefined());

    fireEvent.click(screen.getByRole("button", { name: "操作" }));
    fireEvent.click(
      await screen.findByRole("button", { name: /使用中のモデル/ }),
    );
    await screen.findByRole("heading", { name: "設定" });
    // The role is not lost while settings is open.
    expect(screen.getByText("先生")).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "閉じる" }));
    await waitFor(() =>
      expect(screen.queryByRole("heading", { name: "設定" })).toBeNull(),
    );
    expect(screen.getByText("先生")).toBeDefined();
  });

  it("no longer offers model management as a teacher-only tab", async () => {
    render(<App />);
    fireEvent.click(screen.getByText("先生として始める"));
    await waitFor(() => expect(screen.getByText("先生")).toBeDefined());
    // The model is shared with the students' hints, so it is not a tab here.
    expect(screen.queryByRole("button", { name: "モデル管理" })).toBeNull();
  });

  it("leaves the role switch to the app bar alone", async () => {
    render(<App />);
    fireEvent.click(screen.getByText("先生として始める"));
    await waitFor(() => expect(screen.getByText("先生")).toBeDefined());

    // One way out of the role, in the bar next to the badge that names it -
    // the sidebar carries only the teacher's own tabs.
    const switches = screen.getAllByRole("button", {
      name: "役割選択へ戻る",
    });
    expect(switches).toHaveLength(1);
    expect(switches[0].closest("header")).not.toBeNull();

    const nav = document.querySelector("nav") as HTMLElement;
    expect(
      Array.from(nav.querySelectorAll("button")).map((b) => b.textContent),
    ).toEqual(["概要", "セッション", "レッスンプラン", "AIチャット"]);
  });
  it("keeps the chat draft and active tab across settings and teacher tabs", async () => {
    render(<App />);
    fireEvent.click(screen.getByText("先生として始める"));
    fireEvent.click(screen.getByRole("button", { name: "AIチャット" }));
    fireEvent.change(screen.getByPlaceholderText("学習状況について質問"), {
      target: { value: "未送信の質問" },
    });
    fireEvent.click(screen.getByRole("button", { name: "操作" }));
    fireEvent.click(
      await screen.findByRole("button", { name: /使用中のモデル/ }),
    );
    fireEvent.click(await screen.findByRole("button", { name: "閉じる" }));
    expect((screen.getByRole("textbox") as HTMLTextAreaElement).value).toBe(
      "未送信の質問",
    );
    fireEvent.click(screen.getByRole("button", { name: "概要" }));
    fireEvent.click(screen.getByRole("button", { name: "AIチャット" }));
    expect((screen.getByRole("textbox") as HTMLTextAreaElement).value).toBe(
      "未送信の質問",
    );
  });

  it("keeps partially entered classroom details when settings closes", async () => {
    render(<App />);
    fireEvent.click(screen.getByText("生徒として参加"));
    fireEvent.change(screen.getByPlaceholderText("IPアドレス"), {
      target: { value: "192.168.1.20" },
    });
    fireEvent.click(screen.getByRole("button", { name: "操作" }));
    fireEvent.click(
      await screen.findByRole("button", { name: /使用中のモデル/ }),
    );
    fireEvent.click(await screen.findByRole("button", { name: "閉じる" }));
    expect(
      (screen.getByPlaceholderText("IPアドレス") as HTMLInputElement).value,
    ).toBe("192.168.1.20");
  });
});
