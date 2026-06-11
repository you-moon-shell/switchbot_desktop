import { invoke } from "@tauri-apps/api/core";

import { isCommandError, type CommandError } from "./types";

/**
 * Tauri の invoke の型付きラッパ。
 *
 * - 生の `invoke` を呼ぶのはこのファイルだけ（境界を1箇所に集約）
 * - 失敗時は必ず `CommandError` を reject する（呼び出し側は code で分岐できる）
 */
export async function ipcInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeError(error);
  }
}

/** Rust から来た reject（unknown）を CommandError に正規化する */
function normalizeError(error: unknown): CommandError {
  if (isCommandError(error)) {
    return error;
  }
  // コマンド名間違い等、CommandError 以外の reject（Tauri 自体のエラーは文字列で来る）
  return {
    code: "unexpected",
    message: typeof error === "string" ? error : "不明なエラーが発生しました",
  };
}
