type ClassValue = string | number | false | null | undefined;

/**
 * クラス名を結合する小さなヘルパ（falsy は除外）。
 * shadcn の `cn` 相当だが、tailwind-merge は使わず依存ゼロで軽量に保つ。
 */
export function cn(...classes: ClassValue[]): string {
  return classes.filter(Boolean).join(" ");
}
