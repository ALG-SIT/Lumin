/**
 * Tauriホスト内で動作しているかを実行時に判定する。
 * `bun run dev`(素のブラウザ)では false になり、IPCを持たないことを示す。
 */
export function isTauriEnvironment(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
