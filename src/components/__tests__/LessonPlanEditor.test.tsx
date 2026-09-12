import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { LessonPlanEditor } from "../LessonPlanEditor";
import { EMPTY_PLAN, type LessonPlanDraft } from "../useLessonPlan";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) =>
    cmd === "save_lesson_plan" ? null : {},
  ),
  Channel: class {
    onmessage: (m: unknown) => void = () => {};
  },
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

/** A settled draft: nothing generating, a plan already in hand. */
function draft(overrides: Partial<LessonPlanDraft> = {}): LessonPlanDraft {
  return {
    plan: {
      ...EMPTY_PLAN,
      focus: "一次関数の理解",
      steps: ["復習", "新出", "練習", "まとめ"],
    },
    setPlan: vi.fn(),
    busy: false,
    progress: null,
    modelLoading: false,
    elapsedMs: 0,
    error: null,
    setError: vi.fn(),
    regenerate: vi.fn(),
    ...overrides,
  };
}

describe("LessonPlanEditor", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders all editable fields", () => {
    render(<LessonPlanEditor draft={draft()} onAdopted={() => {}} />);
    expect(screen.getByText("学習の焦点")).toBeDefined();
    expect(screen.getByText("授業の展開（4段階）")).toBeDefined();
    expect(screen.getByText("確認質問")).toBeDefined();
    expect(screen.getByText("教師メモ")).toBeDefined();
    expect(screen.getByDisplayValue("一次関数の理解")).toBeDefined();
  });

  it("renders AI regenerate and adopt buttons", () => {
    render(<LessonPlanEditor draft={draft()} onAdopted={() => {}} />);
    expect(screen.getByText("AIで再生成")).toBeDefined();
    expect(screen.getByText("この案を採用")).toBeDefined();
  });

  it("asks the owner to regenerate rather than generating itself", () => {
    const regenerate = vi.fn();
    render(
      <LessonPlanEditor draft={draft({ regenerate })} onAdopted={() => {}} />,
    );
    fireEvent.click(screen.getByText("AIで再生成"));
    expect(regenerate).toHaveBeenCalledOnce();
  });

  it("reports edits through the owner's setPlan", () => {
    const setPlan = vi.fn();
    render(
      <LessonPlanEditor draft={draft({ setPlan })} onAdopted={() => {}} />,
    );
    fireEvent.change(screen.getByLabelText("学習の焦点"), {
      target: { value: "傾きの意味" },
    });
    expect(setPlan).toHaveBeenCalled();
  });

  it("calls onAdopted when adopt button is clicked", async () => {
    const onAdopted = vi.fn();
    render(<LessonPlanEditor draft={draft()} onAdopted={onAdopted} />);
    fireEvent.click(screen.getByText("この案を採用"));
    await vi.waitFor(() => expect(onAdopted).toHaveBeenCalled());
  });

  it("disables both actions while a plan is being generated", () => {
    render(
      <LessonPlanEditor draft={draft({ busy: true })} onAdopted={() => {}} />,
    );
    expect(screen.getByText("生成中…").closest("button")?.disabled).toBe(true);
    expect(
      (screen.getByText("この案を採用") as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it("shows generated text in full instead of clipping it", () => {
    // jsdom has no layout, so the content height is supplied here; the point
    // under test is that the field is resized to it.
    const contentHeight = 132;
    Object.defineProperty(HTMLTextAreaElement.prototype, "scrollHeight", {
      configurable: true,
      get: () => contentHeight,
    });

    const long =
      "最も多い誤概念である傾きと変化量の分子分母の混同に焦点を当てています。" +
      "計算過程を重視し、なぜその計算順序なのかを口頭で確認させることが重要です。";
    render(
      <LessonPlanEditor
        draft={draft({
          plan: {
            focus: "傾きと変化量の分子・分母の役割の明確化",
            steps: [
              "0〜2分：デモ",
              "2〜5分：計算",
              "5〜8分：共有",
              "8〜10分：確認",
            ],
            checkQuestion:
              "点A(1, 3)と点B(4, 9)を結ぶ直線の傾きを求めなさい。計算過程も示すこと。",
            teacherNote: long,
          },
        })}
        onAdopted={() => {}}
      />,
    );

    const note = screen.getByLabelText("教師メモ") as HTMLTextAreaElement;
    expect(note.style.height).toBe(`${contentHeight}px`);

    // The check question used to be a single-line input, so a generated
    // question ran off the right edge with no way to see the rest.
    const check = screen.getByLabelText("確認質問");
    expect(check.tagName).toBe("TEXTAREA");
    expect((check as HTMLTextAreaElement).style.height).toBe(
      `${contentHeight}px`,
    );
    expect(screen.getByLabelText("学習の焦点").tagName).toBe("TEXTAREA");

    Reflect.deleteProperty(HTMLTextAreaElement.prototype, "scrollHeight");
  });
});
