// Atollo marks, variant A of the brand board (brand.png). The SVGs come
// from docs/brand/make.py: edit the script, not the files.
import symbol from "./assets/brand/symbol.svg";
import wordmarkLight from "./assets/brand/wordmark-light.svg";
import wordmarkDark from "./assets/brand/wordmark-dark.svg";

/** The app icon: the N on its deep forest tile. */
export function BrandSymbol({ size }: { size: number }) {
  return <img data-tauri-drag-region src={symbol} alt="" width={size} height={size} draggable={false} />;
}

/** The name, in Deep Forest on light and Cloud on dark; `height` in px. */
export function BrandWordmark({ height, label = true }: { height: number; label?: boolean }) {
  // 664 x 100: the wordmark's viewBox.
  const width = Math.round((height * 664) / 100);
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
