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
  it("records retry success and the first misconception without sending answer text", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "analyze_answer") {
        return Promise.resolve(
          invokeMock.mock.calls.filter(([name]) => name === "analyze_answer")
            .length === 1
            ? { isCorrect: false, misconception: "傾きと切片の混同" }
            : { isCorrect: true, misconception: null },
        );
      }
      if (cmd === "generate_hint") return Promise.reject("offline");
      if (cmd === "send_analysis_event") return Promise.resolve();
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    const onComplete = vi.fn();
    render(
      <StudentQuiz
        sessionId="session-1"
        participantToken="participant-1"
        quiz={QUIZ}
        onComplete={onComplete}
      />,
    );

    fireEvent.change(screen.getByPlaceholderText("答えを入力"), {
      target: { value: "secret wrong answer" },
    });
    fireEvent.click(screen.getByRole("button", { name: "答えを確かめる" }));
    await screen.findByText(/答えはまだ見せません/);

    fireEvent.click(screen.getByRole("button", { name: "もう一度答える" }));
    fireEvent.change(screen.getByPlaceholderText("答えを入力"), {
      target: { value: "3" },
    });
    fireEvent.click(screen.getByRole("button", { name: "答えを確かめる" }));
    const finish = await screen.findByRole("button", {
      name: "結果を送って終了",
    });
    await waitFor(() => expect(finish.hasAttribute("disabled")).toBe(false));
    fireEvent.click(finish);
    await screen.findByText("おつかれさまでした");

    const call = invokeMock.mock.calls.find(
      ([name]) => name === "send_analysis_event",
    );
    expect(call).toBeDefined();
    const event = JSON.parse(call?.[1].eventJson as string);
    expect(event).toMatchObject({
      participantToken: "participant-1",
      sessionId: "session-1",
      questionID: "lf-01",
      misconception: "傾きと切片の混同",
      correct: false,
      hintCount: 1,
      retrySuccess: true,
    });
    expect(JSON.stringify(event)).not.toContain("secret wrong answer");
    expect(onComplete).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "参加画面へ戻る" }));
    expect(onComplete).toHaveBeenCalledOnce();
  });

  it("does not finish the last question until result delivery succeeds", async () => {
    let rejectDelivery = true;
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "analyze_answer")
        return Promise.resolve({ isCorrect: true, misconception: null });
      if (cmd === "send_analysis_event")
        return rejectDelivery ? Promise.reject("offline") : Promise.resolve();
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<StudentQuiz sessionId="sess" quiz={QUIZ} onComplete={() => {}} />);
    fireEvent.change(screen.getByPlaceholderText("答えを入力"), {
      target: { value: "3" },
    });
    fireEvent.click(screen.getByRole("button", { name: "答えを確かめる" }));
    const finish = await screen.findByRole("button", {
      name: "結果を送って終了",
    });
    await waitFor(() => expect(finish.hasAttribute("disabled")).toBe(false));
    fireEvent.click(finish);
    await screen.findByText(/結果を送信できませんでした/);
    expect(screen.queryByText("おつかれさまでした")).toBeNull();

    rejectDelivery = false;
    fireEvent.click(screen.getByRole("button", { name: "結果を送って終了" }));
    expect(await screen.findByText("おつかれさまでした")).toBeDefined();
    const sent = invokeMock.mock.calls.filter(
      ([name]) => name === "send_analysis_event",
    );
    expect(JSON.parse(sent[0][1].eventJson).id).toBe(
      JSON.parse(sent[1][1].eventJson).id,
    );
  });

  it("waits for one result delivery before advancing and resets per-question analytics", async () => {
    let resolveFirstDelivery!: () => void;
    const firstDelivery = new Promise<void>((resolve) => {
      resolveFirstDelivery = resolve;
    });
    const twoQuestionQuiz: Quiz = {
      ...QUIZ,
      questions: [
        QUIZ.questions[0],
        {
          ...QUIZ.questions[0],
          id: "lf-02",
          prompt: "切片は？",
          acceptedAnswers: ["5"],
        },
      ],
    };
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "analyze_answer")
        return Promise.resolve({ isCorrect: true, misconception: null });
      if (cmd === "send_analysis_event")
        return invokeMock.mock.calls.filter(([name]) => name === cmd).length ===
          1
          ? firstDelivery
          : Promise.resolve();
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(
      <StudentQuiz
        sessionId="sess"
        quiz={twoQuestionQuiz}
        onComplete={() => {}}
      />,
    );

    fireEvent.change(screen.getByPlaceholderText("答えを入力"), {
      target: { value: "3" },
    });
    fireEvent.click(screen.getByRole("button", { name: "答えを確かめる" }));
    const next = await screen.findByRole("button", { name: "次の問題へ" });
    fireEvent.click(next);
    fireEvent.click(next);

    await waitFor(() => {
      expect(
        invokeMock.mock.calls.filter(
          ([name]) => name === "send_analysis_event",
        ),
      ).toHaveLength(1);
    });
    expect(screen.getByText("1 / 2 問題目")).toBeDefined();
    resolveFirstDelivery();
    await screen.findByText("2 / 2 問題目");

    fireEvent.change(screen.getByPlaceholderText("答えを入力"), {
      target: { value: "5" },
    });
    fireEvent.click(screen.getByRole("button", { name: "答えを確かめる" }));
    fireEvent.click(
      await screen.findByRole("button", { name: "結果を送って終了" }),
    );
    await screen.findByText("おつかれさまでした");

    const events = invokeMock.mock.calls
      .filter(([name]) => name === "send_analysis_event")
      .map(([, args]) => JSON.parse(args.eventJson as string));
    expect(events).toHaveLength(2);
    expect(events[0]).toMatchObject({
      questionID: "lf-01",
      correct: true,
      hintCount: 0,
    });
    expect(events[1]).toMatchObject({
      questionID: "lf-02",
      correct: true,
      hintCount: 0,
    });
    expect(events[0].id).not.toBe(events[1].id);
    expect(events[1].misconception).toBeNull();
    expect(events[1].retrySuccess).toBe(false);
  });

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
      expect(keys).toEqual([
        "hintLevel",
        "previousHints",
        "questionJson",
        "studentAnswer",
      ]);
      expect(JSON.parse(call[1].questionJson)).toEqual(QUIZ.questions[0]);
      expect(call[1].studentAnswer).toBe("999");
    });
    // 失敗してもバンクのヒントが表示される(モック文は出ない)
    await waitFor(() => {
      expect(screen.getByText(/y = mx \+ b の係数に注目/)).toBeDefined();
    });
    expect(screen.queryByText(/モック出力/)).toBeNull();
  });
});

