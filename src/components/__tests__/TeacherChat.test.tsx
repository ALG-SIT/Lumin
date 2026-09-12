import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TeacherChat } from "../TeacherChat";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}));
afterEach(cleanup);
beforeEach(() => invoke.mockReset());

const summary = {
  participantCount: 10,
  responseCount: 12,
  correctRate: 0.5,
  retrySuccessRate: 0.25,
  averageHints: 2,
  misconceptions: [{ name: "通分", count: 6, share: 0.5 }],
};

describe("TeacherChat", () => {
  it("does not scroll on mount, follows new messages only inside history and preserves reading position", async () => {
    let resolve!: (text: string) => void;
    invoke.mockReturnValue(
      new Promise<string>((done) => {
        resolve = done;
      }),
    );
    render(<TeacherChat classSummary={summary} activeQuiz={null} />);
    const history = screen.getByRole("log");
    expect(history.scrollTop).toBe(0);
    Object.defineProperties(history, {
      scrollHeight: { value: 1000, configurable: true },
      clientHeight: { value: 200 },
    });
    fireEvent.click(screen.getByText("次の手は？"));
    expect(history.scrollTop).toBe(1000);
    history.scrollTop = 100;
    fireEvent.scroll(history);
    await act(async () => resolve("通分を確認しましょう"));
    expect(history.scrollTop).toBe(100);
    expect(screen.getByText("通分を確認しましょう")).toBeDefined();
  });

  it("does not send Enter during IME composition and sends full context afterward", async () => {
    invoke.mockResolvedValue("確認しましょう");
    render(<TeacherChat classSummary={summary} activeQuiz={null} />);
    const input = screen.getByPlaceholderText("学習状況について質問");
    fireEvent.change(input, { target: { value: "指導方法" } });
    fireEvent.keyDown(input, { key: "Enter", isComposing: true });
    expect(invoke).not.toHaveBeenCalled();
    fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(invoke).toHaveBeenCalledOnce());
    expect(
      JSON.parse(invoke.mock.calls[0][1].contextJson).classSummary,
    ).toEqual(summary);
  });
});
