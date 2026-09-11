import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { LessonPlanEditor } from "../LessonPlanEditor";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "generate_lesson_plan") {
      return {
        focus: "一次関数の理解",
        steps: ["復習", "新出", "練習", "まとめ"],
        checkQuestion: "傾きの意味を説明せよ",
        teacherNote: "注意点あり",
      };
    }
    if (cmd === "save_lesson_plan") return null;
    return {};
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

const mockSummary = {
  participantCount: 10,
  responseCount: 25,
  correctRate: 0.4,
  retrySuccessRate: 0.5,
  averageHints: 1.5,
  misconceptions: [],
};

describe("LessonPlanEditor", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders all editable fields", () => {
    render(
      <LessonPlanEditor classSummary={mockSummary} onAdopted={() => {}} />,
    );
    expect(screen.getByText("学習の焦点")).toBeDefined();
    expect(screen.getByText("授業の展開（4段階）")).toBeDefined();
    expect(screen.getByText("確認質問")).toBeDefined();
    expect(screen.getByText("教師メモ")).toBeDefined();
  });

  it("renders AI regenerate and adopt buttons", () => {
    render(
      <LessonPlanEditor classSummary={mockSummary} onAdopted={() => {}} />,
    );
    expect(screen.getByText("AIで再生成")).toBeDefined();
    expect(screen.getByText("この案を採用")).toBeDefined();
  });

  it("calls onAdopted when adopt button is clicked", async () => {
    const onAdopted = vi.fn();
    render(
      <LessonPlanEditor classSummary={mockSummary} onAdopted={onAdopted} />,
    );
    fireEvent.click(screen.getByText("この案を採用"));
    await vi.waitFor(() => {
      expect(onAdopted).toHaveBeenCalled();
    });
  });
});
