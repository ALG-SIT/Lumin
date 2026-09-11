// @vitest-environment jsdom

import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ModelManager } from "../ModelManager";
import { modelCard } from "../ModelManager.css.ts";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

// Shape returned by the Rust `list_models` command.
const MODELS_FIXTURE = [
  {
    id: "1b-int4",
    name: "Gemma 3 1B INT4",
    size_bytes: 879_776_749,
    variant: "1b-int4",
    family: "Gemma 3",
    description: "軽量・高速。まず試すならこれ。",
    status: "installed",
    recommended: true,
    active: true,
  },
  {
    id: "4-e2b-int4",
    name: "Gemma 4 E2B INT4",
    size_bytes: 3_646_851_160,
    variant: "4-e2b-int4",
    family: "Gemma 4",
    description: "Gemma 4 の軽量版。教室端末での常用を想定。",
    status: "installed",
    recommended: true,
    active: false,
  },
  {
    id: "4-e4b-int4",
    name: "Gemma 4 E4B INT4",
    size_bytes: 5_635_731_854,
    variant: "4-e4b-int4",
    family: "Gemma 4",
    description: "Gemma 4 の高精度版。メモリに余裕のある教師端末向け。",
    status: "available",
    recommended: false,
    active: false,
  },
];

/// The card element for one model, so assertions cannot leak into a sibling
/// card in the same family group.
function cardFor(name: string): HTMLElement {
  const card = Array.from(
    document.querySelectorAll<HTMLElement>(`.${modelCard}`),
  ).find((el) => el.textContent?.includes(name));
  if (!card) throw new Error(`no card for ${name}`);
  return card;
}

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockImplementation((cmd: string) => {
    if (cmd === "list_models") return Promise.resolve(MODELS_FIXTURE);
    return Promise.resolve(undefined);
  });
});

afterEach(cleanup);

describe("ModelManager", () => {
  it("offers both Gemma 4 options under one family heading", async () => {
    render(<ModelManager />);

    await waitFor(() => {
      expect(screen.getByText("Gemma 4 E2B INT4")).toBeDefined();
    });
    expect(screen.getByText("Gemma 4 E4B INT4")).toBeDefined();
    // E2B and E4B are grouped together, not listed as two separate families.
    expect(screen.getAllByText("Gemma 4")).toHaveLength(1);
  });

  it("downloads the variant that was clicked", async () => {
    render(<ModelManager />);
    await waitFor(() => screen.getByText("Gemma 4 E4B INT4"));

    const card = cardFor("Gemma 4 E4B INT4");
    fireEvent.click(within(card).getByText("ダウンロード"));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("download_model", {
        variant: "4-e4b-int4",
      });
    });
  });

  it("switches the active model to the selected variant", async () => {
    render(<ModelManager />);
    await waitFor(() => screen.getByText("Gemma 4 E2B INT4"));

    const card = cardFor("Gemma 4 E2B INT4");
    fireEvent.click(within(card).getByText("使用する"));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("set_active_model", {
        variant: "4-e2b-int4",
      });
    });
    // The list is refreshed so the "使用中" marker follows the switch.
    expect(
      invokeMock.mock.calls.filter((c) => c[0] === "list_models").length,
    ).toBeGreaterThan(1);
  });

  it("marks the active model and does not offer to re-select it", async () => {
    render(<ModelManager />);
    await waitFor(() => screen.getByText("Gemma 3 1B INT4"));

    const activeCard = cardFor("Gemma 3 1B INT4");
    expect(within(activeCard).getByText("使用中")).toBeDefined();
    expect(within(activeCard).queryByText("使用する")).toBeNull();
  });

  it("does not offer an uninstalled model as selectable", async () => {
    render(<ModelManager />);
    await waitFor(() => screen.getByText("Gemma 4 E4B INT4"));

    const card = cardFor("Gemma 4 E4B INT4");
    expect(within(card).queryByText("使用する")).toBeNull();
    expect(within(card).getByText("ダウンロード")).toBeDefined();
  });

  it("shows download size for an uninstalled model", async () => {
    render(<ModelManager />);
    await waitFor(() => screen.getByText("Gemma 4 E4B INT4"));

    const card = cardFor("Gemma 4 E4B INT4");
    expect(within(card).getByText("5.2 GB")).toBeDefined();
    expect(within(card).getByText("未導入")).toBeDefined();
  });

  it("surfaces a failed model switch instead of silently keeping the old one", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_models") return Promise.resolve(MODELS_FIXTURE);
      if (cmd === "set_active_model") {
        return Promise.reject(new Error("unknown model"));
      }
      return Promise.resolve(undefined);
    });

    render(<ModelManager />);
    await waitFor(() => screen.getByText("Gemma 4 E2B INT4"));

    const card = cardFor("Gemma 4 E2B INT4");
    fireEvent.click(within(card).getByText("使用する"));

    await waitFor(() => {
      expect(screen.getByText(/モデルの切り替えに失敗しました/)).toBeDefined();
    });
  });
});
