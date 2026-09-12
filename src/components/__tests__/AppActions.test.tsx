import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { AppActions } from "../AppActions";

vi.mock("../AppBarModelStatus", () => ({
  AppBarModelStatus: () => <p>モデル情報</p>,
}));
beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
    this.dispatchEvent(new Event("close"));
  };
});
afterEach(cleanup);
it("keeps secondary actions inside the dismissible dialog", () => {
  const settings = vi.fn();
  const reset = vi.fn();
  render(<AppActions roleLabel="先生" onSettings={settings} onReset={reset} />);
  expect(screen.queryByText("設定・モデル管理")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "操作" }));
  expect(screen.getByRole("dialog", { name: "Luminの操作" })).toBeDefined();
  fireEvent.click(screen.getByRole("button", { name: "閉じる" }));
  expect(settings).not.toHaveBeenCalled();
  expect(reset).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "操作" }));
  fireEvent.click(screen.getByRole("button", { name: "設定・モデル管理" }));
  expect(settings).toHaveBeenCalledTimes(1);
  expect(screen.queryByRole("dialog")).toBeNull();
});

it("returns directly to role selection without opening settings", () => {
  const reset = vi.fn();
  render(<AppActions roleLabel="先生" onSettings={() => {}} onReset={reset} />);
  fireEvent.click(screen.getByRole("button", { name: "役割選択へ戻る" }));
  expect(reset).toHaveBeenCalledTimes(1);
  expect(screen.queryByRole("dialog")).toBeNull();
});
