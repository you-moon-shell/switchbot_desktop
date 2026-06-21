import { RouterProvider } from "react-router";

import { AppProvider } from "./provider";
import { router } from "./router";

export function App() {
  return (
    <AppProvider>
      {/* オーバーレイ型タイトルバー: 上部の透明な帯をドラッグでウィンドウ移動できるようにする
          （信号機ボタンはネイティブで上に描画されクリック可。コンテンツは下に逃がす） */}
      <div data-tauri-drag-region className="titlebar-drag" />
      <RouterProvider router={router} />
    </AppProvider>
  );
}
