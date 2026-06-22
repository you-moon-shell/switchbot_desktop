import { RouterProvider } from "react-router";

import { AppProvider } from "./provider";
import { router } from "./router";

export function App() {
  return (
    <AppProvider>
      {/* オーバーレイ型タイトルバー用: 上部の透明な帯でウィンドウをドラッグ移動 */}
      <div data-tauri-drag-region className="titlebar-drag" />
      <RouterProvider router={router} />
    </AppProvider>
  );
}
