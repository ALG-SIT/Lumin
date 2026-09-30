import "@testing-library/jest-dom/vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));

import { TeacherSessionControl } from "../TeacherSessionControl";

const quiz = {
  id: "quiz-1",
  title: "Fixture quiz",
  subject: "Math",
  questions: [
    {
      id: "q1",
      prompt: "What is 1 + 1?",
      concept: "addition",
      acceptedAnswers: ["2"],
      hints: ["Count up"],
      genericMisconception: "arithmetic error",
      explanation: "1 + 1 = 2",
    },
  ],
};

describe("TeacherSessionControl lifecycle", () => {
  beforeEach(() => {
    mocks.invoke.mockReset();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_teacher_session")
        return Promise.resolve({ quiz: null, joinCode: null });
      if (command === "list_quizzes") return Promise.resolve([quiz]);
      if (command === "list_students") return Promise.resolve([]);
      if (command === "start_session") return Promise.resolve("1234");
      if (command === "end_session") return Promise.resolve();
      return Promise.resolve();
    });
  });
  afterEach(cleanup);

  it("starts and ends a session, clearing the visible student list", async () => {
    const onSessionStarted = vi.fn();
    const onSessionEnded = vi.fn();
    render(
      <TeacherSessionControl
        onSessionStarted={onSessionStarted}
        onSessionEnded={onSessionEnded}
      />,
    );
    await screen.findByText("Fixture quiz");
    fireEvent.click(screen.getByRole("button", { name: /Fixture quiz/ }));
    fireEvent.click(screen.getByRole("button", { name: "この小テストを配信" }));

    await screen.findByText("1234");
    expect(onSessionStarted).toHaveBeenCalledWith("1234");
    expect(mocks.invoke).toHaveBeenCalledWith("start_session", {
      quizId: "quiz-1",
    });

    fireEvent.click(screen.getByRole("button", { name: "配信を終了" }));
    await waitFor(() => expect(onSessionEnded).toHaveBeenCalledOnce());
    expect(await screen.findByText("教材を選択")).toBeInTheDocument();
    expect(screen.queryByText("1234")).toBeNull();
  });

  it("keeps the start screen available and shows a start failure", async () => {
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_teacher_session")
        return Promise.resolve({ quiz: null, joinCode: null });
      if (command === "list_quizzes") return Promise.resolve([quiz]);
      if (command === "start_session") return Promise.reject("server busy");
      return Promise.resolve([]);
    });
    render(<TeacherSessionControl onSessionStarted={() => {}} />);
    await screen.findByText("Fixture quiz");
    fireEvent.click(screen.getByRole("button", { name: /Fixture quiz/ }));
    fireEvent.click(screen.getByRole("button", { name: "この小テストを配信" }));
    expect(await screen.findByText("server busy")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "この小テストを配信" }),
    ).toBeEnabled();
  });
});
