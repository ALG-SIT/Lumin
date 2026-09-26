import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { iconButton } from "./App.css.ts";
import { ModelManager } from "./ModelManager";
import type { ModelLoadProgress } from "./modelTypes";
import { CloseIcon } from "./NavigationIcons";
import {
  settings,
  settingsHeader,
  settingsSection,
  systemGrid,
} from "./Settings.css.ts";

export interface SettingsProps {
  onClose: () => void;
}

interface SystemInfo {
  platform: string;
  arch: string;
  tauri_version: string;
  ort_available: boolean;
  model_dir: string;
  execution_provider: string | null;
}

function modelDirectoryLabel(system: SystemInfo) {
  if (system.platform === "ios") {
    return "このiPhone内（アプリ専用領域）";
  }
  return system.model_dir;
}

/**
 * App-wide settings, reachable from the bar in any role and before one is
 * chosen.
 *
 * The model is the reason this screen exists: it is shared by the teacher's
 * chat and lesson plans and by the hints students receive, so choosing and
 * downloading one is not a teacher-dashboard concern.
 */
export function Settings({ onClose }: SettingsProps) {
  const [system, setSystem] = useState<SystemInfo | null>(null);

  useEffect(() => {
    let cancelled = false;
    let stop: (() => void) | undefined;
    const refresh = async () => {
      try {
        const info = await invoke<SystemInfo>("get_system_info");
        if (!cancelled) setSystem(info);
      } catch {
        // Keep the last known status if a refresh fails.
      }
    };

    listen<ModelLoadProgress>("model-load-progress", ({ payload }) => {
      if (!cancelled && payload.stage === "ready") void refresh();
    })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else stop = unlisten;
      })
      .catch(() => {})
      .finally(() => {
        if (!cancelled) void refresh();
      });

    return () => {
      cancelled = true;
      stop?.();
    };
  }, []);

  return (
    <section className={settings} aria-label="設定">
      <header className={settingsHeader}>
        <div>
          <h2>設定</h2>
          <p>
            モデルの選択と取得は、先生・生徒どちらの画面にも共通で効きます。
          </p>
        </div>
        <button
          type="button"
          className={iconButton}
          onClick={onClose}
          aria-label="閉じる"
          title="閉じる"
        >
          <CloseIcon />
        </button>
      </header>

      <div className={settingsSection}>
        <ModelManager />
      </div>

      {system && (
        <div className={settingsSection}>
          <h3>この端末</h3>
          <dl className={systemGrid}>
            <dt>プラットフォーム</dt>
            <dd>
              {system.platform} / {system.arch}
            </dd>
            <dt>モデルの保存先</dt>
            <dd
              title={system.platform === "ios" ? undefined : system.model_dir}
            >
              {modelDirectoryLabel(system)}
            </dd>
            <dt>推論ランタイム</dt>
            <dd>
              {system.ort_available ? "ONNX Runtime 利用可能" : "利用不可"}
            </dd>
            <dt>実行プロバイダ</dt>
            <dd>{system.execution_provider ?? "推論開始後に表示"}</dd>
          </dl>
        </div>
      )}
    </section>
  );
}
