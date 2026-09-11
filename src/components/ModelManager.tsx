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
  badge,
  importHint,
  importSection,
  modelActions,
  modelCard,
  modelMeta,
  modelName,
  statusBadge,
} from "./ModelManager.css.ts";

interface ModelEntry {
  id: string;
  name: string;
  size_bytes: number;
  variant: string;
  status: "installed" | "downloading" | "available";
  progress?: number;
  recommended?: boolean;
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

  const loadModels = useCallback(async () => {
    try {
      const entries = await invoke<ModelEntry[]>("list_models");
      setModels(entries);
    } catch (err) {
      setError(`モデル一覧の取得に失敗しました: ${err}`);
    }
  }, []);

  useEffect(() => {
    loadModels();
  }, [loadModels]);

  useEffect(() => {
    let cancelled = false;
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

      if (cancelled) {
        unlistenProgress();
        return;
      }

      unlistenComplete = await listen<unknown>("download-complete", () => {
        setDownloadingVariant(null);
        setProgress({});
        loadModels();
      });
      if (cancelled) unlistenComplete();
    };

    void setupListeners().catch((err) => {
      if (!cancelled) setError(`進捗の取得に失敗しました: ${err}`);
    });

    return () => {
      cancelled = true;
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

      await invoke("import_model", { onnxPath, tokenizerPath });
      await loadModels();
    } catch (err) {
      setError(`モデルの取り込みに失敗しました: ${err}`);
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className={modelManager}>
      <h2>モデル管理</h2>

      {models.length === 0 && (
        <div className={emptyState}>モデル情報を取得中…</div>
      )}

      {models.map((model) => (
        <div key={model.id} className={modelCard}>
          <div className={modelName}>
            {model.name}
            {model.recommended && <span className={badge}>推奨</span>}
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

      <div className={importSection}>
        <h3>手元のモデルを取り込む</h3>
        <p className={importHint}>
          ローカルの .onnx と tokenizer.json
          のペアを選択してください。ファイルは検証された上で保存されます。
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

function formatBytes(bytes: number): string {
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  }
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}
