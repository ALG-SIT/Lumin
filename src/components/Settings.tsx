import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { secondaryButton } from "../styles/shared.css.ts";
import { ModelManager } from "./ModelManager";
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
    invoke<SystemInfo>("get_system_info")
      .then(setSystem)
      .catch(() => setSystem(null));
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
        <button type="button" className={secondaryButton} onClick={onClose}>
          閉じる
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
            <dd>{system.model_dir}</dd>
            <dt>推論ランタイム</dt>
            <dd>
              {system.ort_available ? "ONNX Runtime 利用可能" : "利用不可"}
            </dd>
          </dl>
        </div>
      )}
    </section>
  );
}
