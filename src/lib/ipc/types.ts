/**
 * Rust 側 `commands/error.rs` と対になる型。
 *
 * - `code`: 分岐に使う（例: unauthorized → 再認証導線へ）
 * - `message`: ユーザーに表示する説明文（Rust 側で日本語化済み。フロントで文言を作らない）
 */
export interface CommandError {
  code: ErrorCode;
  message: string;
}

/** Rust 側 `ErrorCode` enum（serde rename_all = "snake_case"）と対 */
export type ErrorCode =
  | "empty_input"
  | "unauthorized"
  | "rate_limited"
  | "network"
  | "secret"
  | "unexpected";

/** 値が CommandError の形をしているかの型ガード */
export function isCommandError(value: unknown): value is CommandError {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return typeof v.code === "string" && typeof v.message === "string";
}
