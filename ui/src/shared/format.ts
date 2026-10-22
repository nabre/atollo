// Sizes as text, in the interface's language: one place for the units the
// pages show, so the same size reads the same everywhere.
import { locale } from "../i18n";

const number = (n: number, digits: number, min = 0) =>
  new Intl.NumberFormat(locale(), { minimumFractionDigits: min, maximumFractionDigits: digits }).format(n);

/** A file or a copy: KB (at least 1) up to 1024 KB, then MB with one decimal. */
export function formatBytes(bytes: number): string {
  const kb = bytes / 1024;
  return kb < 1024 ? `${number(Math.max(1, kb), 0)} KB` : `${number(kb / 1024, 1)} MB`;
}

/** A large archive (the kit's copy): MB (at least 1) up to 1024 MB, then GB with one decimal. */
export function formatLargeBytes(bytes: number): string {
  const mb = bytes / 1024 / 1024;
  return mb < 1024 ? `${number(Math.max(1, mb), 0)} MB` : `${number(mb / 1024, 1)} GB`;
}

/** Whole MB, rounded up, without the unit. */
export function megabytes(bytes: number): string {
  return number(Math.ceil(bytes / 1024 / 1024), 0);
}

/** Bytes as GB (10⁹, as model files are sized) with one decimal. */
export function gb(bytes: number): string {
  return `${number(bytes / 1e9, 1, 1)} GB`;
}

/** A download's MB from the catalog: MB below 1000, then GB (10³ MB) with one decimal. */
export function sizeText(mb: number): string {
  if (mb < 1000) return `${String(mb)} MB`;
  return `${number(mb / 1000, 1, 1)} GB`;
}
