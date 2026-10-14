// Official logos of the components (Linguaggi e strumenti, Agenti) and of
// the languages found in a project, from Simple Icons (react-icons/si) with
// each brand's colour. A brand drawn in black or white takes the text colour,
// so it reads in both themes. Where Simple Icons has no logo (PowerShell,
// OpenAI, Antigravity) the closest icon of another set stands in.
import type { IconType } from "react-icons";
import { RiOpenaiFill } from "react-icons/ri";
import {
  SiApache,
  SiAstro,
  SiBun,
  SiC,
  SiClaude,
  SiCmake,
  SiCplusplus,
  SiCss,
  SiDart,
  SiDeno,
  SiDotnet,
  SiFsharp,
  SiGit,
  SiGithub,
  SiGo,
  SiHtml5,
  SiJavascript,
  SiJulia,
  SiKotlin,
  SiLatex,
  SiNodedotjs,
  SiOpenjdk,
  SiPhp,
  SiPnpm,
  SiPython,
  SiR,
  SiRuby,
  SiRust,
  SiSvelte,
  SiTypescript,
  SiTypst,
  SiUv,
  SiVscodium,
  SiVuedotjs,
  SiZig,
} from "react-icons/si";
import { VscSparkle, VscTerminalPowershell } from "react-icons/vsc";

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
  antigravity: { icon: VscSparkle, color: "#4285F4" },
  git: { icon: SiGit, color: "#F05032" },
  python: PYTHON,
  node: { icon: SiNodedotjs, color: "#5FA04E" },
  uv: { icon: SiUv, color: "#DE5FE9" },
  php: PHP,
  apache: { icon: SiApache, color: "#D22128" },
  rust: RUST,
  go: GO,
  java: JAVA,
  kotlin: KOTLIN,
  dotnet: DOTNET,
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
  r: R,
};

/** Languages as named by src-tauri/src/detect/mod.rs `language_of`. */
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
    <span className="inline-flex items-center gap-1 rounded-sm border border-vsc-border px-1.5 py-px text-[11px] leading-4">
      <LogoIcon logo={LANGUAGES[name]} size={11} />
      {name}
    </span>
  );
}

export const hasLogo = (id: string): boolean => id in COMPONENTS;
export const hasLanguageLogo = (name: string): boolean => name in LANGUAGES;
