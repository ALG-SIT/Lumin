// @vitest-environment jsdom
// Tauri v2 引数契約ロック: JSキーはcamelCase必須(tauri-macros既定 ArgumentCase::Camel)。

import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Quiz } from "../StudentQuiz";
import { StudentQuiz } from "../StudentQuiz";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

const QUIZ: Quiz = {
  id: "lf",
  title: "一次関数チェック",
  subject: "数学",
  questions: [
    {
      id: "lf-01",
      prompt: "y = 3x + 4 の傾きは？",
      concept: "一次関数の傾き",
      acceptedAnswers: ["3", "+3"],
      hints: ["y = mx + b の係数に注目", "b は y 切片"],
      genericMisconception: "傾きの符号の読み落とし",
      explanation: "m = 3。",
    },
  ],
};

beforeEach(() => {
  invokeMock.mockReset();
});
afterEach(() => cleanup());

describe("StudentQuiz IPC contracts (camelCase keys)", () => {
  it("analyze_answer には questionJson/studentAnswer/hintLevel を送る", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "analyze_answer")
        return Promise.resolve({ isCorrect: true, misconception: null });
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    const onComplete = vi.fn();
    render(
      <StudentQuiz sessionId="sess" quiz={QUIZ} onComplete={onComplete} />,
    );

    fireEvent.change(screen.getByPlaceholderText("答えを入力"), {
      target: { value: "3" },
    });
    fireEvent.click(screen.getByRole("button", { name: "答えを確かめる" }));

    await waitFor(() => {
      const call = invokeMock.mock.calls.find(([c]) => c === "analyze_answer");
      expect(call).toBeDefined();
      if (!call) throw new Error("analyze_answer was not invoked");
      const keys = Object.keys(call[1] as Record<string, unknown>).sort();
      expect(keys).toEqual(["hintLevel", "questionJson", "studentAnswer"]);
    });
  });

  it("generate_hint はcamelCaseキーで、失敗時は問題バンクのヒントへフォールバックする", async () => {
    // 必ず不正解→ヒント生成が走る
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "analyze_answer")
        return Promise.resolve({
          isCorrect: false,
          misconception: "傾きの符号の読み落とし",
        });
      // generate_hint はAI未ロード想定でreject
      return Promise.reject(
        new Error("AIモデルが未ロードのためヒント生成できません"),
      );
    });
    render(<StudentQuiz sessionId="sess" quiz={QUIZ} onComplete={() => {}} />);

    fireEvent.change(screen.getByPlaceholderText("答えを入力"), {
      target: { value: "999" },
    });
    fireEvent.click(screen.getByRole("button", { name: "答えを確かめる" }));

    await waitFor(() => {
      const call = invokeMock.mock.calls.find(([c]) => c === "generate_hint");
      expect(call).toBeDefined();
      if (!call) throw new Error("generate_hint was not invoked");
      const keys = Object.keys(call[1] as Record<string, unknown>).sort();
      expect(keys).toEqual(["concept", "hintLevel", "questionId"]);
    });
    // 失敗してもバンクのヒントが表示される(モック文は出ない)
    await waitFor(() => {
      expect(screen.getByText(/y = mx \+ b の係数に注目/)).toBeDefined();
    });
    expect(screen.queryByText(/モック出力/)).toBeNull();
  });
});
