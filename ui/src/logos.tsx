// Official logos of the components (Linguaggi e strumenti, Agenti) and of
// the languages found in a project, from Simple Icons (react-icons/si) with
// each brand's colour. A brand drawn in black or white takes the text colour,
// so it reads in both themes. Where Simple Icons has no logo (PowerShell,
// OpenAI) the closest icon of another set stands in;
// Antigravity's arch is drawn here. Also the frameworks of the wizard.
import { useId, type ReactNode } from "react";
import type { IconBaseProps, IconType } from "react-icons";
import { RiOpenaiFill } from "react-icons/ri";
import {
  SiAndroid,
  SiApache,
  SiAstro,
  SiBootstrap,
  SiBulma,
  SiBun,
  SiC,
  SiCaddy,
  SiCapacitor,
  SiClaude,
  SiCmake,
  SiComposer,
  SiCplusplus,
  SiCss,
  SiDart,
  SiDeno,
  SiDotnet,
  SiDrupal,
  SiElectron,
  SiFilament,
  SiFsharp,
  SiGit,
  SiGithub,
  SiGnubash,
  SiGo,
  SiHtml5,
  SiJavascript,
  SiJulia,
  SiKotlin,
  SiLaravel,
  SiLatex,
  SiLivewire,
  SiMongodb,
  SiMysql,
  SiNextdotjs,
  SiNodedotjs,
  SiOpenjdk,
  SiPhp,
  SiPhpmyadmin,
  SiPnpm,
  SiPrettier,
  SiPython,
  SiR,
  SiReact,
  SiRuby,
  SiRust,
  SiSqlite,
  SiStatamic,
  SiSvelte,
  SiSymfony,
  SiTailwindcss,
  SiTauri,
  SiTypescript,
  SiTypo3,
  SiTypst,
  SiUv,
  SiVite,
  SiVitest,
  SiVscodium,
  SiVuedotjs,
  SiWordpress,
  SiZig,
} from "react-icons/si";
import { VscTerminalPowershell } from "react-icons/vsc";
import { Mark, Tag } from "./tags";

/** Antigravity's logo: an arch in Google's colours, red at the top to blue
 * at the feet. */
function AntigravityIcon({ size = 16, className, style }: IconBaseProps) {
  const id = useId();
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden="true" className={className} style={style}>
      <defs>
        <linearGradient id={id} x1="0.8" y1="0" x2="0.2" y2="1">
          <stop offset="0" stopColor="#EA4335" />
          <stop offset="0.3" stopColor="#FBBC04" />
          <stop offset="0.5" stopColor="#34A853" />
          <stop offset="0.75" stopColor="#4285F4" />
        </linearGradient>
      </defs>
      <path
        d="M3 21.5C6.5 20.5 8 3.5 12 3.5S17.5 20.5 21 21.5"
        fill="none"
        stroke={`url(#${id})`}
        strokeWidth="3.6"
        strokeLinecap="round"
      />
    </svg>
  );
}

interface Logo {
  icon: IconType;
  /** Brand colour; none = the text colour (black or white brands). */
  color?: string;
}

const PHP: Logo = { icon: SiPhp, color: "#777BB4" };
const PYTHON: Logo = { icon: SiPython, color: "#3776AB" };
const RUST: Logo = { icon: SiRust };
const GO: Logo = { icon: SiGo, color: "#00ADD8" };
const JAVA: Logo = { icon: SiOpenjdk };
const KOTLIN: Logo = { icon: SiKotlin, color: "#7F52FF" };
const DOTNET: Logo = { icon: SiDotnet, color: "#512BD4" };
const DART: Logo = { icon: SiDart, color: "#0175C2" };
const ZIG: Logo = { icon: SiZig, color: "#F7A41D" };
const JULIA: Logo = { icon: SiJulia, color: "#9558B2" };
const RUBY: Logo = { icon: SiRuby, color: "#CC342D" };
const R: Logo = { icon: SiR, color: "#276DC3" };
const C: Logo = { icon: SiC, color: "#A8B9CC" };

const COMPONENTS: Record<string, Logo> = {
  gh: { icon: SiGithub },
  pwsh: { icon: VscTerminalPowershell, color: "#5391FE" },
  "vscodium-server": { icon: SiVscodium, color: "#2F80ED" },
  claude: { icon: SiClaude, color: "#D97757" },
  codex: { icon: RiOpenaiFill },
  antigravity: { icon: AntigravityIcon },
  git: { icon: SiGit, color: "#F05032" },
  python: PYTHON,
  node: { icon: SiNodedotjs, color: "#5FA04E" },
  uv: { icon: SiUv, color: "#DE5FE9" },
  php: PHP,
  composer: { icon: SiComposer, color: "#885630" },
  apache: { icon: SiApache, color: "#D22128" },
  caddy: { icon: SiCaddy, color: "#1F88C0" },
  rust: RUST,
  go: GO,
  java: JAVA,
  kotlin: KOTLIN,
  dotnet: DOTNET,
  android: { icon: SiAndroid, color: "#3DDC84" },
  dart: DART,
  zig: ZIG,
  julia: JULIA,
  bun: { icon: SiBun },
  deno: { icon: SiDeno },
  typst: { icon: SiTypst, color: "#239DAD" },
  tinytex: { icon: SiLatex, color: "#008080" },
  pnpm: { icon: SiPnpm, color: "#F69220" },
  cmake: { icon: SiCmake, color: "#064F8C" },
  cc: C,
  ruby: RUBY,
  mongodb: { icon: SiMongodb, color: "#47A248" },
  phpmyadmin: { icon: SiPhpmyadmin, color: "#6C78AF" },
  // SQLite's #003B57 is unreadable on dark: the light blue of its logo.
  sqlite: { icon: SiSqlite, color: "#0F80CC" },
  r: R,
};

