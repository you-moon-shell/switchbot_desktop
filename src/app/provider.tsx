import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // invoke はローカルIPC（ネットワークではない）ため、ブラウザ向けの自動再取得は不要
      refetchOnWindowFocus: false,
      retry: false,
    },
  },
});

export function AppProvider({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}
