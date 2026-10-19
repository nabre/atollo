// Atollo marks, variant A of the brand board (brand.png). The SVGs come
// from docs/brand/make.py: edit the script, not the files.
import symbol from "./assets/brand/symbol.svg";
import wordmarkLight from "./assets/brand/wordmark-light.svg";
import wordmarkDark from "./assets/brand/wordmark-dark.svg";

/** The app icon: the A on its deep forest tile. */
export function BrandSymbol({ size }: { size: number }) {
  return <img data-tauri-drag-region src={symbol} alt="" width={size} height={size} draggable={false} />;
}

/** The name, in Deep Forest on light and Cloud on dark, its last O the
 * symbol with the waves below the baseline: never next to BrandSymbol.
 * `height` in px, waves included (the capitals take 100/129 of it). */
export function BrandWordmark({ height, label = true }: { height: number; label?: boolean }) {
  // 514 x 129: the wordmark's viewBox.
  const width = Math.round((height * 514) / 129);
  const alt = label ? "Atollo" : "";
  return (
    <>
      <img
        data-tauri-drag-region
        className="only-light"
        src={wordmarkLight}
        alt={alt}
        width={width}
        height={height}
        draggable={false}
      />
      <img
        data-tauri-drag-region
        className="only-dark"
        src={wordmarkDark}
        alt={alt}
        width={width}
        height={height}
        draggable={false}
      />
    </>
  );
}
