import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TeacherChat } from "../TeacherChat";
import { teacherChatPanel } from "../TeacherChat.css.ts";
import { GEMMA4_SUMMARY_REPLY } from "./realReplies";

const invoke = vi.fn();
const listeners = new Map<string, (event: { payload: unknown }) => void>();

/** Stand-in for Tauri's Channel, which needs a real IPC bridge to construct. */
const { FakeChannel } = vi.hoisted(() => ({
  FakeChannel: class {
    onmessage: (message: unknown) => void = () => {};
  },
}));

type ChatChunk =
  | { kind: "prefill"; done: number; total: number }
  | { kind: "text"; text: string; generated: number };

type ChunkChannel = { onmessage: (message: ChatChunk) => void };

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

afterEach(cleanup);
beforeEach(() => {
  invoke.mockReset();
  listeners.clear();
  // The streaming path needs a Tauri host; tests opt in per case.
  Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
});

function asTauriHost() {
  Object.defineProperty(window, "__TAURI_INTERNALS__", {
    value: {},
    configurable: true,
  });
}

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

  it("sends earlier turns with their speakers and only the new question as the message", async () => {
    invoke.mockResolvedValueOnce("通分の練習をしましょう");
    render(<TeacherChat classSummary={summary} activeQuiz={null} />);

    fireEvent.click(screen.getByText("何が分かっていない？"));
    await waitFor(() => expect(invoke).toHaveBeenCalledOnce());
    // Nothing precedes the first question.
    expect(JSON.parse(invoke.mock.calls[0][1].contextJson).history).toEqual([]);
    await screen.findByText("通分の練習をしましょう");

    invoke.mockResolvedValueOnce("練習問題を3問出しましょう");
    fireEvent.click(screen.getByText("次の手は？"));
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));

    const second = invoke.mock.calls[1][1];
    expect(second.message).toBe("次の手は？");
    expect(JSON.parse(second.contextJson).history).toEqual([
      { role: "user", content: "何が分かっていない？" },
      { role: "assistant", content: "通分の練習をしましょう" },
    ]);
    // The new question must not also be buried in the history.
    expect(JSON.parse(second.contextJson).history).toHaveLength(2);
  });

  it("renders the reply as markdown rather than as literal syntax", async () => {
    invoke.mockResolvedValue(
      "## 次の一手\n\n- **通分**を確認\n- `分母` をそろえる",
    );
    render(<TeacherChat classSummary={summary} activeQuiz={null} />);
    fireEvent.click(screen.getByText("次の手は？"));

    // The bubble sits under the panel's own h2, so a reply's headings start
    // below it rather than competing with the page hierarchy.
    const heading = await screen.findByRole("heading", { name: "次の一手" });
    expect(heading.tagName).toBe("H4");
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByText("通分").tagName).toBe("STRONG");
    expect(screen.getByText("分母").tagName).toBe("CODE");
    expect(screen.queryByText(/## 次の一手/)).toBeNull();
  });

  it("shows streamed chunks while generating and settles on the final reply", async () => {
    asTauriHost();
    let channel!: ChunkChannel;
    let resolve!: (text: string) => void;
    invoke.mockImplementation(
      (command: string, args: Record<string, unknown>) => {
        if (command === "preload_active_model") return Promise.resolve(true);
        channel = args.onChunk as ChunkChannel;
        return new Promise<string>((done) => {
          resolve = done;
        });
      },
    );

    render(<TeacherChat classSummary={summary} activeQuiz={null} />);
    fireEvent.click(screen.getByText("次の手は？"));
    await waitFor(() => expect(channel).toBeDefined());

    // Before any text exists the prompt is being read into the cache; that
    // wait is reported as a real fraction rather than a bare spinner.
    await act(async () => {
      channel.onmessage({ kind: "prefill", done: 0, total: 400 });
      channel.onmessage({ kind: "prefill", done: 128, total: 400 });
    });
    const prefill = screen.getByRole("progressbar", {
      name: "回答の生成状況",
    });
    expect(prefill.getAttribute("aria-valuenow")).toBe("32");
    expect(screen.getByText(/128\/400 トークン/)).toBeDefined();
    expect(screen.getByText(/経過 /)).toBeDefined();

    // Replay a real reply the way the model emits it: a few characters at a
    // time, so half-written Markdown is rendered as it arrives.
    const chunks = GEMMA4_SUMMARY_REPLY.match(/[\s\S]{1,7}/g) ?? [];
    await act(async () => {
      chunks.slice(0, 4).forEach((text, i) => {
        channel.onmessage({ kind: "text", text, generated: i + 1 });
      });
    });
    // Partway through the first bullet: already a list, with no stray syntax.
    expect(screen.getAllByRole("listitem").length).toBeGreaterThan(0);
    expect(screen.getByRole("log").textContent).not.toContain("*");
    // The reply is on screen now, so the progress panel steps aside rather
    // than reporting on something the teacher can already read.
    expect(
      screen.queryByRole("progressbar", { name: "回答の生成状況" }),
    ).toBeNull();

    await act(async () => {
      chunks.slice(4).forEach((text, i) => {
        channel.onmessage({ kind: "text", text, generated: i + 5 });
      });
    });
    await act(async () => resolve(GEMMA4_SUMMARY_REPLY));

    expect(screen.getAllByRole("listitem")).toHaveLength(3);
    expect(screen.getByText("誤概念名").tagName).toBe("STRONG");
    expect(screen.getByText(/added numerators/)).toBeDefined();
    expect(
      invoke.mock.calls.some(([c]) => c === "chat_with_teacher_stream"),
    ).toBe(true);
  });

  it("says the question is waiting on the model, leaving the number to the app bar", async () => {
    asTauriHost();
    let channel!: ChunkChannel;
    invoke.mockImplementation(
      (command: string, args: Record<string, unknown>) => {
        if (command === "preload_active_model") return new Promise(() => {});
        channel = args.onChunk as ChunkChannel;
        return new Promise(() => {});
      },
    );

    render(<TeacherChat classSummary={summary} activeQuiz={null} />);
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("preload_active_model"),
    );
    fireEvent.click(screen.getByText("次の手は？"));
    await waitFor(() => expect(channel).toBeDefined());

    await act(async () => {
      listeners.get("model-load-progress")?.({
        payload: {
          variant: "1b-int4",
          modelName: "Gemma 3 1B INT4",
          stage: "decoder",
          label: "モデル本体をメモリに読み込み中…",
          percent: 40,
          done: false,
        },
      });
    });

    expect(screen.getByText("モデルの読み込みを待っています…")).toBeDefined();
    // The percentage lives in the app bar, which owns model state; showing it
    // here as well would be the same bar drawn twice.
    expect(
      screen
        .getByRole("progressbar", { name: "回答の生成状況" })
        .getAttribute("aria-valuenow"),
    ).toBeNull();
  });

  it("draws the wait inside the history, where the reply will appear", async () => {
    asTauriHost();
    let channel!: ChunkChannel;
    invoke.mockImplementation(
      (command: string, args: Record<string, unknown>) => {
        if (command === "preload_active_model") return Promise.resolve(true);
        channel = args.onChunk as ChunkChannel;
        return new Promise(() => {});
      },
    );
    render(<TeacherChat classSummary={summary} activeQuiz={null} />);
    const history = screen.getByRole("log");
    const before = history.parentElement?.childElementCount;

    fireEvent.click(screen.getByText("次の手は？"));
    await waitFor(() => expect(channel).toBeDefined());

    // Inside the scrolling history, in the assistant bubble the answer will
    // fill - not a panel above it that resizes the chat area.
    const ring = within(history).getByRole("progressbar", {
      name: "回答の生成状況",
    });
    expect(history.contains(ring)).toBe(true);
    expect(history.parentElement?.childElementCount).toBe(before);

    await act(async () => {
      channel.onmessage({ kind: "prefill", done: 100, total: 400 });
    });
    expect(ring.getAttribute("aria-valuenow")).toBe("25");
  });

  it("puts the transcript and the composer on one surface", () => {
    invoke.mockResolvedValue("確認しましょう");
    const { container } = render(
      <TeacherChat classSummary={summary} activeQuiz={null} />,
    );

    const history = screen.getByRole("log");
    const input = screen.getByPlaceholderText("学習状況について質問");
    const panel = container.querySelector(`.${teacherChatPanel}`);

    // The suggestions no longer sit between two separate cards: the whole
    // conversation, the chips and the input share one panel, which is what
    // gives the transcript the height back.
    expect(panel).not.toBeNull();
    expect(panel?.contains(history)).toBe(true);
    expect(panel?.contains(input)).toBe(true);
    expect(screen.getByText("次の手は？").closest(`.${teacherChatPanel}`)).toBe(
      panel,
    );

    // Below the header there is one surface, not three stacked ones.
    const section = container.querySelector("section") as HTMLElement;
    expect(section.childElementCount).toBe(2);
    expect(section.lastElementChild).toBe(panel);
  });
});
