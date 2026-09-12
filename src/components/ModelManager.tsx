import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import {
  emptyState,
  errorMessage,
  modelManager,
  progressBar,
  progressFill,
} from "../styles/shared.css.ts";
import {
  activeBadge,
  activeCard,
  badge,
  familyGroup,
  familyHeading,
  importHint,
  importSection,
  modelActions,
  modelCard,
  modelDescription,
  modelMeta,
  modelName,
  statusBadge,
} from "./ModelManager.css.ts";

interface ModelEntry {
  id: string;
  name: string;
  size_bytes: number;
  variant: string;
  family: string;
  description: string;
  status: "installed" | "downloading" | "available";
  progress?: number;
  recommended?: boolean;
  active?: boolean;
}

interface DownloadProgress {
  file: string;
  downloaded: number;
  total?: number;
  percent?: number;
  done: boolean;
  error?: string;
}

type ProgressMap = Record<string, DownloadProgress>;

export function ModelManager() {
  const [models, setModels] = useState<ModelEntry[]>([]);
  const [downloadingVariant, setDownloadingVariant] = useState<string | null>(
    null,
  );
  const [progress, setProgress] = useState<ProgressMap>({});
  const [error, setError] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const [switching, setSwitching] = useState(false);

  const loadModels = useCallback(async () => {
    try {
      const entries = await invoke<ModelEntry[]>("list_models");
      setModels(entries);
      setError(null);
    } catch (err) {
      setError(`モデル一覧の取得に失敗しました: ${err}`);
    }
  }, []);

  useEffect(() => {
    loadModels();
  }, [loadModels]);

  useEffect(() => {
    let unlistenProgress: (() => void) | undefined;
    let unlistenComplete: (() => void) | undefined;

    const setupListeners = async () => {
      unlistenProgress = await listen<DownloadProgress>(
        "download-progress",
        (event) => {
          setProgress((prev) => {
            const next = { ...prev, [event.payload.file]: event.payload };
            return next;
          });
        },
      );

      unlistenComplete = await listen<unknown>("download-complete", () => {
        setDownloadingVariant(null);
        setProgress({});
        loadModels();
      });
    };

    setupListeners();

    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenComplete) unlistenComplete();
    };
  }, [loadModels]);

  const overallPercent = (): number => {
    const entries = Object.values(progress);
    if (entries.length === 0) return 0;
    const percents = entries
      .map((p) => p.percent)
      .filter((p): p is number => p != null);
    if (percents.length === 0) return 0;
    const sum = percents.reduce((a, b) => a + b, 0);
    return Math.round(sum / percents.length);
  };

  const handleDownload = async (variant: string) => {
    setDownloadingVariant(variant);
    setProgress({});
    setError(null);
    try {
      await invoke("download_model", { variant });
    } catch (err) {
      setError(`ダウンロードに失敗しました: ${err}`);
    } finally {
      setDownloadingVariant(null);
      setProgress({});
      loadModels();
    }
  };

  const handleCancel = async () => {
    try {
      await invoke("cancel_download");
    } catch (err) {
      setError(`キャンセル要求に失敗しました: ${err}`);
    }
  };

  const handleSelect = async (variant: string) => {
    setSwitching(true);
    setError(null);
    try {
      await invoke("set_active_model", { variant });
      await loadModels();
    } catch (err) {
      setError(`モデルの切り替えに失敗しました: ${err}`);
    } finally {
      setSwitching(false);
    }
  };

  const handleImport = async () => {
    setImporting(true);
    setError(null);
    try {
      const onnxPath = await open({
        multiple: false,
        filters: [{ name: "ONNX model", extensions: ["onnx"] }],
      });
      if (!onnxPath || Array.isArray(onnxPath)) {
        setImporting(false);
        return;
      }

      const tokenizerPath = await open({
        multiple: false,
        filters: [{ name: "Tokenizer", extensions: ["json"] }],
      });
      if (!tokenizerPath || Array.isArray(tokenizerPath)) {
        setImporting(false);
        return;
      }

      // Import into whichever variant is currently selected, so the files land
      // in that variant's directory and are checked against its hashes.
      const activeVariant =
        models.find((m) => m.active)?.variant ?? models[0]?.variant;
      await invoke("import_model", {
        onnxPath,
        tokenizerPath,
        variant: activeVariant,
      });
      await loadModels();
    } catch (err) {
      setError(`モデルの取り込みに失敗しました: ${err}`);
    } finally {
      setImporting(false);
    }
  };

  const families = groupByFamily(models);

  return (
    <div className={modelManager}>
      <h2>モデル管理</h2>

      {models.length === 0 && (
        <div className={emptyState}>モデル情報を取得中…</div>
      )}

      {families.map(([family, entries]) => (
        <div key={family} className={familyGroup}>
          <h3 className={familyHeading}>{family}</h3>

          {entries.map((model) => (
            <div
              key={model.id}
              className={`${modelCard}${model.active ? ` ${activeCard}` : ""}`}
            >
              <div className={modelName}>
                {model.name}
                {model.recommended && <span className={badge}>推奨</span>}
                {model.active && <span className={activeBadge}>使用中</span>}
                <div className={modelDescription}>{model.description}</div>
              </div>

              <div className={modelMeta}>
                <span>{formatBytes(model.size_bytes)}</span>
                {model.status === "installed" ? (
                  <span className={statusBadge}>✓ 導入済み</span>
                ) : (
                  <span>未導入</span>
                )}
              </div>

              <div className={modelActions}>
                {model.status === "installed" && !model.active && (
                  <button
                    type="button"
                    onClick={() => handleSelect(model.variant)}
                    disabled={switching || downloadingVariant != null}
                  >
                    使用する
                  </button>
                )}

                {model.status === "available" &&
                  downloadingVariant !== model.variant && (
                    <button
                      type="button"
                      onClick={() => handleDownload(model.variant)}
                      disabled={downloadingVariant != null}
                    >
                      ダウンロード
                    </button>
                  )}

                {downloadingVariant === model.variant && (
                  <>
                    <div className={progressBar}>
                      <div
                        className={progressFill}
                        style={{ width: `${overallPercent()}%` }}
                      />
                    </div>
                    <span>{overallPercent()}%</span>
                    <button type="button" onClick={handleCancel}>
                      中止
                    </button>
                  </>
                )}
              </div>
            </div>
          ))}
        </div>
      ))}

      <div className={importSection}>
        <h3>手元のモデルを取り込む</h3>
        <p className={importHint}>
          ローカルの .onnx と tokenizer.json
          のペアを選択してください。ファイルは検証された上で、使用中のモデルとして保存されます。
        </p>
        <button
          type="button"
          onClick={handleImport}
          disabled={importing || downloadingVariant != null}
        >
          {importing ? "取り込み中…" : "モデルを読み込む"}
        </button>
      </div>

      {error && <div className={errorMessage}>{error}</div>}
    </div>
  );
}

/// Group entries by family, preserving the order the backend sent them in.
function groupByFamily(models: ModelEntry[]): [string, ModelEntry[]][] {
  const groups: [string, ModelEntry[]][] = [];
  for (const model of models) {
    const existing = groups.find(([family]) => family === model.family);
    if (existing) {
      existing[1].push(model);
    } else {
      groups.push([model.family, [model]]);
    }
  }
  return groups;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  }
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}
