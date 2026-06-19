import { Badge, Button, Card, GlassBackground } from "@/components/ui";
import { useDeleteCredentials } from "@/features/credential";

/**
 * ホーム（仮）。資格情報あり時の着地点。
 * 今は接続状態とログアウト（A6）のみ。Epic C でテレメトリ + デバイスのダッシュボードになる。
 */
export default function HomeRoute() {
  const logout = useDeleteCredentials();

  return (
    <>
      <GlassBackground />
      <main className="relative z-10 mx-auto flex min-h-screen max-w-2xl flex-col gap-8 px-6 py-16">
        <header className="flex flex-col gap-2">
          <div className="flex items-center justify-between gap-4">
            <h1 className="text-4xl" style={{ fontFamily: "var(--font-display)" }}>
              SwitchBot Desktop
            </h1>
            <Badge color="lime" dot>
              接続済み
            </Badge>
          </div>
          <p className="text-sm" style={{ color: "var(--color-text-muted)" }}>
            ダッシュボードは Epic C で実装予定
          </p>
        </header>

        <Card label="ACCOUNT" title="接続中">
          <p className="glass-card__body mt-2">
            資格情報は OS キーチェーンに安全に保存されています。
          </p>
          <div className="mt-6">
            <Button
              variant="danger"
              onClick={() => logout.mutate()}
              disabled={logout.isPending}
            >
              {logout.isPending ? "切断中…" : "ログアウト（資格情報を削除）"}
            </Button>
          </div>
        </Card>
      </main>
    </>
  );
}
