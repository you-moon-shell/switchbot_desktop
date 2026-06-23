import { AlertTriangle } from "lucide-react";
import { type FormEvent, useState } from "react";

import { Button, Card, Field, Input } from "@/components/ui";

import { useSaveCredential } from "../hooks";

/**
 * Token / Secret 入力フォーム（A2/A3）。検証・保存はバックエンドに委譲。
 * 成功時の遷移は書かない（mutation 成功 → invalidate → ガードが遷移）。
 */
export function OnboardingForm() {
  const [token, setToken] = useState("");
  const [secret, setSecret] = useState("");
  const save = useSaveCredential();

  const canSubmit = token.trim() !== "" && secret.trim() !== "" && !save.isPending;

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canSubmit) return;
    save.mutate({ token, secret });
  }

  return (
    <Card label="ONBOARDING" title="SwitchBot に接続" className="w-full max-w-md">
      <p className="glass-card__body mt-2 mb-6">
        SwitchBotアプリ → プロフィール → 設定 → 開発者向けオプション で取得した Token / Secret を入力してください。
      </p>

      <form onSubmit={handleSubmit} className="flex flex-col gap-4" noValidate>
        <Field id="token" label="トークン">
          <Input
            id="token"
            value={token}
            onChange={(e) => setToken(e.target.value)}
            placeholder="API トークン"
            autoFocus
            autoComplete="off"
            spellCheck={false}
          />
        </Field>

        <Field id="secret" label="シークレット">
          <Input
            id="secret"
            type="password"
            value={secret}
            onChange={(e) => setSecret(e.target.value)}
            placeholder="シークレット"
            autoComplete="off"
            spellCheck={false}
          />
        </Field>

        {save.isError && (
          <p className="glass-field-error" role="alert">
            <AlertTriangle size={14} aria-hidden="true" />
            {save.error.message}
          </p>
        )}

        <Button type="submit" color="info" block disabled={!canSubmit}>
          {save.isPending ? "検証中…" : "接続して保存"}
        </Button>
      </form>
    </Card>
  );
}
