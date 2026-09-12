import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import React, { useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { LessonPlanEditor } from "../LessonPlanEditor";
import { planGenerating, planOverlay } from "../LessonPlanEditor.css.ts";
import { type LessonPlan, useLessonPlan } from "../useLessonPlan";

const { FakeChannel } = vi.hoisted(() => ({
  FakeChannel: class {
    onmessage: (message: unknown) => void = () => {};
  },
}));

type PlanProgress =
  | { stage: "prompt"; done: number; total: number; attempt: number }
  | { stage: "generating"; generated: number; max: number; attempt: number }
  | { stage: "validating"; attempt: number }
  | { stage: "repairing"; reason: string };

type ProgressChannel = { onmessage: (message: PlanProgress) => void };

const invoke = vi.fn();
const listeners = new Map<string, (event: { payload: unknown }) => void>();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invoke(...args),
  Channel: FakeChannel,
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: (event: string, handler: (e: { payload: unknown }) => void) => {
    listeners.set(event, handler);
    return Promise.resolve(() => listeners.delete(event));
  },
}));

const summary = {
  participantCount: 10,
  responseCount: 25,
  correctRate: 0.4,
  retrySuccessRate: 0.5,
  averageHints: 1.5,
  misconceptions: [],
};

const PLAN: LessonPlan = {
  focus: "一次関数の理解",
  steps: ["復習", "新出", "練習", "まとめ"],
  checkQuestion: "傾きの意味を説明せよ",
  teacherNote: "注意点あり",
};

/** The lesson tab: the plan is owned above the editor, which a tab hides. */
function LessonTab({ visible = true }: { visible?: boolean }) {
  const draft = useLessonPlan(summary, null, true);
  return visible ? (
    <LessonPlanEditor draft={draft} onAdopted={() => {}} />
  ) : null;
}

/** The same, with a control that leaves the tab and comes back. */
function SwitchableTab() {
  const [open, setOpen] = useState(true);
  const draft = useLessonPlan(summary, null, true);
  return (
    <>
      <button type="button" onClick={() => setOpen((o) => !o)}>
        タブ切り替え
      </button>
      {open && <LessonPlanEditor draft={draft} onAdopted={() => {}} />}
    </>
  );
}

afterEach(cleanup);
beforeEach(() => {
  invoke.mockReset();
  listeners.clear();
  Object.defineProperty(window, "__TAURI_INTERNALS__", {
    value: {},
    configurable: true,
  });
});

/** Hold every generation open, handing back its channel and resolver. */
function pendingGenerations() {
  const channels: ProgressChannel[] = [];
  const resolvers: ((plan: LessonPlan) => void)[] = [];
  invoke.mockImplementation(
    (command: string, args: Record<string, unknown>) => {
      if (!String(command).startsWith("generate_lesson_plan")) {
        return Promise.resolve(null);
      }
      if (args?.onProgress) channels.push(args.onProgress as ProgressChannel);
      return new Promise<LessonPlan>((done) => resolvers.push(done));
    },
  );
  return {
    count: () => resolvers.length,
    channel: () => channels[channels.length - 1],
    resolveAll: (plan: LessonPlan) => {
      for (const done of resolvers) done(plan);
    },
  };
}

const bar = () =>
  screen.getByRole("progressbar", { name: "レッスンプランの生成状況" });