it("次のヒントには実際に表示したAIヒントと直前の解答を渡す", async () => {
  invokeMock.mockImplementation((cmd: string, args: { hintLevel?: number }) => {
    if (cmd === "analyze_answer")
      return Promise.resolve({
        isCorrect: false,
        misconception: "傾きの符号の読み落とし",
      });
    if (cmd === "generate_hint")
      return Promise.resolve(
        args.hintLevel === 1 ? "xが変わるときに注目しよう" : "xの係数を見よう",
      );
    return Promise.resolve(null);
  });
  render(<StudentQuiz sessionId="sess" quiz={QUIZ} onComplete={() => {}} />);
  fireEvent.change(screen.getByPlaceholderText("答えを入力"), {
    target: { value: "2" },
  });
  fireEvent.click(screen.getByRole("button", { name: "答えを確かめる" }));
  await screen.findByText("xが変わるときに注目しよう");
  fireEvent.click(screen.getByRole("button", { name: "次のヒント" }));
  await screen.findByText("xの係数を見よう");
  const calls = invokeMock.mock.calls.filter(
    ([cmd]) => cmd === "generate_hint",
  );
  expect(calls[1][1]).toMatchObject({
    hintLevel: 2,
    studentAnswer: "2",
    previousHints: ["xが変わるときに注目しよう"],
  });
});
