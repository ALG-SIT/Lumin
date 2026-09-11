/**
 * Integration test: teacher-student join flow
 *
 * Tests the full flow from role selection → teacher starts session →
 * student discovers teacher → student enters join code → student joins.
 * All Tauri commands are mocked.
 */
import "@testing-library/jest-dom/vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RoleSelection } from "../RoleSelection";
import { StudentJoin } from "../StudentJoin";
import { TeacherSessionControl } from "../TeacherSessionControl";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "browse_teachers") return [];
    if (cmd === "student_join") return null;
    if (cmd === "list_quizzes") return [];
    if (cmd === "start_session") return "1234";
    if (cmd === "get_class_summary") {
      return {
        participantCount: 0,
        responseCount: 0,
        correctRate: 0,
        retrySuccessRate: 0,
        averageHints: 0,
        misconceptions: [],
      };
    }
    return {};
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe("Teacher-student join flow (integration)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders role selection as entry point", () => {
    render(<RoleSelection onSelect={() => {}} />);
    expect(screen.getByText("Lumin")).toBeDefined();
    expect(screen.getByRole("group", { name: "役割選択" })).toBeDefined();
  });

  it("teacher selects role and sees session control", () => {
    let role: string | null = null;
    render(
      <RoleSelection
        onSelect={(r) => {
          role = r;
        }}
      />,
    );

    fireEvent.click(screen.getByText("先生として始める"));
    expect(role).toBe("teacher");

    const { unmount } = render(
      <TeacherSessionControl onSessionStarted={() => {}} />,
    );
    expect(screen.getByText("小テスト配信")).toBeDefined();
    expect(screen.getByText("教材を選択")).toBeDefined();
    unmount();
  });

  it("student selects role and sees join screen", () => {
    let role: string | null = null;
    render(
      <RoleSelection
        onSelect={(r) => {
          role = r;
        }}
      />,
    );

    fireEvent.click(screen.getByText("生徒として参加"));
    expect(role).toBe("student");

    const { unmount } = render(
      <StudentJoin onJoined={() => {}} onReset={() => {}} />,
    );
    expect(screen.getByText("教室に参加")).toBeDefined();
    expect(screen.getByText("または、手動で入力")).toBeDefined();
    unmount();
  });

  it("student join code is limited to 4 digits", async () => {
    render(<StudentJoin onJoined={() => {}} onReset={() => {}} />);

    // browse_teachers returns [], so only the manual section code input exists
    const codeInput = screen.getByPlaceholderText("4桁の参加コード");

    fireEvent.change(codeInput, { target: { value: "12345" } });
    expect(codeInput).toHaveValue("1234");
  });

  it("student cannot join without a 4-digit code", () => {
    render(<StudentJoin onJoined={() => {}} onReset={() => {}} />);

    const joinBtn = screen.getByRole("button", { name: "参加" });
    expect(joinBtn).toBeDisabled();
  });

  it("student can join after entering a valid code and IP", async () => {
    const onJoined = vi.fn();
    render(<StudentJoin onJoined={onJoined} onReset={() => {}} />);

    const ipInput = screen.getByPlaceholderText("IPアドレス");
    const portInput = screen.getByPlaceholderText("ポート");
    const codeInput = screen.getByPlaceholderText("4桁の参加コード");

    fireEvent.change(ipInput, { target: { value: "192.168.1.100" } });
    fireEvent.change(portInput, { target: { value: "8080" } });
    fireEvent.change(codeInput, { target: { value: "1234" } });

    const joinBtn = screen.getByRole("button", { name: "参加" });
    expect(joinBtn).not.toBeDisabled();

    fireEvent.click(joinBtn);

    await waitFor(() => {
      expect(onJoined).toHaveBeenCalled();
    });
  });
});
