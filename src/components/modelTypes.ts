/**
 * Shapes the backend sends about models, shared by the app bar, the settings
 * screen and the screens that wait on a model being loaded.
 */

/** One entry of the `list_models` command. */
export interface ModelEntry {
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

/** Payload of the backend's `model-load-progress` event. */
export interface ModelLoadProgress {
  variant: string;
  modelName: string;
  stage: "tokenizer" | "decoder" | "embed" | "ready" | "error";
  label: string;
  percent: number;
  done: boolean;
  error?: string;
}
