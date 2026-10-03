import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { StudentJoin } from "../StudentJoin";
import { StudentQuiz } from "../StudentQuiz";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}));
afterEach(cleanup);
beforeEach(() => {
  invoke.mockReset();
  invoke.mockImplementation((command: string) =>
    command === "browse_teachers" ? Promise.resolve([]) : new Promise(() => {}),
  );
});

describe("mobile input flows", () => {
  it("does not submit an answer while confirming Japanese composition", () => {
    render(<StudentQuiz sessionId="test" onComplete={() => {}} />);
    const input = screen.getByPlaceholderText("答えを入力");
    fireEvent.change(input, { target: { value: "回答" } });
    fireEvent.keyDown(input, { key: "Enter", isComposing: true });
    fireEvent.keyDown(input, { key: "Enter", keyCode: 229 });
    expect(invoke).not.toHaveBeenCalled();
    fireEvent.keyDown(input, { key: "Enter" });
    expect(invoke).toHaveBeenCalledWith("analyze_answer", expect.any(Object));
  });

  it("keeps the answer on a send failure and retries the same event", async () => {
    let attempts = 0;
    invoke.mockImplementation((command: string) => {
      if (command === "analyze_answer")
        return Promise.resolve({ isCorrect: true, misconception: null });
      if (command === "send_analysis_event")
        return ++attempts === 1 ? Promise.reject("offline") : Promise.resolve();
      return Promise.resolve();
    });
    render(<StudentQuiz sessionId="test" onComplete={() => {}} />);
    fireEvent.change(screen.getByPlaceholderText("答えを入力"), {
      target: { value: "3" },
    });
    fireEvent.click(screen.getByRole("button", { name: "答えを確かめる" }));
    fireEvent.click(await screen.findByRole("button", { name: "次の問題へ" }));
    expect(await screen.findByText(/結果を送信できませんでした/)).toBeDefined();
    expect(screen.getByText("y = 3x + 4 の傾きは？")).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "次の問題へ" }));
    await screen.findByText("y = 2x + 5 の切片は？");
    const sent = invoke.mock.calls.filter(
      ([name]) => name === "send_analysis_event",
    );
    expect(JSON.parse(sent[0][1].eventJson).id).toBe(
      JSON.parse(sent[1][1].eventJson).id,
    );
  });

  it("offers retry after discovery finishes and rejects invalid ports", async () => {
    render(<StudentJoin onJoined={() => {}} onReset={() => {}} />);
    const retry = await screen.findByRole("button", { name: "教室を再検索" });
    fireEvent.click(retry);
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));
    fireEvent.change(screen.getByPlaceholderText("IPアドレス"), {
      target: { value: "127.0.0.1" },
    });
    fireEvent.change(screen.getByPlaceholderText("4桁の参加コード"), {
      target: { value: "1234" },
    });
    for (const value of ["0", "65536", "1.5", "-1"]) {
      fireEvent.change(screen.getByPlaceholderText("ポート"), {
        target: { value },
      });
      expect(
        (screen.getByRole("button", { name: "参加" }) as HTMLButtonElement)
          .disabled,
      ).toBe(true);
    }
    fireEvent.change(screen.getByPlaceholderText("ポート"), {
      target: { value: "8765" },
    });
    fireEvent.click(screen.getByRole("button", { name: "参加" }));
    expect(
      (screen.getByRole("button", { name: "参加中…" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    expect(
      invoke.mock.calls.filter(
        ([command]) => command === "student_prepare_connection",
      ),
    ).toHaveLength(1);
  });
});
