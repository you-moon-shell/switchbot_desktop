import React from "react";
import ReactDOM from "react-dom/client";

import { App } from "@/app";
import "./index.css";

// デスクトップアプリらしく、右クリックのコンテキストメニューを無効化する。
// 入力欄（貼り付け等）だけは許可し、body の user-select 方針と揃える。
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
