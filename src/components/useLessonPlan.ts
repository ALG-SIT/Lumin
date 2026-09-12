import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { isTauriEnvironment } from "../lib/tauri";
import { useElapsed } from "./GenerationProgress";
import type { ModelLoadProgress } from "./modelTypes";
import type { Quiz } from "./StudentQuiz";
import type { ClassSummary } from "./TeacherChat";

export interface LessonPlan {
  focus: string;
  steps: [string, string, string, string];
  checkQuestion: string;
  teacherNote: string;
}

/**
 * One update from the backend while a plan is being generated.
 *
 * A plan takes the full token budget and may be generated twice (the model's
 * JSON is validated, and one repair pass is allowed), so this covers a wait
 * that runs to a couple of minutes on a small device.
 */
export type PlanProgress =
  | { stage: "prompt"; done: number; total: number; attempt: number }
  | { stage: "generating"; generated: number; max: number; attempt: number }
  | { stage: "validating"; attempt: number }
  | { stage: "repairing"; reason: string }
  | { stage: "fallback"; reason: string }
  | { stage: "done" };

export const EMPTY_PLAN: LessonPlan = {
  focus: "",
  steps: ["", "", "", ""],
  checkQuestion: "",
  teacherNote: "",
};

/**
 * Ask the backend for a plan, reporting progress when the host can carry it.
 *
 * Outside a Tauri host (a plain browser, or tests) there is no channel, so it
 * falls back to the plain command and the caller simply sees no stages.
 */
async function requestPlan(
  classSummary: ClassSummary,
  quiz: Quiz | null | undefined,
  onProgress: (progress: PlanProgress) => void,
): Promise<LessonPlan> {
  const args = {
    classSummaryJson: JSON.stringify(classSummary),
    quizJson: quiz ? JSON.stringify(quiz) : null,
  };
  if (isTauriEnvironment() && typeof Channel === "function") {
    try {
      const channel = new Channel<PlanProgress>();
      channel.onmessage = onProgress;
      return await invoke<LessonPlan>("generate_lesson_plan_stream", {
        ...args,
        onProgress: channel,
      });
    } catch (e) {
      // A missing channel is a host limitation, but a generation failure is
      // real and must not be retried silently as a second full generation.
      if (!(e instanceof TypeError)) throw e;
    }
  }
  return invoke<LessonPlan>("generate_lesson_plan", args);
}

/** Everything the lesson plan editor renders and drives. */
export interface LessonPlanDraft {
  plan: LessonPlan;
  /** Editing the plan; marks it as the teacher's so generation cannot clobber it. */
  setPlan: (next: LessonPlan | ((current: LessonPlan) => LessonPlan)) => void;
  busy: boolean;
  progress: PlanProgress | null;
  modelLoading: boolean;
  elapsedMs: number;
  error: string | null;
  setError: (message: string | null) => void;
  regenerate: () => void;
}

/**
 * Owns the lesson plan and its generation.
 *
 * This lives above the tab switch on purpose. Generating a plan takes tens of
 * seconds, and the editor is unmounted whenever another tab is shown: when the
 * work belonged to the editor, leaving and returning abandoned the running
 * generation, started a second one behind it on the model, and left the
 * returning screen with no sign that anything was in flight.
 *
 * `enabled` starts the first generation - pass it true once the teacher has
 * actually opened the lesson plan, so opening the dashboard does not spend a
 * minute of the model on a plan nobody asked for.
 */
export function useLessonPlan(
  classSummary: ClassSummary | null,
  quiz: Quiz | null | undefined,
  enabled: boolean,
): LessonPlanDraft {
  const [plan, setPlanState] = useState<LessonPlan>(EMPTY_PLAN);
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<PlanProgress | null>(null);
  const [modelLoading, setModelLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const elapsedMs = useElapsed(busy);

  const edited = useRef(false);
  /** Set while a generation is running, so a second one is never started. */
  const inFlight = useRef(false);
  /** The automatic first generation has been asked for. */
  const requested = useRef(false);

  // The first plan of a session also pays for loading the model into memory,
  // which is the larger part of the wait and is reported by the app bar.
  useEffect(() => {
    if (!isTauriEnvironment()) return;
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    listen<ModelLoadProgress>("model-load-progress", (event) => {
      if (cancelled) return;
      setModelLoading(!event.payload.done);
    }).then((stop) => {
      if (cancelled) stop();
      else unlisten = stop;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const generate = useCallback(
    async (manual: boolean) => {
      // One generation at a time: they queue on the model anyway, and a second
      // one only doubles the wait for a result that replaces the first.
      if (inFlight.current || !classSummary) return;
      inFlight.current = true;
      setBusy(true);
      setProgress(null);
      setError(null);
      try {
        const result = await requestPlan(classSummary, quiz, (next) => {
          setProgress((current) =>
            next.stage === "done" && current?.stage === "fallback"
              ? current
              : next,
          );
        });
        // An automatic plan fills in fields the teacher has not claimed; an
        // explicit regenerate is what they asked for, so it always applies.
        if (manual || !edited.current) setPlanState(result);
      } catch (e) {
        setError(`授業案を生成できませんでした: ${e}`);
      } finally {
        inFlight.current = false;
        setBusy(false);
        setProgress((current) =>
          current?.stage === "fallback" ? current : null,
        );
        setModelLoading(false);
      }
    },
    [classSummary, quiz],
  );

  useEffect(() => {
    if (!enabled || requested.current || !classSummary) return;
    requested.current = true;
    void generate(false);
  }, [enabled, classSummary, generate]);

  const setPlan = useCallback(
    (next: LessonPlan | ((current: LessonPlan) => LessonPlan)) => {
      edited.current = true;
      setPlanState(next);
    },
    [],
  );

  const regenerate = useCallback(() => {
    void generate(true);
  }, [generate]);

  return {
    plan,
    setPlan,
    busy,
    progress,
    modelLoading,
    elapsedMs,
    error,
    setError,
    regenerate,
  };
}
