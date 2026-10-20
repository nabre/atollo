// Reference catalogs of the Guide: the external sources of every catalog
// component and part (verified by test/ui/guide/sources.test.ts).
import { tr } from "./model";

export type SourceCategory = "required" | "app" | "agents" | "languages" | "tools" | "databases" | "part";

export interface ToolSourceRow {
  id: string;
  logoId: string;
  name: string;
  group: SourceCategory;
  home: string;
  metaSource: string;
  binarySource: string;
  verifyKind: "SHA-256" | "SHA-512" | "SHA3-256" | "npm sha512" | "Authenticode + SHA";
  verifyDetail: string;
  targetPath: string;
  /** Where macOS and Linux get it when not the same source as Windows
   * (the other columns describe Windows); null when only the file names
   * change. */
  unix: string | null;
}

type LazyStr = string | (() => string);
function evalStr(v: LazyStr): string {
  return typeof v === "function" ? v() : v;
}
function defSource(r: {
  id: string;
  logoId: string;
  name: LazyStr;
  group: SourceCategory;
  home: string;
  metaSource: LazyStr;
  binarySource: LazyStr;
  verifyKind: ToolSourceRow["verifyKind"];
  verifyDetail: LazyStr;
  targetPath: LazyStr;
  unix?: LazyStr;
}): ToolSourceRow {
  return {
    id: r.id,
    logoId: r.logoId,
    get name() {
      return evalStr(r.name);
    },
    group: r.group,
    home: r.home,
    get metaSource() {
      return evalStr(r.metaSource);
    },
    get binarySource() {
      return evalStr(r.binarySource);
    },
    verifyKind: r.verifyKind,
    get verifyDetail() {
      return evalStr(r.verifyDetail);
    },
    get targetPath() {
      return evalStr(r.targetPath);
    },
    get unix() {
      return r.unix === undefined ? null : evalStr(r.unix);
    },
  };
}

/** The rows of the sources table: the catalog's components and their
 * parts (Composer, Maven...), counted from the table itself. */
export function sourceCounts(): { components: number; parts: number; all: number } {
  const parts = TOOL_SOURCES.filter((r) => r.group === "part").length;
  return { components: TOOL_SOURCES.length - parts, parts, all: TOOL_SOURCES.length };
}

const lLine = () => tr("<linea>", "<line>", "<ligne>", "<Linie>");

