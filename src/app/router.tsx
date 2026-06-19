import { createMemoryRouter, Navigate, Outlet } from "react-router";

import { useHasCredentials } from "@/features/credential";

import HomeRoute from "./routes/home";
import OnboardingRoute from "./routes/onboarding";

/**
 * 資格情報が必要なルートのガード（A1）。
 * 無ければ /onboarding へ。判定中は一瞬だけ空表示（起動直後のスプラッシュ代わり）。
 * 保存/削除の mutation 成功で has_credentials が invalidate され、ここが自動で再判定する
 * ——画面遷移を手で書かないための要。
 */
function RequireCredentials() {
  const { data: hasCreds, isPending } = useHasCredentials();
  if (isPending) return null;
  if (!hasCreds) return <Navigate to="/onboarding" replace />;
  return <Outlet />;
}

/**
 * デスクトップアプリのため createMemoryRouter を使う
 * （URLバーが無いので history API に載せる意味がない）。
 */
export const router = createMemoryRouter([
  {
    path: "/onboarding",
    element: <OnboardingRoute />,
  },
  {
    element: <RequireCredentials />,
    children: [
      {
        path: "/",
        element: <HomeRoute />,
      },
    ],
  },
]);