/** Languages as named by tauri/src/detect/mod.rs `LANGUAGES`. */
const LANGUAGES: Record<string, Logo> = {
  JavaScript: { icon: SiJavascript, color: "#F7DF1E" },
  TypeScript: { icon: SiTypescript, color: "#3178C6" },
  Vue: { icon: SiVuedotjs, color: "#4FC08D" },
  Svelte: { icon: SiSvelte, color: "#FF3E00" },
  Astro: { icon: SiAstro, color: "#BC52EE" },
  PHP: PHP,
  Python: PYTHON,
  Rust: RUST,
  Go: GO,
  HTML: { icon: SiHtml5, color: "#E34F26" },
  CSS: { icon: SiCss, color: "#663399" },
  Java: JAVA,
  Kotlin: KOTLIN,
  "C#": DOTNET,
  "F#": { icon: SiFsharp, color: "#378BBA" },
  Ruby: RUBY,
  Dart: DART,
  Zig: ZIG,
  Julia: JULIA,
  R: R,
  C: C,
  "C++": { icon: SiCplusplus, color: "#00599C" },
  LaTeX: { icon: SiLatex, color: "#008080" },
  Typst: { icon: SiTypst, color: "#239DAD" },
  PowerShell: { icon: VscTerminalPowershell, color: "#5391FE" },
  Shell: { icon: SiGnubash, color: "#4EAA25" },
};

/** The frameworks of the new-project wizard, by id (catalog/frameworks.json). */
const FRAMEWORKS: Record<string, Logo> = {
  laravel: { icon: SiLaravel, color: "#FF2D20" },
  symfony: { icon: SiSymfony },
  vite: { icon: SiVite, color: "#646CFF" },
  "vite-bundle": { icon: SiVite, color: "#646CFF" },
  nextjs: { icon: SiNextdotjs },
  electron: { icon: SiElectron, color: "#47848F" },
  tauri: { icon: SiTauri, color: "#24C8D8" },
  android: { icon: SiAndroid, color: "#3DDC84" },
  wordpress: { icon: SiWordpress, color: "#21759B" },
  typo3: { icon: SiTypo3, color: "#FF8700" },
  drupal: { icon: SiDrupal, color: "#0678BE" },
  statamic: { icon: SiStatamic, color: "#FF269E" },
  react: { icon: SiReact, color: "#61DAFB" },
  vue: { icon: SiVuedotjs, color: "#4FC08D" },
  svelte: { icon: SiSvelte, color: "#FF3E00" },
  livewire: { icon: SiLivewire, color: "#FB70A9" },
  tailwind: { icon: SiTailwindcss, color: "#06B6D4" },
  bootstrap: { icon: SiBootstrap, color: "#7952B3" },
  bulma: { icon: SiBulma, color: "#00D1B2" },
  sqlite: { icon: SiSqlite, color: "#0F80CC" },
  mysql: { icon: SiMysql, color: "#4479A1" },
  mongodb: { icon: SiMongodb, color: "#47A248" },
  nativephp: PHP,
  "nativephp-mobile": { icon: SiAndroid, color: "#3DDC84" },
  capacitor: { icon: SiCapacitor, color: "#119EFF" },
  filament: { icon: SiFilament, color: "#FDAE4B" },
  vitest: { icon: SiVitest, color: "#6E9F18" },
  prettier: { icon: SiPrettier, color: "#F7B93E" },
};

function LogoIcon({ logo, size }: { logo: Logo | undefined; size: number }) {
  if (!logo) return null;
  const Icon = logo.icon;
  return (
    <Icon size={size} aria-hidden="true" className="shrink-0" style={logo.color ? { color: logo.color } : undefined} />
  );
}

/** The logo of a component of the catalog (nothing if it has none). */
export function ComponentLogo({ id, size = 16 }: { id: string; size?: number }) {
  return <LogoIcon logo={COMPONENTS[id]} size={size} />;
}

/** A label with the logo of a language found in a project. */
export function LanguageTag({ name }: { name: string }) {
  return (
    <Tag icon={<LogoIcon logo={LANGUAGES[name]} size={11} />} className="text-vsc-fg">
      {name}
    </Tag>
  );
}

export const hasLogo = (id: string): boolean => id in COMPONENTS;

/** The logo of a framework of the wizard, or `fallback` when it has none. */
export function FrameworkLogo({ id, size = 16, fallback }: { id: string; size?: number; fallback?: ReactNode }) {
  const logo = FRAMEWORKS[id];
  return logo ? <LogoIcon logo={logo} size={size} /> : <>{fallback}</>;
}
/** The logo of a language alone (nothing if it has none). */
export function LanguageLogo({ name, size = 14 }: { name: string; size?: number }) {
  return <LogoIcon logo={LANGUAGES[name]} size={size} />;
}

export const hasLanguageLogo = (name: string): boolean => name in LANGUAGES;

/** A language as its logo only, named on hover; the label when it has none. */
export function LanguageIcon({ name }: { name: string }) {
  if (!hasLanguageLogo(name)) return <LanguageTag name={name} />;
  return (
    <Mark label={name}>
      <LogoIcon logo={LANGUAGES[name]} size={14} />
    </Mark>
  );
}