export const TOOL_SOURCES: ToolSourceRow[] = [
  defSource({
    id: "pwsh",
    logoId: "pwsh",
    name: "PowerShell 7 (pwsh)",
    group: "required",
    home: "https://github.com/PowerShell/PowerShell",
    metaSource: "api.github.com/repos/PowerShell/PowerShell/releases/latest",
    binarySource: "PowerShell-<v>-win-(x64|arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () => `pwsh\\${lLine()}\\pwsh.exe`,
  }),
  defSource({
    id: "gh",
    logoId: "gh",
    name: "GitHub CLI (gh)",
    group: "required",
    home: "https://cli.github.com",
    metaSource: "api.github.com/repos/cli/cli/releases/latest",
    binarySource: "gh_<v>_windows_(amd64|arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () => `gh\\${lLine()}\\bin\\gh.exe`,
  }),
  defSource({
    id: "vscodium-server",
    logoId: "vscodium-server",
    name: "Editor (VSCodium Reh Web)",
    group: "app",
    home: "https://vscodium.com",
    metaSource: () =>
      tr(
        "Catalogo integrato (versione fissata 1.135.06055, D-58)",
        "Built-in catalog (pinned version 1.135.06055, D-58)",
        "Catalogue intégré (version figée 1.135.06055, D-58)",
        "Integrierter Katalog (fixierte Version 1.135.06055, D-58)",
      ),
    binarySource: "github.com/VSCodium/vscodium/releases/download/<v>/vscodium-reh-web-win32-x64-<v>.tar.gz",
    unix: () =>
      tr(
        "macOS e Linux: vscodium-reh-web-<target>-<v>.tar.gz, SHA-256 fissato per target (nativo anche su ARM64)",
        "macOS and Linux: vscodium-reh-web-<target>-<v>.tar.gz, SHA-256 pinned per target (native on ARM64 too)",
        "macOS et Linux : vscodium-reh-web-<target>-<v>.tar.gz, SHA-256 fixé par cible (natif aussi sur ARM64)",
        "macOS und Linux: vscodium-reh-web-<target>-<v>.tar.gz, SHA-256 pro Ziel festgelegt (auch auf ARM64 nativ)",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 cablato in components.json",
        "SHA-256 hardcoded in components.json",
        "SHA-256 intégré dans components.json",
        "SHA-256 fest in components.json hinterlegt",
      ),
    targetPath: "vscodium-server\\",
  }),
  defSource({
    id: "claude",
    logoId: "claude",
    name: "Claude Code",
    group: "agents",
    home: "https://claude.com/product/claude-code",
    metaSource: "registry.npmjs.org/@anthropic-ai/claude-code-win32-<arch>/latest",
    binarySource: () =>
      tr(
        "Campo dist.tarball del pacchetto npm",
        "dist.tarball field of the npm package",
        "Champ dist.tarball du paquet npm",
        "Feld dist.tarball des npm-Pakets",
      ),
    verifyKind: "Authenticode + SHA",
    verifyDetail: () =>
      tr(
        "Integrità npm sha512 + firma Authenticode Anthropic",
        "npm sha512 integrity + Anthropic Authenticode signature",
        "Intégrité npm sha512 + signature Authenticode Anthropic",
        "npm-sha512-Integrität + Authenticode-Signatur von Anthropic",
      ),
    targetPath: () => `claude\\${lLine()}\\claude.exe`,
  }),
  defSource({
    id: "codex",
    logoId: "codex",
    name: "Codex (OpenAI)",
    group: "agents",
    home: "https://github.com/openai/codex",
    metaSource: "registry.npmjs.org/@openai/codex/latest → /<ver>-win32-<arch>",
    binarySource: () =>
      tr(
        "Campo dist.tarball del pacchetto specifico per architettura",
        "dist.tarball field of the architecture-specific package",
        "Champ dist.tarball du paquet spécifique à l'architecture",
        "Feld dist.tarball des architekturspezifischen Pakets",
      ),
    verifyKind: "npm sha512",
    verifyDetail: () =>
      tr(
        "Integrità npm dist.integrity (sha512-...)",
        "npm dist.integrity (sha512-...)",
        "Intégrité npm dist.integrity (sha512-...)",
        "npm-Integrität dist.integrity (sha512-...)",
      ),
    targetPath: () => `codex\\${lLine()}\\bin\\codex.exe`,
  }),
  defSource({
    id: "antigravity",
    logoId: "antigravity",
    name: "Antigravity (Google)",
    group: "agents",
    home: "https://antigravity.google",
    metaSource: "antigravity-cli-auto-updater-974169037036.us-central1.run.app/manifests/windows_(amd64|arm64).json",
    unix: () =>
      tr(
        "macOS e Linux: manifests/<os>_<arch>.json, archivio .tar.gz",
        "macOS and Linux: manifests/<os>_<arch>.json, .tar.gz archive",
        "macOS et Linux : manifests/<os>_<arch>.json, archive .tar.gz",
        "macOS und Linux: manifests/<os>_<arch>.json, .tar.gz-Archiv",
      ),
    binarySource: () =>
      tr(
        "Campo url del manifesto JSON ufficiale",
        "url field of the official JSON manifest",
        "Champ url du manifeste JSON officiel",
        "Feld url des offiziellen JSON-Manifests",
      ),
    verifyKind: "SHA-512",
    verifyDetail: () =>
      tr(
        "SHA-512 (128 hex) dal campo sha512 del manifesto",
        "SHA-512 (128 hex) from the manifest sha512 field",
        "SHA-512 (128 hex) depuis le champ sha512 du manifeste",
        "SHA-512 (128 Hex) aus dem Feld sha512 des Manifests",
      ),
    targetPath: () => `antigravity\\${lLine()}\\agy.exe (via data\\kit\\bin\\agy.cmd)`,
  }),
  defSource({
    id: "git",
    logoId: "git",
    name: "Git + Git Bash (PortableGit)",
    group: "tools",
    home: "https://git-scm.com",
    metaSource: "api.github.com/repos/git-for-windows/git/releases/latest",
    unix: () =>
      tr(
        "macOS e Linux: GitHub desktop/dugite-native (dugite-native-v<v>-<sha>-<sistema>.tar.gz, .sha256 dell'asset), segue il kit",
        "macOS and Linux: GitHub desktop/dugite-native (dugite-native-v<v>-<sha>-<system>.tar.gz, the asset's .sha256), follows the kit",
        "macOS et Linux : GitHub desktop/dugite-native (dugite-native-v<v>-<sha>-<système>.tar.gz, .sha256 de l'asset), suit le kit",
        "macOS und Linux: GitHub desktop/dugite-native (dugite-native-v<v>-<sha>-<System>.tar.gz, .sha256 des Assets), zieht mit dem Kit um",
      ),
    binarySource: () =>
      tr(
        "PortableGit-<v>-(64-bit|arm64).7z.exe (estratto in Rust)",
        "PortableGit-<v>-(64-bit|arm64).7z.exe (extracted in Rust)",
        "PortableGit-<v>-(64-bit|arm64).7z.exe (extrait en Rust)",
        "PortableGit-<v>-(64-bit|arm64).7z.exe (in Rust entpackt)",
      ),
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () =>
      tr(
        `git\\${lLine()}\\cmd\\git.exe e git\\${lLine()}\\bin\\bash.exe`,
        `git\\${lLine()}\\cmd\\git.exe and git\\${lLine()}\\bin\\bash.exe`,
        `git\\${lLine()}\\cmd\\git.exe et git\\${lLine()}\\bin\\bash.exe`,
        `git\\${lLine()}\\cmd\\git.exe und git\\${lLine()}\\bin\\bash.exe`,
      ),
  }),
  defSource({
    id: "python",
    logoId: "python",
    name: "Python + pip",
    group: "languages",
    home: "https://www.python.org",
    metaSource: "raw.githubusercontent.com/astral-sh/python-build-standalone/latest-release/latest-release.json",
    binarySource: "cpython-3.<minor>.<patch>+<date>-<triple>-install_only_stripped.tar.gz",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dall'elenco ufficiale SHA256SUMS",
        "SHA-256 from the official SHA256SUMS list",
        "SHA-256 depuis la liste officielle SHA256SUMS",
        "SHA-256 aus der offiziellen SHA256SUMS-Liste",
      ),
    targetPath: () => `python\\${lLine()}\\python.exe (+ pip)`,
  }),
  defSource({
    id: "node",
    logoId: "node",
    name: "Node.js + npm",
    group: "languages",
    home: "https://nodejs.org",
    metaSource: () =>
      tr(
        "nodejs.org/dist/index.json (LTS pari >= 18 + ultima major)",
        "nodejs.org/dist/index.json (even LTS >= 18 + latest major)",
        "nodejs.org/dist/index.json (LTS paires >= 18 + dernière majeure)",
        "nodejs.org/dist/index.json (gerade LTS >= 18 + neueste Major)",
      ),
    binarySource: "nodejs.org/dist/<v>/node-<v>-win-<arch>.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da nodejs.org/dist/<v>/SHASUMS256.txt",
        "SHA-256 from nodejs.org/dist/<v>/SHASUMS256.txt",
        "SHA-256 depuis nodejs.org/dist/<v>/SHASUMS256.txt",
        "SHA-256 aus nodejs.org/dist/<v>/SHASUMS256.txt",
      ),
    targetPath: () => `node\\${lLine()}\\node.exe`,
  }),
  defSource({
    id: "uv",
    logoId: "uv",
    name: "uv (uv, uvx, uvw)",
    group: "tools",
    home: "https://docs.astral.sh/uv/",
    metaSource: "api.github.com/repos/astral-sh/uv/releases/latest",
    binarySource: "uv-<triple>.zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () => `uv\\${lLine()}\\uv.exe, uvx.exe, uvw.exe`,
  }),
  defSource({
    id: "php",
    logoId: "php",
    name: "PHP (Thread Safe)",
    group: "languages",
    home: "https://www.php.net",
    metaSource: () =>
      tr(
        "windows.php.net/downloads/releases/releases.json (+ archivi EOL 8.1..5.6)",
        "windows.php.net/downloads/releases/releases.json (+ EOL archives 8.1..5.6)",
        "windows.php.net/downloads/releases/releases.json (+ archives EOL 8.1..5.6)",
        "windows.php.net/downloads/releases/releases.json (+ EOL-Archive 8.1..5.6)",
      ),
    binarySource: "windows.php.net/downloads/releases/php-<v>-Win32-vs*-x64.zip",
    unix: () =>
      tr(
        "macOS e Linux: static-php-cli «bulk», archivi cli e fpm con SHA-256 fissato nel catalogo (8.4 e 8.5)",
        "macOS and Linux: static-php-cli “bulk”, cli and fpm archives with SHA-256 pinned in the catalog (8.4 and 8.5)",
        "macOS et Linux : static-php-cli « bulk », archives cli et fpm avec SHA-256 fixé dans le catalogue (8.4 et 8.5)",
        "macOS und Linux: static-php-cli „bulk“, cli- und fpm-Archive mit im Katalog festgelegtem SHA-256 (8.4 und 8.5)",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da releases.json (o tabella EOL verificata)",
        "SHA-256 from releases.json (or verified EOL table)",
        "SHA-256 depuis releases.json (ou table EOL vérifiée)",
        "SHA-256 aus releases.json (oder geprüfter EOL-Tabelle)",
      ),
    targetPath: () => `php\\${lLine()}\\php.exe`,
  }),
  defSource({
    id: "composer",
    logoId: "php",
    name: () => tr("↳ Parte: Composer", "↳ Part: Composer", "↳ Partie : Composer", "↳ Teil: Composer"),
    group: "part",
    home: "https://getcomposer.org",
    metaSource: () =>
      tr(
        "getcomposer.org/versions (canale stable o 2.2 LTS per PHP < 7.2.5)",
        "getcomposer.org/versions (stable channel or 2.2 LTS for PHP < 7.2.5)",
        "getcomposer.org/versions (canal stable ou 2.2 LTS pour PHP < 7.2.5)",
        "getcomposer.org/versions (Stable-Kanal oder 2.2 LTS für PHP < 7.2.5)",
      ),
    binarySource: "getcomposer.org/download/<v>/composer.phar",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da <url>.sha256sum",
        "SHA-256 from <url>.sha256sum",
        "SHA-256 depuis <url>.sha256sum",
        "SHA-256 aus <url>.sha256sum",
      ),
    targetPath: () => `php\\${lLine()}\\composer.phar (+ composer.cmd)`,
  }),
  defSource({
    id: "cacert",
    logoId: "php",
    name: () =>
      tr(
        "↳ Parte: Bundle CA SSL (curl)",
        "↳ Part: SSL CA Bundle (curl)",
        "↳ Partie : Bundle CA SSL (curl)",
        "↳ Teil: SSL-CA-Bundle (curl)",
      ),
    group: "part",
    home: "https://curl.se",
    metaSource: "curl.se/ca/cacert.pem.sha256",
    binarySource: "curl.se/ca/cacert.pem",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da cacert.pem.sha256",
        "SHA-256 from cacert.pem.sha256",
        "SHA-256 depuis cacert.pem.sha256",
        "SHA-256 aus cacert.pem.sha256",
      ),
    targetPath: () => `php\\${lLine()}\\extras\\ssl\\cacert.pem`,
  }),
  defSource({
    id: "vcruntime",
    logoId: "cc",
    name: () =>
      tr(
        "↳ Parte: Visual C++ Runtime",
        "↳ Part: Visual C++ Runtime",
        "↳ Partie : Visual C++ Runtime",
        "↳ Teil: Visual C++ Runtime",
      ),
    group: "part",
    home: "https://learn.microsoft.com",
    metaSource: () =>
      tr(
        "Estratto dal CAB vcRuntimeMinimum_amd64\\cab1.cab se assente",
        "Extracted from CAB vcRuntimeMinimum_amd64\\cab1.cab if missing",
        "Extrait du CAB vcRuntimeMinimum_amd64\\cab1.cab si absent",
        "Bei Fehlen aus CAB vcRuntimeMinimum_amd64\\cab1.cab entpackt",
      ),
    binarySource: "aka.ms/vc14/vc_redist.x64.exe",
    unix: () => tr("Solo Windows", "Windows only", "Windows uniquement", "Nur Windows"),
    verifyKind: "Authenticode + SHA",
    verifyDetail: () =>
      tr(
        "Firma Authenticode O=Microsoft Corporation su exe e vcruntime140.dll",
        "Authenticode signature O=Microsoft Corporation on exe and vcruntime140.dll",
        "Signature Authenticode O=Microsoft Corporation sur exe et vcruntime140.dll",
        "Authenticode-Signatur O=Microsoft Corporation auf exe und vcruntime140.dll",
      ),
    targetPath: () =>
      tr(
        "tools\\vcruntime\\ (copiato in PHP, Apache, MongoDB)",
        "tools\\vcruntime\\ (copied into PHP, Apache, MongoDB)",
        "tools\\vcruntime\\ (copié dans PHP, Apache, MongoDB)",
        "tools\\vcruntime\\ (kopiert nach PHP, Apache, MongoDB)",
      ),
  }),
  defSource({
    id: "apache",
    logoId: "apache",
    name: "Apache httpd",
    group: "languages",
    home: "https://httpd.apache.org",
    metaSource: () =>
      tr(
        "apachelounge.com/download/ (scraping versione e checksum)",
        "apachelounge.com/download/ (version and checksum scraping)",
        "apachelounge.com/download/ (extraction de version et somme de contrôle)",
        "apachelounge.com/download/ (Auslesen von Version und Prüfsumme)",
      ),
    binarySource: "apachelounge.com/download/VS<nn>/binaries/httpd-<v>-Win64-VS<nn>.zip",
    unix: () =>
      tr(
        "macOS e Linux: non ancora offerto; l'installatore della build del kit (httpd-v<v>, php-fpm) attende il primo rilascio fissato nel catalogo (D-116)",
        "macOS and Linux: not offered yet; the installer of the kit's build (httpd-v<v>, php-fpm) waits for the first release pinned in the catalog (D-116)",
        "macOS et Linux : pas encore proposé ; l'installateur du build du kit (httpd-v<v>, php-fpm) attend la première version fixée dans le catalogue (D-116)",
        "macOS und Linux: noch nicht angeboten; der Installer des Kit-Builds (httpd-v<v>, php-fpm) wartet auf das erste im Katalog festgelegte Release (D-116)",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da <url>.txt (SHA256-Checksum for:)",
        "SHA-256 from <url>.txt (SHA256-Checksum for:)",
        "SHA-256 depuis <url>.txt (SHA256-Checksum for:)",
        "SHA-256 aus <url>.txt (SHA256-Checksum for:)",
      ),
    targetPath: () => `apache\\${lLine()}\\bin\\httpd.exe`,
  }),
  defSource({
    id: "caddy",
    logoId: "caddy",
    name: "Caddy",
    group: "languages",
    home: "https://caddyserver.com",
    metaSource: "api.github.com/repos/caddyserver/caddy/releases?per_page=100",
    binarySource: () =>
      tr(
        "Asset caddy_<v>_windows_amd64.zip dalla release GitHub ufficiale",
        "caddy_<v>_windows_amd64.zip asset from the official GitHub release",
        "Asset caddy_<v>_windows_amd64.zip depuis la release GitHub officielle",
        "Asset caddy_<v>_windows_amd64.zip aus dem offiziellen GitHub-Release",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "GitHub Release asset.digest (dalla 2.10.1), altrimenti SHA-512 da caddy_<v>_checksums.txt",
        "GitHub Release asset.digest (from 2.10.1), otherwise SHA-512 from caddy_<v>_checksums.txt",
        "GitHub Release asset.digest (depuis 2.10.1), sinon SHA-512 depuis caddy_<v>_checksums.txt",
        "GitHub Release asset.digest (ab 2.10.1), sonst SHA-512 aus caddy_<v>_checksums.txt",
      ),
    targetPath: () => `caddy\\${lLine()}\\caddy.exe`,
  }),
  defSource({
    id: "rust",
    logoId: "rust",
    name: "Rust (rustup, cargo, clippy)",
    group: "tools",
    home: "https://www.rust-lang.org",
    metaSource: "static.rust-lang.org/dist/channel-rust-stable.toml ([pkg.rustc])",
    binarySource: "static.rust-lang.org/rustup/dist/<triple>/rustup-init.exe",
    unix: () =>
      tr(
        "Host gnu su Windows x64, gnullvm su Windows ARM64 (con C/C++), msvc dalla pagina di Rust in Strumenti; macOS e Linux: rustup con l'host nativo",
        "Host gnu on Windows x64, gnullvm on Windows ARM64 (with C/C++), msvc from Rust's page in Tools; macOS and Linux: rustup with the native host",
        "Hôte gnu sur Windows x64, gnullvm sur Windows ARM64 (avec C/C++), msvc depuis la page de Rust dans Outils ; macOS et Linux : rustup avec l'hôte natif",
        "Host gnu auf Windows x64, gnullvm auf Windows ARM64 (mit C/C++), msvc über die Seite von Rust unter Werkzeuge; macOS und Linux: rustup mit nativem Host",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da <url>.sha256",
        "SHA-256 from <url>.sha256",
        "SHA-256 depuis <url>.sha256",
        "SHA-256 aus <url>.sha256",
      ),
    targetPath: () =>
      tr(
        "rust\\cargo\\bin\\rustc.exe e rustup.exe",
        "rust\\cargo\\bin\\rustc.exe and rustup.exe",
        "rust\\cargo\\bin\\rustc.exe et rustup.exe",
        "rust\\cargo\\bin\\rustc.exe und rustup.exe",
      ),
  }),
  defSource({
    id: "go",
    logoId: "go",
    name: "Go SDK",
    group: "languages",
    home: "https://go.dev",
    metaSource: "go.dev/dl/?mode=json (&include=all)",
    binarySource: "go.dev/dl/go<v>.windows-(amd64|arm64).zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dal campo sha256 del JSON ufficiale",
        "SHA-256 from the sha256 field of the official JSON",
        "SHA-256 depuis le champ sha256 du JSON officiel",
        "SHA-256 aus dem Feld sha256 des offiziellen JSON",
      ),
    targetPath: () => `go\\${lLine()}\\bin\\go.exe`,
  }),
  defSource({
    id: "java",
    logoId: "java",
    name: "Java JDK (Eclipse Temurin)",
    group: "languages",
    home: "https://adoptium.net",
    metaSource: "api.adoptium.net/v3/info/available_releases + /v3/assets/latest/<major>/hotspot",
    binarySource: () =>
      tr(
        "Pacchetto .zip restituito dall'API Adoptium",
        ".zip package returned by the Adoptium API",
        "Paquet .zip renvoyé par l'API Adoptium",
        ".zip-Paket aus der Adoptium-API",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da binary.package.checksum",
        "SHA-256 from binary.package.checksum",
        "SHA-256 depuis binary.package.checksum",
        "SHA-256 aus binary.package.checksum",
      ),
    targetPath: () => `java\\${lLine()}\\bin\\java.exe`,
  }),
  defSource({
    id: "maven",
    logoId: "java",
    name: () => tr("↳ Parte: Apache Maven", "↳ Part: Apache Maven", "↳ Partie : Apache Maven", "↳ Teil: Apache Maven"),
    group: "part",
    home: "https://maven.apache.org",
    metaSource: "repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml",
    binarySource: "apache-maven-<v>-bin.zip",
    verifyKind: "SHA-512",
    verifyDetail: () =>
      tr(
        "SHA-512 da <url>.sha512",
        "SHA-512 from <url>.sha512",
        "SHA-512 depuis <url>.sha512",
        "SHA-512 aus <url>.sha512",
      ),
    targetPath: "maven\\bin\\mvn.cmd",
  }),
  defSource({
    id: "gradle",
    logoId: "java",
    name: () => tr("↳ Parte: Gradle", "↳ Part: Gradle", "↳ Partie : Gradle", "↳ Teil: Gradle"),
    group: "part",
    home: "https://gradle.org",
    metaSource: "services.gradle.org/versions/current",
    binarySource: () =>
      tr(
        "Campo downloadUrl del JSON ufficiale",
        "downloadUrl field of the official JSON",
        "Champ downloadUrl du JSON officiel",
        "Feld downloadUrl des offiziellen JSON",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da checksumUrl del JSON",
        "SHA-256 from checksumUrl of the JSON",
        "SHA-256 depuis checksumUrl du JSON",
        "SHA-256 aus checksumUrl des JSON",
      ),
    targetPath: "gradle\\bin\\gradle.bat (via data\\kit\\bin\\gradle.cmd)",
  }),
  defSource({
    id: "kotlin",
    logoId: "kotlin",
    name: "Kotlin Compiler",
    group: "languages",
    home: "https://kotlinlang.org",
    metaSource: "api.github.com/repos/JetBrains/kotlin/releases?per_page=100",
    binarySource: "Asset kotlin-compiler-<v>.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "GitHub asset.digest o <asset>.sha256",
        "GitHub asset.digest or <asset>.sha256",
        "GitHub asset.digest ou <asset>.sha256",
        "GitHub asset.digest oder <asset>.sha256",
      ),
    targetPath: () => `kotlin\\${lLine()}\\bin\\kotlinc.bat`,
  }),
  defSource({
    id: "android",
    logoId: "android",
    name: "Android SDK + Command-line Tools",
    group: "tools",
    home: "https://developer.android.com/studio",
    metaSource: () =>
      tr(
        "developer.android.com/studio (tabella commandlinetools-win-*_latest.zip)",
        "developer.android.com/studio (commandlinetools-win-*_latest.zip table)",
        "developer.android.com/studio (table commandlinetools-win-*_latest.zip)",
        "developer.android.com/studio (Tabelle commandlinetools-win-*_latest.zip)",
      ),
    binarySource: "dl.google.com/android/repository/commandlinetools-win-<build>_latest.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 pubblicato nella tabella ufficiale Android",
        "SHA-256 published in the official Android table",
        "SHA-256 publié dans la table officielle Android",
        "SHA-256 aus der offiziellen Android-Tabelle",
      ),
    targetPath: () =>
      tr(
        "android\\cmdline-tools\\latest\\bin\\sdkmanager.bat e adb.exe",
        "android\\cmdline-tools\\latest\\bin\\sdkmanager.bat and adb.exe",
        "android\\cmdline-tools\\latest\\bin\\sdkmanager.bat et adb.exe",
        "android\\cmdline-tools\\latest\\bin\\sdkmanager.bat und adb.exe",
      ),
  }),
  defSource({
    id: "dotnet",
    logoId: "dotnet",
    name: ".NET SDK",
    group: "languages",
    home: "https://dotnet.microsoft.com",
    metaSource: "builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json",
    binarySource: () =>
      tr(
        "Archivio .zip per rid=win-x64|win-arm64",
        ".zip archive for rid=win-x64|win-arm64",
        "Archive .zip pour rid=win-x64|win-arm64",
        ".zip-Archiv für rid=win-x64|win-arm64",
      ),
    verifyKind: "SHA-512",
    verifyDetail: () =>
      tr(
        "SHA-512 dal campo hash di releases.json",
        "SHA-512 from the hash field of releases.json",
        "SHA-512 depuis le champ hash de releases.json",
        "SHA-512 aus dem Feld hash von releases.json",
      ),
    targetPath: () =>
      tr(
        `dotnet\\${lLine()}\\dotnet.exe (con junction SDK condivise)`,
        `dotnet\\${lLine()}\\dotnet.exe (with shared SDK junctions)`,
        `dotnet\\${lLine()}\\dotnet.exe (avec jonctions SDK partagées)`,
        `dotnet\\${lLine()}\\dotnet.exe (mit geteilten SDK-Junctions)`,
      ),
  }),
  defSource({
    id: "dart",
    logoId: "dart",
    name: "Dart SDK",
    group: "languages",
    home: "https://dart.dev",
    metaSource: "storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION",
    binarySource: "dartsdk-windows-<arch>-release.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 da <url>.sha256sum",
        "SHA-256 from <url>.sha256sum",
        "SHA-256 depuis <url>.sha256sum",
        "SHA-256 aus <url>.sha256sum",
      ),
    targetPath: () => `dart\\${lLine()}\\bin\\dart.exe`,
  }),
  defSource({
    id: "zig",
    logoId: "zig",
    name: "Zig",
    group: "languages",
    home: "https://ziglang.org",
    metaSource: "ziglang.org/download/index.json",
    binarySource: () =>
      tr(
        "Campo tarball per (x86_64|aarch64)-windows",
        "tarball field for (x86_64|aarch64)-windows",
        "Champ tarball pour (x86_64|aarch64)-windows",
        "Feld tarball für (x86_64|aarch64)-windows",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dal campo shasum del JSON",
        "SHA-256 from the shasum field of the JSON",
        "SHA-256 depuis le champ shasum du JSON",
        "SHA-256 aus dem Feld shasum des JSON",
      ),
    targetPath: () => `zig\\${lLine()}\\zig.exe`,
  }),
  defSource({
    id: "julia",
    logoId: "julia",
    name: "Julia",
    group: "languages",
    home: "https://julialang.org",
    metaSource: "julialang-s3.julialang.org/bin/versions.json",
    binarySource: () =>
      tr(
        "Archivio .zip per os=winnt, arch=x86_64",
        ".zip archive for os=winnt, arch=x86_64",
        "Archive .zip pour os=winnt, arch=x86_64",
        ".zip-Archiv für os=winnt, arch=x86_64",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dal campo sha256 di versions.json",
        "SHA-256 from the sha256 field of versions.json",
        "SHA-256 depuis le champ sha256 de versions.json",
        "SHA-256 aus dem Feld sha256 von versions.json",
      ),
    targetPath: () => `julia\\${lLine()}\\bin\\julia.exe`,
  }),
  defSource({
    id: "bun",
    logoId: "bun",
    name: "Bun",
    group: "languages",
    home: "https://bun.sh",
    metaSource: "api.github.com/repos/oven-sh/bun/releases",
    binarySource: "Asset bun-windows-(x64|aarch64).zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "GitHub asset.digest o SHASUMS256.txt",
        "GitHub asset.digest or SHASUMS256.txt",
        "GitHub asset.digest ou SHASUMS256.txt",
        "GitHub asset.digest oder SHASUMS256.txt",
      ),
    targetPath: () => `bun\\${lLine()}\\bun.exe`,
  }),
  defSource({
    id: "deno",
    logoId: "deno",
    name: "Deno",
    group: "languages",
    home: "https://deno.com",
    metaSource: "api.github.com/repos/denoland/deno/releases",
    binarySource: "Asset deno-(x86_64|aarch64)-pc-windows-msvc.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "GitHub asset.digest o <asset>.sha256sum",
        "GitHub asset.digest or <asset>.sha256sum",
        "GitHub asset.digest ou <asset>.sha256sum",
        "GitHub asset.digest oder <asset>.sha256sum",
      ),
    targetPath: () => `deno\\${lLine()}\\deno.exe`,
  }),
  defSource({
    id: "typst",
    logoId: "typst",
    name: "Typst",
    group: "languages",
    home: "https://typst.app",
    metaSource: "api.github.com/repos/typst/typst/releases",
    binarySource: "Asset typst-(x86_64|aarch64)-pc-windows-msvc.zip",
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () => `typst\\${lLine()}\\typst.exe`,
  }),
  defSource({
    id: "tinytex",
    logoId: "tinytex",
    name: "TinyTeX (LaTeX)",
    group: "tools",
    home: "https://yihui.org/tinytex/",
    metaSource: "api.github.com/repos/rstudio/tinytex-releases/releases/latest",
    unix: () =>
      tr(
        "macOS e Linux: TinyTeX-1-<os>-<arch>-v*.tar.xz (universale su macOS)",
        "macOS and Linux: TinyTeX-1-<os>-<arch>-v*.tar.xz (universal on macOS)",
        "macOS et Linux : TinyTeX-1-<os>-<arch>-v*.tar.xz (universel sur macOS)",
        "macOS und Linux: TinyTeX-1-<os>-<arch>-v*.tar.xz (universell auf macOS)",
      ),
    binarySource: () =>
      tr(
        "Asset TinyTeX-1-windows-v*.exe (7z SFX estratto in Rust)",
        "Asset TinyTeX-1-windows-v*.exe (7z SFX extracted in Rust)",
        "Asset TinyTeX-1-windows-v*.exe (7z SFX extrait en Rust)",
        "Asset TinyTeX-1-windows-v*.exe (7z-SFX in Rust entpackt)",
      ),
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () =>
      tr(
        "tinytex\\bin\\windows\\tlmgr.bat e pdftex.exe",
        "tinytex\\bin\\windows\\tlmgr.bat and pdftex.exe",
        "tinytex\\bin\\windows\\tlmgr.bat et pdftex.exe",
        "tinytex\\bin\\windows\\tlmgr.bat und pdftex.exe",
      ),
  }),
  defSource({
    id: "pnpm",
    logoId: "pnpm",
    name: "pnpm + yarn (corepack)",
    group: "tools",
    home: "https://pnpm.io",
    metaSource: () =>
      tr(
        "registry.npmjs.org/pnpm/latest e /corepack/latest",
        "registry.npmjs.org/pnpm/latest and /corepack/latest",
        "registry.npmjs.org/pnpm/latest et /corepack/latest",
        "registry.npmjs.org/pnpm/latest und /corepack/latest",
      ),
    binarySource: () =>
      tr(
        "Installato con npm-cli.js del Node.js del kit",
        "Installed via npm-cli.js of the kit's Node.js",
        "Installé avec npm-cli.js du Node.js du kit",
        "Installiert über npm-cli.js des Kit-Node.js",
      ),
    verifyKind: "npm sha512",
    verifyDetail: () =>
      tr(
        "Integrità del registro npm",
        "npm registry integrity",
        "Intégrité du registre npm",
        "Integrität der npm-Registry",
      ),
    targetPath: () =>
      tr(
        "data\\npm-global\\pnpm.cmd e yarn.cmd",
        "data\\npm-global\\pnpm.cmd and yarn.cmd",
        "data\\npm-global\\pnpm.cmd et yarn.cmd",
        "data\\npm-global\\pnpm.cmd und yarn.cmd",
      ),
  }),
  defSource({
    id: "cmake",
    logoId: "cmake",
    name: "CMake + Ninja",
    group: "tools",
    home: "https://cmake.org",
    metaSource: () =>
      tr(
        "GitHub API: Kitware/CMake e ninja-build/ninja (releases/latest)",
        "GitHub API: Kitware/CMake and ninja-build/ninja (releases/latest)",
        "API GitHub : Kitware/CMake et ninja-build/ninja (releases/latest)",
        "GitHub-API: Kitware/CMake und ninja-build/ninja (releases/latest)",
      ),
    binarySource: () =>
      tr(
        "cmake-*-windows-(x86_64|arm64).zip e ninja-win(arm64).zip",
        "cmake-*-windows-(x86_64|arm64).zip and ninja-win(arm64).zip",
        "cmake-*-windows-(x86_64|arm64).zip et ninja-win(arm64).zip",
        "cmake-*-windows-(x86_64|arm64).zip und ninja-win(arm64).zip",
      ),
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () =>
      tr(
        `cmake\\${lLine()}\\bin\\cmake.exe e tools\\ninja.exe`,
        `cmake\\${lLine()}\\bin\\cmake.exe and tools\\ninja.exe`,
        `cmake\\${lLine()}\\bin\\cmake.exe et tools\\ninja.exe`,
        `cmake\\${lLine()}\\bin\\cmake.exe und tools\\ninja.exe`,
      ),
  }),
  defSource({
    id: "cc",
    logoId: "cc",
    name: "C/C++ (llvm-mingw)",
    group: "tools",
    home: "https://github.com/mstorsjo/llvm-mingw",
    metaSource: "api.github.com/repos/mstorsjo/llvm-mingw/releases/latest",
    binarySource: "Asset llvm-mingw-*-ucrt-(x86_64|aarch64).zip",
    unix: () =>
      tr(
        "macOS e Linux: shim cc, c++, gcc, g++, clang sullo zig cc del kit (richiede Zig) e GNU make da conda-forge, non rilocabile",
        "macOS and Linux: cc, c++, gcc, g++, clang shims over the kit's zig cc (needs Zig) and GNU make from conda-forge, not relocatable",
        "macOS et Linux : shims cc, c++, gcc, g++, clang sur le zig cc du kit (requiert Zig) et GNU make de conda-forge, non relocalisable",
        "macOS und Linux: Shims cc, c++, gcc, g++, clang über das zig cc des Kits (braucht Zig) und GNU make aus conda-forge, nicht verschiebbar",
      ),
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: "cc\\bin\\clang.exe (+ make.cmd)",
  }),
  defSource({
    id: "ruby",
    logoId: "ruby",
    name: "Ruby + Bundler",
    group: "languages",
    home: "https://www.ruby-lang.org",
    metaSource: "api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30",
    binarySource: "Asset rubyinstaller-<v>-(x64|arm).7z",
    unix: () =>
      tr(
        "macOS e Linux: conda-forge ruby (3.2 e successive) con micromamba, non rilocabile",
        "macOS and Linux: conda-forge ruby (3.2 and later) via micromamba, not relocatable",
        "macOS et Linux : ruby de conda-forge (3.2 et suivantes) via micromamba, non relocalisable",
        "macOS und Linux: conda-forge ruby (ab 3.2) über micromamba, nicht verschiebbar",
      ),
    verifyKind: "SHA-256",
    verifyDetail: "GitHub Release asset.digest",
    targetPath: () => `ruby\\${lLine()}\\bin\\ruby.exe`,
  }),
  defSource({
    id: "r",
    logoId: "r",
    name: "R (conda-forge + OpenBLAS)",
    group: "languages",
    home: "https://www.r-project.org",
    metaSource: "api.anaconda.org/package/conda-forge/r-base + mamba-org/micromamba-releases",
    unix: () =>
      tr(
        "Windows ARM64: i pacchetti win-64 in emulazione x64, da micromamba nativo (--platform win-64). macOS e Linux: gli stessi pacchetti per osx-* e linux-*, lanciatori in kit-bin, non rilocabile",
        "Windows ARM64: the win-64 packages under x64 emulation, from the native micromamba (--platform win-64). macOS and Linux: the same packages for osx-* and linux-*, launchers in kit-bin, not relocatable",
        "Windows ARM64 : les paquets win-64 en émulation x64, par le micromamba natif (--platform win-64). macOS et Linux : mêmes paquets pour osx-* et linux-*, lanceurs dans kit-bin, non relocalisable",
        "Windows ARM64: die win-64-Pakete in x64-Emulation, vom nativen micromamba (--platform win-64). macOS und Linux: dieselben Pakete für osx-* und linux-*, Starter in kit-bin, nicht verschiebbar",
      ),
    binarySource: () =>
      tr(
        "micromamba-win-64 + pacchetti r-base e libblas=*=*openblas",
        "micromamba-win-64 + r-base and libblas=*=*openblas packages",
        "micromamba-win-64 + paquets r-base et libblas=*=*openblas",
        "micromamba-win-64 + Pakete r-base und libblas=*=*openblas",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 di micromamba + SHA-256 pacchetti conda-forge",
        "micromamba SHA-256 + conda-forge package SHA-256",
        "SHA-256 de micromamba + SHA-256 des paquets conda-forge",
        "SHA-256 von micromamba + SHA-256 der conda-forge-Pakete",
      ),
    targetPath: () =>
      tr(
        `r\\${lLine()}\\bin\\R.cmd e Rscript.cmd`,
        `r\\${lLine()}\\bin\\R.cmd and Rscript.cmd`,
        `r\\${lLine()}\\bin\\R.cmd et Rscript.cmd`,
        `r\\${lLine()}\\bin\\R.cmd und Rscript.cmd`,
      ),
  }),
  defSource({
    id: "mysql",
    logoId: "mysql",
    name: "MySQL Community Server",
    group: "databases",
    home: "https://www.mysql.com",
    metaSource: () =>
      tr(
        "Linee 8.4 e 9.7 LTS fissate nel catalogo (sha256ByLine)",
        "8.4 and 9.7 LTS lines pinned in the catalog (sha256ByLine)",
        "Lignes LTS 8.4 et 9.7 fixées dans le catalogue (sha256ByLine)",
        "LTS-Linien 8.4 und 9.7 im Katalog festgelegt (sha256ByLine)",
      ),
    binarySource: "cdn.mysql.com/Downloads/MySQL-<linea>/mysql-<v>-winx64.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 fissato per linea e target dopo la firma GPG di Oracle",
        "SHA-256 pinned per line and target after Oracle's GPG signature",
        "SHA-256 fixé par ligne et cible après la signature GPG d'Oracle",
        "SHA-256 pro Linie und Ziel nach Oracles GPG-Signatur festgelegt",
      ),
    targetPath: () =>
      tr(
        `mysql\\${lLine()}\\bin\\mysqld.exe e mysql.exe`,
        `mysql\\${lLine()}\\bin\\mysqld.exe and mysql.exe`,
        `mysql\\${lLine()}\\bin\\mysqld.exe et mysql.exe`,
        `mysql\\${lLine()}\\bin\\mysqld.exe und mysql.exe`,
      ),
  }),
  defSource({
    id: "mongodb",
    logoId: "mongodb",
    name: "MongoDB + mongosh + Database Tools",
    group: "databases",
    home: "https://www.mongodb.com",
    metaSource: "downloads.mongodb.org/current.json, GitHub mongodb-js/mongosh, tools/db/release.json",
    unix: () =>
      tr(
        "macOS: archivi del fornitore; Linux: server da conda-forge (non rilocabile), mongosh generico, Database Tools per distribuzione",
        "macOS: vendor archives; Linux: server from conda-forge (not relocatable), generic mongosh, Database Tools per distribution",
        "macOS : archives de l'éditeur ; Linux : serveur depuis conda-forge (non relocalisable), mongosh générique, Database Tools par distribution",
        "macOS: Archive des Herstellers; Linux: Server aus conda-forge (nicht verschiebbar), generisches mongosh, Database Tools je Distribution",
      ),
    binarySource: () =>
      tr(
        "Archivi .zip Windows x64 di mongod, mongosh e mongodump/mongorestore",
        "Windows x64 .zip archives of mongod, mongosh and mongodump/mongorestore",
        "Archives .zip Windows x64 de mongod, mongosh et mongodump/mongorestore",
        "Windows-x64-.zip-Archive von mongod, mongosh und mongodump/mongorestore",
      ),
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dai JSON MongoDB + asset.digest GitHub per mongosh",
        "SHA-256 from MongoDB JSONs + GitHub asset.digest for mongosh",
        "SHA-256 depuis les JSON MongoDB + asset.digest GitHub pour mongosh",
        "SHA-256 aus den MongoDB-JSONs + GitHub asset.digest für mongosh",
      ),
    targetPath: () => `mongodb\\${lLine()}\\bin\\mongod.exe, mongosh\\, mongodb-tools\\`,
  }),
  defSource({
    id: "phpmyadmin",
    logoId: "phpmyadmin",
    name: () =>
      tr("phpMyAdmin (per MySQL)", "phpMyAdmin (for MySQL)", "phpMyAdmin (pour MySQL)", "phpMyAdmin (für MySQL)"),
    group: "databases",
    home: "https://www.phpmyadmin.net",
    metaSource: "www.phpmyadmin.net/home_page/version.json",
    binarySource: "files.phpmyadmin.net/phpMyAdmin/<v>/phpMyAdmin-<v>-all-languages.zip",
    verifyKind: "SHA-256",
    verifyDetail: () =>
      tr(
        "SHA-256 dal file .sha256 ufficiale + controllo sintattico php -l",
        "SHA-256 from the official .sha256 file + php -l syntax check",
        "SHA-256 depuis le fichier .sha256 officiel + vérification syntaxique php -l",
        "SHA-256 aus der offiziellen .sha256-Datei + Syntaxprüfung php -l",
      ),
    targetPath: () => `phpmyadmin\\${lLine()}\\index.php`,
  }),
  defSource({
    id: "sqlite",
    logoId: "sqlite",
    name: "SQLite CLI",
    unix: () =>
      tr(
        "Linux ARM64: conda-forge sqlite (sqlite.org non pubblica gli strumenti), non rilocabile; Windows ARM64: lo zip x64 se manca quello arm64",
        "Linux ARM64: conda-forge sqlite (sqlite.org publishes no tools), not relocatable; Windows ARM64: the x64 zip when there is no arm64 one",
        "Linux ARM64 : sqlite de conda-forge (sqlite.org ne publie pas les outils), non relocalisable ; Windows ARM64 : le zip x64 s'il manque le zip arm64",
        "Linux ARM64: conda-forge sqlite (sqlite.org veröffentlicht keine Tools), nicht verschiebbar; Windows ARM64: das x64-Zip, wenn das arm64-Zip fehlt",
      ),
    group: "databases",
    home: "https://sqlite.org",
    metaSource: () =>
      tr(
        "sqlite.org/download.html (righe CSV PRODUCT,...)",
        "sqlite.org/download.html (PRODUCT,... CSV rows)",
        "sqlite.org/download.html (lignes CSV PRODUCT,...)",
        "sqlite.org/download.html (CSV-Zeilen PRODUCT,...)",
      ),
    binarySource: () =>
      tr(
        "sqlite.org/<anno>/sqlite-tools-win-(x64|arm64)-<v>.zip",
        "sqlite.org/<year>/sqlite-tools-win-(x64|arm64)-<v>.zip",
        "sqlite.org/<année>/sqlite-tools-win-(x64|arm64)-<v>.zip",
        "sqlite.org/<Jahr>/sqlite-tools-win-(x64|arm64)-<v>.zip",
      ),
    verifyKind: "SHA3-256",
    verifyDetail: () =>
      tr(
        "SHA3-256 pubblicato nella riga PRODUCT della pagina ufficiale",
        "SHA3-256 published in the PRODUCT row of the official page",
        "SHA3-256 publié dans la ligne PRODUCT de la page officielle",
        "SHA3-256 aus der PRODUCT-Zeile der offiziellen Seite",
      ),
    targetPath: () => `sqlite\\${lLine()}\\sqlite3.exe`,
  }),
];
