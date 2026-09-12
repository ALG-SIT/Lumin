import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import { isTauriEnvironment } from "../lib/tauri";
import {
  modelChip,
  modelChipDot,
  modelChipFill,
  modelChipName,
  modelChipRow,
  modelChipState,
  modelChipTrack,
} from "./AppBarModelStatus.css.ts";
import type { ModelEntry, ModelLoadProgress } from "./modelTypes";

export interface AppBarModelStatusProps {
  /** Opens the settings screen, where the model is chosen and downloaded. */
  onOpenSettings: () => void;
}

/**
 * The active model, its readiness, and its load progress, shown in the app bar.
 *
 * Which model is running is app-wide state: it applies to the teacher's chat,
 * the lesson plan and the students' hints alike, and a load started on one
 * screen is still running when you move to another. Putting it in the bar is
 * what lets every screen stop keeping its own copy.
 */
export function AppBarModelStatus({ onOpenSettings }: AppBarModelStatusProps) {
  const [active, setActive] = useState<ModelEntry | null>(null);
  const [load, setLoad] = useState<ModelLoadProgress | null>(null);

  const refresh = useCallback(async () => {
    try {
      const models = await invoke<ModelEntry[]>("list_models");
      setActive(models.find((m) => m.active) ?? null);
    } catch {
      // The bar is ambient status; a failed refresh must not break the app.
    }
  }, []);

  useEffect(() => {
    if (!isTauriEnvironment()) return;
    let cancelled = false;
    const stops: (() => void)[] = [];

    const track = <T,>(name: string, handler: (payload: T) => void) => {
      listen<T>(name, (event) => {
        if (!cancelled) handler(event.payload);
      }).then((stop) => (cancelled ? stop() : stops.push(stop)));
    };

    track<ModelLoadProgress>("model-load-progress", (payload) => {
      setLoad(payload.stage === "ready" ? null : payload);
      if (payload.stage === "ready") refresh();
    });
    // A switch or a finished download changes what the bar should name.
    track<ModelEntry>("active-model-changed", (payload) => setActive(payload));
    track<unknown>("download-complete", () => refresh());

    refresh();
    return () => {
      cancelled = true;
      for (const stop of stops) stop();
    };
  }, [refresh]);

  const state = load
    ? load.stage === "error"
      ? "error"
      : "loading"
    : active?.status === "installed"
      ? "ready"
      : "missing";

  const caption = {
    loading: load
      ? `${load.label} ${Math.round(load.percent)}%`
      : "読み込み中…",
    error: "読み込みに失敗しました",
    ready: "使用できます",
    missing: "未導入 — 選択してください",
  }[state];

  return (
    <button
      type="button"
      className={modelChip}
      onClick={onOpenSettings}
      aria-label={`使用中のモデル: ${active?.name ?? "未選択"}（${caption}）。設定を開く`}
    >
      <span className={modelChipRow}>
        <span className={modelChipDot} data-state={state} aria-hidden="true" />
        <span className={modelChipName}>{active?.name ?? "モデル未選択"}</span>
      </span>
      <span className={modelChipState}>{caption}</span>
      {state === "loading" && load && (
        <span
          className={modelChipTrack}
          role="progressbar"
          aria-label="モデルの読み込み"
          aria-valuenow={Math.round(load.percent)}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <span
            className={modelChipFill}
            style={{ width: `${Math.max(load.percent, 3)}%` }}
          />
        </span>
      )}
    </button>
  );
}
