import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "react-router";

import { router } from "./router";

// invoke はローカルIPC（ネットワークではない）ため、ブラウザ向けの自動再取得は不要。
// プロバイダが増えたら app/provider.tsx として切り出す。
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: false,
    },
  },
});

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      {/* オーバーレイ型タイトルバー用: 上部の透明な帯でウィンドウをドラッグ移動 */}
      <div data-tauri-drag-region className="titlebar-drag" />
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