describe("lesson plan generation progress", () => {
  it("reports prompt processing, writing, validation and the repair pass", async () => {
    const run = pendingGenerations();
    render(<LessonTab />);
    await waitFor(() => expect(run.channel()).toBeDefined());

    await act(async () => {
      run.channel().onmessage({
        stage: "prompt",
        done: 200,
        total: 400,
        attempt: 1,
      });
    });
    expect(screen.getByText(/集計と教材を読み込み中/)).toBeDefined();
    expect(screen.getByText(/200\/400 トークン/)).toBeDefined();
    // Half of the prompt, which is the first tenth of one of two passes.
    expect(bar().getAttribute("aria-valuenow")).toBe("5");

    await act(async () => {
      run.channel().onmessage({
        stage: "generating",
        generated: 256,
        max: 512,
        attempt: 1,
      });
    });
    expect(
      screen.getByText(/授業案を作成中… \(256\/512 トークン\)/),
    ).toBeDefined();
    expect(Number(bar().getAttribute("aria-valuenow"))).toBeGreaterThan(25);

    await act(async () => {
      run.channel().onmessage({ stage: "validating", attempt: 1 });
    });
    expect(screen.getByText(/出力の形式を確認中/)).toBeDefined();
    expect(bar().getAttribute("aria-valuenow")).toBe("50");

    await act(async () => {
      run.channel().onmessage({
        stage: "repairing",
        reason: "steps must have exactly 4 items",
      });
    });
    expect(screen.getByText(/形式が不正だったため作り直します/)).toBeDefined();

    await act(async () => {
      run.channel().onmessage({
        stage: "generating",
        generated: 100,
        max: 512,
        attempt: 2,
      });
    });
    expect(screen.getByText(/作り直し: 授業案を作成中/)).toBeDefined();
    expect(Number(bar().getAttribute("aria-valuenow"))).toBeGreaterThan(50);
  });

  it("names the model load as the reason, and clears the panel when the plan arrives", async () => {
    const run = pendingGenerations();
    render(<LessonTab />);
    await waitFor(() => expect(run.channel()).toBeDefined());

    await act(async () => {
      listeners.get("model-load-progress")?.({
        payload: {
          variant: "4-e2b-int4",
          modelName: "Gemma 4 E2B INT4",
          stage: "decoder",
          label: "モデル本体をメモリに読み込み中…",
          percent: 40,
          done: false,
        },
      });
    });
    // Why the plan has not started; how far the load has got is in the app
    // bar, which owns model state across screens.
    expect(screen.getByText("モデルの読み込みを待っています…")).toBeDefined();
    expect(bar().getAttribute("aria-valuenow")).toBeNull();

    await act(async () => run.resolveAll(PLAN));
    expect(
      screen.queryByRole("progressbar", { name: "レッスンプランの生成状況" }),
    ).toBeNull();
    expect(screen.getByDisplayValue("一次関数の理解")).toBeDefined();
  });

  it("keeps an elapsed time visible for the whole wait", async () => {
    const run = pendingGenerations();
    render(<LessonTab />);
    await waitFor(() => expect(run.count()).toBe(1));
    expect(screen.getByText(/経過 \d+\.\d 秒/)).toBeDefined();
  });

  it("generates once on mount, even though effects run twice in development", async () => {
    const run = pendingGenerations();
    render(
      <React.StrictMode>
        <LessonTab />
      </React.StrictMode>,
    );
    await waitFor(() => expect(run.count()).toBeGreaterThan(0));
    // A second pass would queue another full generation behind the first on
    // the model, and its result used to be discarded as stale.
    expect(run.count()).toBe(1);

    await act(async () => run.resolveAll(PLAN));
    expect(screen.getByDisplayValue("一次関数の理解")).toBeDefined();
  });

  it("survives leaving the tab and coming back mid-generation", async () => {
    const run = pendingGenerations();
    render(<SwitchableTab />);
    await waitFor(() => expect(run.count()).toBe(1));

    await act(async () => {
      run.channel().onmessage({
        stage: "generating",
        generated: 120,
        max: 512,
        attempt: 1,
      });
    });

    // Leave the tab while the model is still writing, then come back.
    await act(async () => {
      screen.getByText("タブ切り替え").click();
    });
    expect(screen.queryByText("学習の焦点")).toBeNull();
    await act(async () => {
      screen.getByText("タブ切り替え").click();
    });

    // The generation was never abandoned or duplicated, and the returning
    // screen picks the wait back up where it actually is.
    expect(run.count()).toBe(1);
    expect(
      screen.getByText(/授業案を作成中… \(120\/512 トークン\)/),
    ).toBeDefined();
    expect(screen.getByText(/経過 \d+\.\d 秒/)).toBeDefined();

    await act(async () => run.resolveAll(PLAN));
    expect(screen.getByDisplayValue("一次関数の理解")).toBeDefined();
  });

  it("does not start a second generation when regenerate is pressed mid-run", async () => {
    const run = pendingGenerations();
    render(<LessonTab />);
    await waitFor(() => expect(run.count()).toBe(1));

    await act(async () => {
      screen.getByText("生成中…").click();
    });
    expect(run.count()).toBe(1);
  });

  it("says why nothing arrived when the first generation fails", async () => {
    invoke.mockRejectedValue("model missing");
    render(<LessonTab />);
    expect(
      await screen.findByText(/授業案を生成できませんでした: model missing/),
    ).toBeDefined();
  });

  it("animates the card while generating, since there is no output to show yet", async () => {
    const run = pendingGenerations();
    render(<LessonTab />);
    await waitFor(() => expect(run.count()).toBe(1));

    const card = document.querySelector(".lesson-plan-card") as HTMLElement;
    expect(card.className).toContain(planGenerating);
    expect(card.getAttribute("aria-busy")).toBe("true");

    await act(async () => run.resolveAll(PLAN));
    expect(card.className).not.toContain(planGenerating);
    expect(card.getAttribute("aria-busy")).toBe("false");
  });

  it("draws the wait over the card, so nothing on the page moves", async () => {
    const run = pendingGenerations();
    const { container } = render(<LessonTab />);
    await waitFor(() => expect(run.count()).toBe(1));

    const card = container.querySelector(".lesson-plan-card") as HTMLElement;
    const ringEl = bar();
    // Over the card the plan will fill, not between the header and the card
    // where it used to push everything down.
    expect(card.contains(ringEl)).toBe(true);
    expect(ringEl.closest(`.${planOverlay}`)).not.toBeNull();

    // Nothing is inserted between the header and the card.
    const header = container.querySelector(
      ".lesson-plan-header",
    ) as HTMLElement;
    expect(header.nextElementSibling).toBe(card);

    const childrenWhileBusy = card.childElementCount;
    await act(async () => run.resolveAll(PLAN));
    // The overlay is the only thing that leaves; the fields stay put.
    expect(card.childElementCount).toBe(childrenWhileBusy - 1);
    expect(
      container.querySelector(".lesson-plan-header")?.nextElementSibling,
    ).toBe(card);
  });
});
