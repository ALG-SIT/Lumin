import "@testing-library/jest-dom/vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { StudentJoin } from "../StudentJoin";
import { TeacherChat } from "../TeacherChat";
import { TeacherDashboard } from "../TeacherDashboard";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
const summary = {
  participantCount: 0,
  responseCount: 0,
  correctRate: 0,
  retrySuccessRate: 0,
  averageHints: 0,
  misconceptions: [],
};
const quiz = {
  id: "quiz",
  title: "テスト教材",
  subject: "数学",
  questions: [],
};
const plan = {
  focus: "初期案",
  steps: ["", "", "", ""],
  checkQuestion: "",
  teacherNote: "",
};
afterEach(cleanup);
beforeEach(() => {
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "get_class_summary") return summary;
    if (cmd === "list_quizzes") return [quiz];
    if (cmd === "start_session") return "1234";
    if (cmd === "generate_lesson_plan") return plan;
    if (cmd === "chat_with_teacher") return "回答です";
    return [];
  });
});
describe("Mac navigation experience", () => {
  it("keeps the running session available after changing tabs", async () => {
    render(<TeacherDashboard />);
    fireEvent.click(screen.getByRole("button", { name: "セッション" }));
    fireEvent.click(await screen.findByRole("button", { name: /テスト教材/ }));
    fireEvent.click(screen.getByRole("button", { name: "この小テストを配信" }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "概要" })).toHaveAttribute(
        "aria-current",
        "page",
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "セッション" }));
    expect(screen.getByRole("button", { name: "配信を終了" })).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "配信を終了" }));
    await waitFor(() =>
      expect(screen.queryByText("1234")).not.toBeInTheDocument(),
    );
  });
  it("keeps an edited lesson and restores each tab's scroll position", async () => {
    render(<TeacherDashboard />);
    fireEvent.click(screen.getByRole("button", { name: "レッスンプラン" }));
    const input = await screen.findByDisplayValue("初期案");
    fireEvent.change(input, { target: { value: "編集中の案" } });
    const content = screen.getByRole("navigation", {
      name: "教師ナビゲーション",
    }).nextElementSibling as HTMLElement | null;
    expect(content).toBeTruthy();
    if (!content) throw new Error("Missing scroll container");
    content.scrollTop = 240;
    fireEvent.click(screen.getByRole("button", { name: "概要" }));
    expect(content.scrollTop).toBe(0);
    fireEvent.click(screen.getByRole("button", { name: "レッスンプラン" }));
    expect(screen.getByDisplayValue("編集中の案")).toBeVisible();
    expect(content.scrollTop).toBe(240);
  });
  it("does not scroll the page on chat mount or send during IME confirmation", async () => {
    const scrollIntoView = vi.fn();
    Element.prototype.scrollIntoView = scrollIntoView;
    render(<TeacherChat classSummary={null} activeQuiz={null} />);
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "質問" } });
    fireEvent.keyDown(input, { key: "Enter", isComposing: true });
    fireEvent.keyDown(input, { key: "Enter", keyCode: 229 });
    expect(invoke).not.toHaveBeenCalled();
    fireEvent.keyDown(input, { key: "Enter", shiftKey: true });
    expect(invoke).not.toHaveBeenCalled();
    fireEvent.keyDown(input, { key: "Enter" });
    await screen.findByText("回答です");
    expect(scrollIntoView).not.toHaveBeenCalled();
  });
  it("preserves chat drafts when switching tabs", async () => {
    render(<TeacherDashboard />);
    fireEvent.click(screen.getByRole("button", { name: "AIチャット" }));
    fireEvent.change(screen.getByRole("textbox"), {
      target: { value: "未送信の質問" },
    });
    fireEvent.click(screen.getByRole("button", { name: "概要" }));
    fireEvent.click(screen.getByRole("button", { name: "AIチャット" }));
    expect(screen.getByDisplayValue("未送信の質問")).toBeVisible();
  });
  it("finishes discovery and rejects out-of-range manual ports", async () => {
    render(<StudentJoin onJoined={vi.fn()} onReset={vi.fn()} />);
    await screen.findByText(/教室が見つかりません/);
    fireEvent.change(screen.getByPlaceholderText("IPアドレス"), {
      target: { value: "127.0.0.1" },
    });
    fireEvent.change(screen.getByPlaceholderText("ポート"), {
      target: { value: "65536" },
    });
    fireEvent.change(screen.getByPlaceholderText("4桁の参加コード"), {
      target: { value: "1234" },
    });
    expect(screen.getByRole("button", { name: "参加" })).toBeDisabled();
    fireEvent.change(screen.getByPlaceholderText("ポート"), {
      target: { value: "8080" },
    });
    expect(screen.getByRole("button", { name: "参加" })).toBeEnabled();
  });
});
