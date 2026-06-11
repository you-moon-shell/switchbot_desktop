import { createMemoryRouter } from "react-router";

import HomeRoute from "./routes/home";

/**
 * デスクトップアプリのため createMemoryRouter を使う
 * （URLバーが無いので history API に載せる意味がない）。
 */
export const router = createMemoryRouter([
  {
    path: "/",
    element: <HomeRoute />,
  },
  // /onboarding は features/credential 実装時に追加（ガードもその際に導入）
]);
