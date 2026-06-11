import { RouterProvider } from "react-router";

import { AppProvider } from "./provider";
import { router } from "./router";

export function App() {
  return (
    <AppProvider>
      <RouterProvider router={router} />
    </AppProvider>
  );
}
