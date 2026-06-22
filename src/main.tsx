import React from "react";
import ReactDOM from "react-dom/client";

import { App } from "@/app";
import "./index.css";

// 右クリックメニューを無効化（入力欄だけ許可）。
window.addEventListener("contextmenu", (event) => {
  const target = event.target as HTMLElement | null;
  if (target?.closest("input, textarea, [contenteditable='true']")) return;
  event.preventDefault();
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
