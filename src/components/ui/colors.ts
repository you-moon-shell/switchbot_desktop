/** Badge / Button のカラー指定。色名でも状態でも渡せ、状態は色名へ正規化する。 */

/** パレットの色名（直接の見た目）。 */
export type ColorName = "aqua" | "violet" | "rose" | "amber" | "lime" | "red";

/** 意味（状態）。色名へ正規化される。 */
export type ColorState =
  | "success" // → lime
  | "warning" // → amber
  | "danger" // → red
  | "info" // → aqua
  | "neutral"; // → neutral（グレー）

export type Color = ColorName | ColorState;

/** 状態名・色名を CSS クラスのサフィックス（パレット色名 / neutral）へ正規化する。 */
export function resolveColor(color: Color): string {
  switch (color) {
    case "success":
      return "lime";
    case "warning":
      return "amber";
    case "danger":
      return "red";
    case "info":
      return "aqua";
    default:
      return color; // aqua/violet/rose/amber/lime/red/neutral はそのまま
  }
}
