import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { CommandError } from "@/lib/ipc";

import { deleteCredentials, hasCredentials, saveCredentials } from "./api";

/** has_credentials のクエリキー。保存／削除の成功時に invalidate して再判定させる。 */
const HAS_CREDENTIALS_KEY = ["credentials", "has"] as const;

/** 資格情報の有無（ガードと画面振り分けの根拠）。 */
export function useHasCredentials() {
  return useQuery<boolean, CommandError>({
    queryKey: HAS_CREDENTIALS_KEY,
    queryFn: hasCredentials,
  });
}

/** 検証して保存。成功すると has_credentials を無効化 → ガードが反応して自動遷移する。 */
export function useSaveCredentials() {
  const queryClient = useQueryClient();
  return useMutation<void, CommandError, { token: string; secret: string }>({
    mutationFn: ({ token, secret }) => saveCredentials(token, secret),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: HAS_CREDENTIALS_KEY });
    },
  });
}

/** ログアウト（削除）。成功すると同様にガードが /onboarding へ戻す。 */
export function useDeleteCredentials() {
  const queryClient = useQueryClient();
  return useMutation<void, CommandError, void>({
    mutationFn: () => deleteCredentials(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: HAS_CREDENTIALS_KEY });
    },
  });
}
