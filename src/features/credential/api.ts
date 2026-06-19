import { ipcInvoke } from "@/lib/ipc";

/**
 * 資格情報まわりの invoke 呼び出し。
 * コマンド文字列（Rust 側 `commands::credential::*`）はこの feature が所有し、外に散らさない。
 */

/** 検証して成功時のみ保存（A3/A4）。失敗時は CommandError を throw。 */
export function saveCredentials(token: string, secret: string): Promise<void> {
  return ipcInvoke<void>("save_credentials", { token, secret });
}

/** 資格情報が保存済みか（A1/A5）。bool のみ返る。 */
export function hasCredentials(): Promise<boolean> {
  return ipcInvoke<boolean>("has_credentials");
}

/** 資格情報を削除（A6 ログアウト）。 */
export function deleteCredentials(): Promise<void> {
  return ipcInvoke<void>("delete_credentials");
}
