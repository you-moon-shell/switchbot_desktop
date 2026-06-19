/**
 * 背景に固定配置する、ゆっくり漂うグラデーションのブロブ。
 * ガラス面の向こうに色が透けるための装飾レイヤー（操作不可・装飾のみ）。
 */
export function GlassBackground() {
  return (
    <div className="scene" aria-hidden="true">
      <div className="scene__blob scene__blob--1" />
      <div className="scene__blob scene__blob--2" />
      <div className="scene__blob scene__blob--3" />
    </div>
  );
}
