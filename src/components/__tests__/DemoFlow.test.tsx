// @vitest-environment jsdom

import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DemoFlow } from "../DemoFlow";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

// 実機(dev/release双方)で計測したRust返却JSONそのままのフィクスチャ
const SUMMARY_FIXTURE = {
  participantCount: 8,
  responseCount: 8,
  correctRate: 0.375,
  retrySuccessRate: 0.8,
  averageHints: 1.25,
  misconceptions: [
    { name: "傾きと切片の混同", count: 3, share: 0.375 },
    { name: "切片の計算漏れ", count: 1, share: 0.125 },
    { name: "変化量の分子と分母の逆転", count: 1, share: 0.125 },
  ],
};
const PLAN_FIXTURE = {
  focus: "「傾きと切片の混同」を解きほぐす",
  steps: ["step1", "step2"],
  checkQuestion: "今日の要点を一文で説明してください。",
  teacherNote: "正解を先に示さないこと。",
};

beforeEach(() => {
  invokeMock.mockReset();
});
afterEach(() => {
  cleanup();
  Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
});

describe("DemoFlow", () => {
  it("非Tauri環境では開始ボタンを提示せずデスクトップアプリ用の案内を出す", () => {
    render(<DemoFlow />);
    // 開始ボタン自体が存在しない(IPCを持たない環境での誤実行を排除)
    expect(screen.queryByRole("button", { name: "デモを始める" })).toBeNull();
    expect(screen.getByText(/bun run tauri dev/)).toBeDefined();
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("Tauri環境ではステップが完了まで進行する", async () => {
    Reflect.set(window, "__TAURI_INTERNALS__", {});
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "start_demo_session") return Promise.resolve("uuid");
      if (cmd === "submit_demo_events") return Promise.resolve(null);
      if (cmd === "get_demo_quiz_summary")
        return Promise.resolve(SUMMARY_FIXTURE);
      if (cmd === "get_demo_lesson_plan") return Promise.resolve(PLAN_FIXTURE);
      return Promise.reject(new Error(`unexpected cmd ${cmd}`));
    });
    vi.useFakeTimers();
    try {
      render(<DemoFlow />);
      fireEvent.click(screen.getByRole("button", { name: "デモを始める" }));
      // sleep合計10.5秒分を仮想時計で一気に進め、UI状態遷移をawaitする
      for (let i = 0; i < 6; i++) {
        await vi.advanceTimersByTimeAsync(2500);
        if (screen.queryByText(/3分デモが完了しました。/)) break;
      }
      expect(screen.getByText(/3分デモが完了しました。/)).toBeDefined();
      // 実データが画面に描画されていること
      expect(screen.getByText("8名")).toBeDefined();
      expect(
        screen.getByText("「傾きと切片の混同」を解きほぐす"),
      ).toBeDefined();
    } finally {
      vi.useRealTimers();
    }
  }, 20000);

  it("Tauri環境でコマンド失敗時はエラー内容と再試行手段を出す", async () => {
    Reflect.set(window, "__TAURI_INTERNALS__", {});
    invokeMock.mockRejectedValueOnce(new Error("boom: start_demo_session"));
    render(<DemoFlow />);
    fireEvent.click(screen.getByRole("button", { name: "デモを始める" }));
    await waitFor(() => {
      expect(screen.getByText(/boom: start_demo_session/)).toBeDefined();
    });
    expect(screen.getByRole("button", { name: "もう一度試す" })).toBeDefined();
  });
});
