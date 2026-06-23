import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { CommandError } from "@/lib/ipc";

import { deleteCredential, hasCredential, saveCredential } from "./api";

/** has_credential のクエリキー。保存／削除の成功時に invalidate して再判定させる。 */
const HAS_CREDENTIAL_KEY = ["credential", "has"] as const;

/** 資格情報の有無（ガードと画面振り分けの根拠）。 */
export function useHasCredential() {
  return useQuery<boolean, CommandError>({
    queryKey: HAS_CREDENTIAL_KEY,
    queryFn: hasCredential,
  });
}

/** 検証して保存。成功すると has_credential を無効化 → ガードが反応して自動遷移する。 */
export function useSaveCredential() {
  const queryClient = useQueryClient();
  return useMutation<void, CommandError, { token: string; secret: string }>({
    mutationFn: ({ token, secret }) => saveCredential(token, secret),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: HAS_CREDENTIAL_KEY });
    },
  });
}

/** ログアウト（削除）。成功すると同様にガードが /onboarding へ戻す。 */
export function useDeleteCredential() {
  const queryClient = useQueryClient();
  return useMutation<void, CommandError, void>({
    mutationFn: () => deleteCredential(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: HAS_CREDENTIAL_KEY });
    },
  });
}
