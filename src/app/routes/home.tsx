import { Boxes, LayoutGrid, LogOut, MousePointerClick, UserRound } from "lucide-react";
import { useState } from "react";

import { Button, GlassBackground } from "@/components/ui";
import { useDeleteCredential } from "@/features/credential";
import { type Device, DeviceDetailPanel, DeviceList } from "@/features/device";

/**
 * 3ペインシェル（左ナビ / 中央リスト / 右詳細）。枠と選択 state を持つ合成層。
 * ログアウトと 401 時の再認証は credential の delete を組み立てて渡す。
 */
export default function HomeRoute() {
  const logout = useDeleteCredential();
  const [selected, setSelected] = useState<Device | null>(null);

  return (
    <>
      <GlassBackground />
      <div className="app-shell">
        <nav className="nav">
          <div className="nav__brand">
            <Boxes size={18} aria-hidden="true" /> SwitchBot Desktop
          </div>

          <div className="nav__section">メニュー</div>
          <button type="button" className="nav__item nav__item--active">
            <LayoutGrid size={18} aria-hidden="true" /> デバイス
          </button>

          <div className="nav__spacer" />

          <div className="nav__account">
            <div className="nav__avatar" aria-hidden="true">
              <UserRound size={16} />
            </div>
            <div className="min-w-0 flex-1">
              <div className="truncate text-sm">接続済み</div>
            </div>
            <button
              type="button"
              onClick={() => logout.mutate()}
              disabled={logout.isPending}
              aria-label="ログアウト"
              title="ログアウト"
              className="icon-btn shrink-0"
            >
              <LogOut size={16} />
            </button>
          </div>
        </nav>

        <main className="list-pane">
          <DeviceList
            selectedId={selected?.deviceId ?? null}
            onSelect={setSelected}
            reauthSlot={
              <Button color="info" onClick={() => logout.mutate()} disabled={logout.isPending}>
                {logout.isPending ? "切断中…" : "再認証する"}
              </Button>
            }
          />
        </main>

        <aside className="detail-pane">
          {selected ? (
            <DeviceDetailPanel device={selected} />
          ) : (
            <div className="detail-pane__empty">
              <MousePointerClick size={40} aria-hidden="true" />
              <span>デバイスを選択</span>
            </div>
          )}
        </aside>
      </div>
    </>
  );
}
