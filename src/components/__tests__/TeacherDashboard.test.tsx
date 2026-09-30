import "@testing-library/jest-dom/vitest";
import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listener: undefined as unknown,
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: (_name: string, callback: (event: { payload: unknown }) => void) => {
    mocks.listener = callback;
    return Promise.resolve(vi.fn());
  },
}));
vi.mock("../TeacherChat", () => ({ TeacherChat: () => null }));
vi.mock("../LessonPlanEditor", () => ({ LessonPlanEditor: () => null }));
vi.mock("../TeacherSessionControl", () => ({
  TeacherSessionControl: () => null,
}));
vi.mock("../useLessonPlan", () => ({ useLessonPlan: () => ({}) }));

import { TeacherDashboard } from "../TeacherDashboard";

const summary = (responseCount: number, correctRate: number) => ({
  participantCount: 1,
  responseCount,
  correctRate,
  retrySuccessRate: 0,
  averageHints: 0,
  misconceptions: [],
});

describe("TeacherDashboard event refresh", () => {
  beforeEach(() => {
    mocks.invoke.mockReset();
    mocks.listener = undefined;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_class_summary")
        return Promise.resolve(summary(1, 0.5));
      if (command === "get_teacher_session")
        return Promise.resolve({ quiz: null, joinCode: null });
      return Promise.resolve();
    });
  });
  afterEach(cleanup);

  it("refreshes the teacher summary when a student event arrives", async () => {
    render(<TeacherDashboard />);
    await screen.findByText("50%");
    expect(mocks.listener).toBeTypeOf("function");

    mocks.invoke.mockImplementation((command: string) => {
      if (command === "get_class_summary")
        return Promise.resolve(summary(2, 0.75));
      if (command === "get_teacher_session")
        return Promise.resolve({ quiz: null, joinCode: null });
      return Promise.resolve();
    });
    await act(async () => {
      (mocks.listener as (event: { payload: unknown }) => void)({
        payload: {
          id: "event-1",
          participantToken: "participant-1",
          sessionId: "session-1",
          questionID: "q1",
          concept: "fractions",
          misconception: null,
          correct: true,
          hintCount: 0,
          retrySuccess: false,
          submittedAt: 1,
        },
      });
      await Promise.resolve();
      await Promise.resolve();
    });

    await waitFor(() => expect(screen.getByText("75%")).toBeInTheDocument());
    expect(screen.getByText("全2回答")).toBeInTheDocument();
  });
});
