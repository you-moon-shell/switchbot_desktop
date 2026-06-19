import { Navigate } from "react-router";

import { GlassBackground } from "@/components/ui";
import { OnboardingForm, useHasCredentials } from "@/features/credential";

/**
 * /onboarding — Token/Secret 入力ページ（feature の合成層・薄い）。
 * 既に資格情報がある場合は / へ送り返す（A5）。
 */
export default function OnboardingRoute() {
  const { data: hasCreds, isPending } = useHasCredentials();

  if (isPending) return null; // 判定が出るまで一瞬だけ何も描かない（チラつき防止）
  if (hasCreds) return <Navigate to="/" replace />;

  return (
    <>
      <GlassBackground />
      <main className="relative z-10 flex min-h-screen items-center justify-center px-6 py-16">
        <OnboardingForm />
      </main>
    </>
  );
}
