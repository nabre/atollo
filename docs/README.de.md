<p align="center">
  <a href="https://nabre.github.io/atollo/de/">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="brand/wordmark-dark.svg">
      <img src="brand/wordmark-light.svg" alt="Atollo" width="280">
    </picture>
  </a>
</p>

<h1 align="center">Atollo — Universal Portable Dev & AI Workspace</h1>

<p align="center">
  Eine portable Entwicklungsumgebung ohne Installation für Windows x64, Macs mit Apple Silicon und Linux x64 und ARM64 (die Systeme, auf denen Docker Sandboxes läuft), die jedes Projekt in seiner eigenen Docker-Sandboxes-microVM ausführt.<br>
  Editor, über 20 Sprachen und Runtimes, lokale Datenbanken, lokale KI-Modelle und die KI-Agenten Claude Code, Codex, Antigravity und Pi in einem Ordner, den du auf einem USB-Stick mitnimmst.
</p>

<p align="center">
  <a href="https://github.com/nabre/atollo/releases"><img alt="Latest release" src="https://img.shields.io/github/v/release/nabre/atollo?include_prereleases&label=release"></a>
  <a href="https://github.com/nabre/atollo/releases"><img alt="Downloads" src="https://img.shields.io/github/downloads/nabre/atollo/total?label=downloads"></a>
  <a href="https://github.com/nabre/atollo/actions/workflows/app.yml"><img alt="Build" src="https://img.shields.io/github/actions/workflow/status/nabre/atollo/app.yml?branch=main&label=build"></a>
  <a href="../LICENSE"><img alt="License: MIT" src="https://img.shields.io/github/license/nabre/atollo?label=license"></a>
  <img alt="Platforms: Windows, macOS, Linux" src="https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-informational">
  <img alt="Architectures: x64, ARM64" src="https://img.shields.io/badge/arch-x64%20%7C%20ARM64-informational">
  <img alt="Built with Rust and Tauri 2" src="https://img.shields.io/badge/built%20with-Rust%20%2B%20Tauri%202-informational">
</p>

<p align="center">
  <a href="https://nabre.github.io/atollo/de/"><b>Website</b></a> ·
  <a href="https://github.com/nabre/atollo/releases/latest"><b>Download</b></a> ·
  <a href="https://nabre.github.io/atollo/de/guide/"><b>Anleitung</b></a> ·
  <a href="https://github.com/nabre/atollo/issues"><b>Issues</b></a> ·
  <a href="PRIVACY.de.md"><b>Datenschutz</b></a>
</p>

<p align="center"><a href="../README.md">English</a> · <a href="README.it.md">Italiano</a> · <a href="README.fr.md">Français</a> · Deutsch</p>

**Atollo** ist ein eigenständiger *Universal Portable Dev & AI Workspace* (`Atollo.exe`). Sein Kern ist der portable Konfigurator von **Docker Sandboxes** (`sbx`): Jedes Projekt läuft in seiner eigenen microVM, mit seinem Terminal, seinen Aktionen, seiner Website, den Terminals des Editors und den Agenten Claude Code, Codex und Pi, und sieht nur seinen eigenen Ordner. Um diesen Kern herum vereint Atollo **Editor** (integriertes VSCodium), **Toolchain-/Runtime-Manager** (über 20 bei Bedarf heruntergeladene Sprachen und Werkzeuge, auch für das Linux der VM), **lokale Server/Datenbanken** (MySQL, MongoDB, SQLite, Caddy/Apache) und **KI-Agenten-Orchestrierung** (Claude Code, Codex, Antigravity und Pi mit synchronisierten Anweisungen, Skills und MCP-Servern, dazu lokale Sprachmodelle mit llama.cpp) in einem einzigen portablen Ordner vereint. Keine Installation, keine Administratorrechte, keine Systemdienste außerhalb des Workspace-Ordners.

Jedes Programm lebt in einer von drei Boxen:

- **Die VM**: alles, was Code eines Projekts ausführt. Das Terminal des Projekts, seine Aktionen (`npm run dev`…) und ihre Websites (ein Container in der VM, sein Port auf dem PC veröffentlicht), die Websites der Webserver-Pools, ein Projekt, das aus einem Terminal oder einem Framework angelegt wird, die Terminals, Tasks und Agentenprofile des Editors und die Agenten: Claude Code und Codex in den Agenten-Sandboxes von `sbx`, Pi in der VM des Projekts (sein Build für Linux). Drinnen: der Ordner des Projekts, beschreibbar, und die Werkzeuge des Kits für Linux in den Versionen des Projekts, schreibgeschützt. Zum PC hin: nur der Port der Datenbank des Projekts und, wenn das Projekt es erlaubt, GitHub.
- **Das Kit auf dem PC**: die Programme des Kits, nie ein Agent: die Terminals des Kits („Alle Werkzeuge“ und deine eigenen, in jedem Ordner, mit den Werkzeugen des Kits, aber ohne die CLIs der Agenten, und einem Hinweis, dass Agenten nur in der VM laufen), die Datenbankserver und ihre Werkzeuge (phpMyAdmin, mongo-express, Sicherungen), llama.cpp, das Fenster des Editors, Git zum Klonen, `sbx` selbst und die App. Antigravity, ein Desktop-Editor, bleibt hier für die synchronisierten Anweisungen, Skills und MCP-Server und startet nie auf einem Projekt.
- **Ohne VM**: Ist die VM nicht verfügbar, laufen Terminal, Aktionen, Websites und Pools des Projekts auf dem PC mit einer versiegelten Umgebung und ohne die CLIs der Agenten; Terminals, Tasks und Agentenprofile des Editors starten nicht (das Terminal sagt, warum und wie es sich beheben lässt; das Fenster des Editors öffnet sich trotzdem), und kein Agent startet. Die Statusleiste sagt es mit der Warnung **Ohne VM**, den Gründen und wie sich jeder beheben lässt.

| Was | Mit VM | Ohne VM |
| --- | --- | --- |
| Terminals des Kits („Alle Werkzeuge“ und deine eigenen, jeder Ordner) | auf dem PC mit den Werkzeugen des Kits, ohne die CLIs der Agenten, mit einem Hinweis, dass Agenten nur in der VM laufen | genauso |
| Terminal des Projekts, Aktionen, ihre Websites, Webserver-Pools, ein Projekt aus einem Terminal oder einem Framework anlegen | in der VM des Projekts (Ports auf dem PC veröffentlicht, Datenbanken des Kits über ihren Port erreicht) | auf dem PC mit der versiegelten Umgebung, ohne die CLIs der Agenten |
| Terminals, Tasks und Agentenprofile des Editors | in der VM des Projekts, mit den Werkzeugversionen des Projekts | starten nicht: Das Terminal sagt, warum und wie es sich beheben lässt; das Fenster des Editors öffnet sich trotzdem |
| Agenten | Claude Code und Codex in ihren eigenen Sandboxes, Pi in der VM des Projekts; Antigravity startet nie auf einem Projekt | kein Agent startet |

Beim ersten Start lädt die App die erforderlichen Komponenten selbst herunter: PowerShell 7, GitHub CLI, den Editor und Docker Sandboxes (0.49.2 oder neuer, etwa 470 MB, auf den Systemen, für die Docker es veröffentlicht). Alles andere wird auf der Seite „Werkzeuge“ gewählt oder kommt, wenn es gebraucht wird: Node.js mit einer `npm`-Aktion, uv mit einem `uvx`-MCP-Server und so weiter.

<picture>
  <source media="(prefers-color-scheme: light)" srcset="../site/shots/de/shot-projects-light.png">
  <img src="../site/shots/de/shot-projects-dark.png" alt="Die Seite Projekte von Atollo: Schnellübersicht und Aktivität" width="100%">
</picture>

## Funktionen

- **Jedes Projekt in seiner eigenen microVM.** Mit Docker Sandboxes bekommt jedes Projekt eine eigene microVM (Kernel, Netzwerk hinter dem Proxy von `sbx` mit Atollos Richtlinie `balanced`, eigenes Docker): das Terminal des Projekts (`sbx exec`), seine Aktionen, seine Website (der statische, Caddy- oder Apache-Server des Kits wird ein Container in der VM, `httpd:2.4` oder `php:<Version>-apache`, mit den Aliassen des Projekts selbst, sein Port auf `127.0.0.1` des PCs veröffentlicht oder im lokalen Netzwerk, wenn die Einstellungen es erlauben), die Websites der Webserver-Pools, die Projekte, die aus einem Terminal oder einem Framework angelegt werden, Claude Code und Codex in den Agenten-Sandboxes von `sbx` und Pi in der VM selbst. Die VM bindet den Ordner des Projekts beschreibbar und die Werkzeuge des Kits für Linux schreibgeschützt ein, in den Versionen, die das Projekt gewählt hat: einmal ins Kit heruntergeladen und per SHA-256 geprüft, reisen sie mit dem Kit, funktionieren offline, starten schnell, die VM kann sie nicht verändern und braucht keine Download-Seiten für sie; die eigenen Pakete des Projekts (`node_modules`, `vendor`, `.venv`) installiert das Projekt in der VM. Es wird nur das `sbx` des Kits benutzt, nie eines, das auf dem PC installiert ist; seine Sandboxes, Festplatten, Images, Einstellungen und Logs bleiben in `data`. Das Fenster der VM (ein Klick auf VM in der Statusleiste) listet die Sandboxes des Kits, wem jede gehört, ihren tatsächlichen Platz auf der Festplatte, und entfernt sie. **Beenden** stoppt eine Aktion in ihrer VM und nimmt ihren Port zurück. Wechselt das Kit Laufwerksbuchstaben oder Ordner, wird die Sandbox eines Projekts von selbst neu angelegt: Dateien, Datenbanken, Werkzeuge und Anmeldungen liegen im Kit, verloren gehen nur das Home der VM und die heruntergeladenen Images.
- **Ohne VM? Eine klare Warnung.** Kann der PC die VM nicht ausführen (ein nicht unterstütztes System, der Hypervisor aus, `sbx` nicht bei Docker angemeldet, ein anderes Docker Sandboxes läuft schon, ein FAT32- oder exFAT-Laufwerk…), zeigt die Statusleiste **Ohne VM** neben Aktivität; ein Klick nennt jeden Grund und seine Lösung: **Starten** öffnet `sbx login` im Terminal von Atollo, **Im Kit installieren** (ist eine Installation fehlgeschlagen, mit ihrem Grund; Atollo versucht es außerdem jede Stunde von selbst) und, für die Administratorschritte (Windows Hypervisor Platform, die Gruppe `kvm`, das AppArmor-Profil), der genaue Befehl mit einer Schaltfläche **Kopieren**, den du einmal selbst ausführst (Atollo startet sie nie und verlangt nie Administratorrechte); **Erneut prüfen** fragt neu. Solange laufen Terminal, Aktionen und Websites des Projekts auf dem PC mit einer versiegelten Umgebung: ohne die Variablen, die zu deinem echten Profil, deinem SSH-Agenten und der D-Bus-Sitzung führen, ohne die CLIs der Agenten, und mit dem Datenbank-Login des eigenen Benutzers des Projekts, nie dem Root des Kits. Die Terminals des Editors und die Agenten starten nicht.
- **Die Terminals des Kits auf dem PC.** „Alle Werkzeuge“ und deine eigenen Terminals laufen in jedem Ordner immer auf dem PC mit den Werkzeugen des Kits, mit oder ohne VM, aber ohne die CLIs der Agenten (`claude`, `codex`, `agy`, `pi`): Ein Hinweis im Terminal sagt, dass die Agenten nur in der VM laufen, von der Projektseite oder aus dem Editor.
- **Editor.** Integriertes VSCodium mit den Atollo-Themes dunkel und hell, in einem Fenster, das am Workbench von VSCodium 1.135 gemessen ist: dieselben Größen, dieselben Farben und die Codicon-Icons. Sein Fenster läuft auf dem PC. Seine Terminals (jedes Profil des Kits), seine Tasks und seine Agentenprofile (Claude Code, Codex, Pi) laufen in der VM des Projekts, in dem Ordner, in dem sie sich öffnen, mit den Werkzeugversionen des Projekts; Antigravity startet nie auf einem Projekt. Standardmäßig geschlossen: Ohne VM, oder hat Atollo nicht notiert, ob die VM aktiv ist, wurde die VM des Projekts nicht vorbereitet oder gehört der Ordner zu keinem Projekt, öffnet das Terminal nichts und sagt, warum und wie es sich beheben lässt; das Fenster des Editors öffnet sich trotzdem. Eine erklärte Grenze: Die Debugger-Adapter des Editors starten Programme auf dem PC.
- **Sprachen und Werkzeuge bei Bedarf.** Über 20 Sprachen und Werkzeuge (Node.js, Python, uv, PHP, Rust, Go, Java, .NET, Dart, Julia, Ruby, C/C++, Bun, Deno, Typst, TinyTeX, R, Android SDK…), heruntergeladen, wenn sie gebraucht werden, und per SHA-256-Prüfsumme oder Signatur geprüft. Mehrere Versionen nebeneinander, eine pro Projekt. Mit aktiver VM werden die gewählten Werkzeuge für das Linux der VM installiert, und der PC bekommt nur die Komponenten der App (Werkzeuge, die schon auf dem PC sind, bleiben, für einen PC ohne VM); die Matrix Komponenten × Systeme zeigt, was „in der VM“ läuft.
- **Lokale Datenbanken und Webserver.** MySQL mit phpMyAdmin, MongoDB mit mongo-express und SQLite, mit einem Klick gestartet und gestoppt, mit Datenbank, Benutzer und Passwort für jedes Projekt; mehrere Versionen nebeneinander, jede behält ihren Port. Die Server bleiben auf dem PC, im Kit: Die VM eines Projekts erreicht nur den Port seiner Datenbank, als Benutzer des Projekts (das Passwort in einer nur für dich lesbaren Datei, nie auf einer Befehlszeile); der Root-Login des Kits kommt nie in eine VM. Caddy und Apache für Websites unter `<Projekt>.localhost`, auf jedem System, allein oder in einem Server, den sich mehrere Projekte teilen (ein Pool): Mit VM läuft die Website jedes Projekts in seiner VM, ohne auf dem PC. Ein schon belegter Port wird abgelehnt, mit dem Namen dessen, der ihn hält. Keine Systemdienste.
- **GitHub in der VM, pro Projekt.** Der Git-Abschnitt eines Projekts hat „GitHub in der VM“, standardmäßig aus: Eingeschaltet fragt `sbx` das Kit über einen Befehl (`Atollo secret github`) nach dem Token, und sein Proxy fügt es den Anfragen an GitHub hinzu; das Token wird nie kopiert und kommt nie in die VM. Nie an ein Terminal des Kits gegeben.
- **Vier KI-Agenten, eine Konfiguration.** Claude Code, Codex, Antigravity und Pi: Installation, Anmeldung und Start. Anweisungen, Skills und MCP-Server schreibst du einmal in `data/agents`, und `atollo sync` hält alle vier gleich. Fertige MCP-Server sind einen Klick entfernt: GitHub, Playwright und Fetch für das ganze Kit, die Dateien des Projekts und seine MySQL- oder MongoDB-Datenbank für ein Projekt, ohne Token oder Passwort in den Dateien. Tariflimits und Verbrauch von heute und den letzten 7 Tagen. Agenten laufen nur in der VM: Claude Code und Codex in den Agenten-Sandboxes von `sbx`, mit den Anmeldungen, die `sbx` verwaltet, den Werkzeugen des Kits für das Projekt im `PATH` und seiner Datenbank (der Port für ihre Sandbox geöffnet), und Pi in der VM des Projekts mit seinem Build für Linux, von der Seite der App wie aus dem Editor; die MCP-Server des Kits erreichen sie noch nicht. Antigravity, ein Desktop-Editor, startet nie auf einem Projekt: Es bleibt im Kit für die synchronisierten Anweisungen, Skills und MCP-Server. Ohne VM startet kein Agent, und nie stattdessen auf dem PC.
- **Lokale KI-Modelle.** Sprachmodelle, die auf dem PC laufen, ohne Netzwerk und ohne Konto: GGUF-Modelle aus dem Katalog von Atollo oder aus jedem Hugging-Face-Repository, Downloads, die dort weitermachen, wo sie aufgehört haben (auch nach dem Schließen von Atollo), und per SHA-256 geprüft werden. Der Server von llama.cpp (CPU, oder GPU mit Vulkan oder Metal) lauscht nur auf 127.0.0.1, mit einem Schlüssel; Pi nutzt ihn ohne weitere Einstellungen, und sein Chat öffnet sich im Browser. Eine einzige Kopie in `data/models`, gemeinsam für Windows, macOS und Linux.
- **Ein Ordner für vier Systeme.** Derselbe Ordner oder dieselbe externe SSD wechselt von Windows zu macOS zu Linux, Projekte, Datenbanken und Agenten-Einstellungen bleiben erhalten: Jeder Rechner führt die nativen Programme seines Systems aus. „Laufwerk entfernen“ schließt alles, was Atollo geöffnet hat, und lässt das Laufwerk auswerfen.

## Dokumentation

Die ausführliche Dokumentation ist auf Englisch (und Italienisch):

| Dokument | Inhalt |
| --- | --- |
| [Anleitung](https://nabre.github.io/atollo/de/guide/) | jede Seite der App, Schritt für Schritt, mit Screenshots, auf Deutsch auf der Website (auch in der App) |
| [Komponentenkatalog](app/CATALOG.md) | die Komponenten, woher jede geladen wird, Verfügbarkeit pro System |
| [Matrix Werkzeuge × Systeme](app/TOOLS-MATRIX.html) | jede Komponente auf den vier Zielen: nativ oder nicht vorhanden |
| [Architektur](app/ARCHITECTURE.md) | wie der Code aufgebaut ist und wohin eine Änderung gehört |
| [Tests](app/TESTING.md) | automatische Prüfungen und die manuelle Checkliste pro System |
| [Entscheidungen](app/DECISIONS.md) und [Neuigkeiten](app/CHANGELOG.md) | die Gründe jeder Wahl und was sich geändert hat, auf Italienisch |

Offene Arbeit steht in den [Issues](https://github.com/nabre/atollo/issues).

## Verwendung

1. Kopiere die exe von Atollo (zum Beispiel `Atollo-0.7.2-win32-x64.exe`) in einen leeren Ordner: einen USB-Stick, eine externe Festplatte oder einen Ordner in deinem Profil (Leerzeichen, Klammern oder `&` im Namen sind kein Problem).
2. Doppelklicke sie: Die App legt `projects\` und `data\` an. Beim ersten Start wählst du Agenten, Sprachen und Werkzeuge: Die App lädt sie herunter und prüft sie (Prüfsumme oder Signatur), Docker Sandboxes eingeschlossen.
3. Melde dich einmal pro PC bei Docker an: Mit einem (kostenlosen) Docker-Konto klickst du in der Statusleiste auf **Ohne VM** und dann auf **Starten** neben `sbx login`: Der Befehl öffnet sich im Terminal von Atollo mit dem `sbx` des Kits, und dort erscheint der Anmeldelink (ein Klick öffnet ihn in deinem Browser). Die anderen Gründe derselben Liste sagen, was der PC sonst noch braucht.
4. Lege auf der Seite **Projekte** ein Projekt an oder klone eines. Editor, Terminal und Agenten eines Projekts öffnen sich immer in seinem Ordner; mit aktiver VM öffnen sich Terminal, Aktionen, die Terminals des Editors und die Agenten in der microVM des Projekts.

### Voraussetzungen für die VM

- Windows 11 (x64) mit eingeschalteter **Windows Hypervisor Platform**;
- macOS 14 oder neuer auf Apple silicon;
- Ubuntu 24.04 oder neuer (x64 oder ARM64; keine Ableger) mit `/dev/kvm`, das dein Benutzer nutzen kann (Gruppe `kvm`), und, auf neueren Ubuntu-Versionen, die Benutzer-Namespaces einschränken, das AppArmor-Profil für den Shim der microVM des Kits (erneut, wenn das Kit den Ordner wechselt);
- Virtualisierung in der Firmware (BIOS/UEFI) eingeschaltet; in einer virtuellen Maschine verschachtelte Virtualisierung;
- ein kostenloses Docker-Konto (`sbx login`, einmal pro PC, ohne Administrator);
- das Kit auf NTFS, APFS oder ext4: Die Festplatten der VMs passen nicht auf FAT32 oder exFAT.

Den Rest erledigt Atollo selbst: Es installiert Docker Sandboxes im Kit (bei einem Fehler jede Stunde ein neuer Versuch), startet den Daemon des Kits, setzt die Netzwerkrichtlinie und die Einstellungen des Daemons, räumt die Junction des Daemons auf und legt die Sandbox eines Projekts neu an, wenn das Kit umgezogen ist. Die Administratorschritte (Windows Hypervisor Platform und ein Neustart, die Gruppe `kvm` und eine neue Anmeldung, das AppArmor-Profil) werden mit dem genauen Befehl und einer Schaltfläche **Kopieren** gezeigt, zum Beispiel `Enable-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform -All` in einer Administrator-PowerShell: Atollo startet sie nie. Keine Software kann die VM auf Windows 10, macOS vor 14, einem anderen Linux als Ubuntu 24.04 oder neuer, einem PC mit in der Firmware ausgeschalteter Virtualisierung, einer virtuellen Maschine ohne verschachtelte Virtualisierung oder einem Kit auf FAT32 oder exFAT bringen: Dort bleibt die Warnung **Ohne VM**. Docker Desktop wird nicht gebraucht.

Von der [Release-Seite](https://github.com/nabre/atollo/releases): `Atollo-<Version>-win32-x64.exe` für Windows, `Atollo-<Version>-darwin-arm64.app.zip` für macOS (Apple Silicon), `Atollo-<Version>-linux-x64` oder `Atollo-<Version>-linux-arm64` (`.AppImage` und `.tar.gz`) für Linux; jede Datei hat ihre `.sha256`. Der Name nennt die Version: Wenn sich die App selbst aktualisiert, bekommen die exe und die AppImage den Namen der neuen Version. In den Beispielen unten steht `Atollo.exe` für deine exe.

**macOS** (12 oder neuer, Apple Silicon). Die App hat nur eine Ad-hoc-Signatur, ohne Notarisierung durch Apple:
1. Entpacke das Zip und verschiebe `Atollo.app` mit dem Finder in den Ordner des Kits (zum Beispiel `/Volumes/KIT/Atollo/`).
2. Entferne im Terminal die Quarantäne des Browsers: `xattr -dr com.apple.quarantine /Volumes/KIT/Atollo/Atollo.app`.
3. Öffne `Atollo.app` aus diesem Ordner: `projects/` und `data/` entstehen daneben, nie darin.

Ohne Schritt 2 startet macOS die App aus einer temporären Kopie (App Translocation), und Atollo hält mit demselben Befehl an, statt das Kit in einem Ordner anzulegen, der wieder verschwindet.

**Linux** (x64 und ARM64, glibc 2.35 oder neuer: Ubuntu 22.04, Debian 12 und neuer). Kopiere `Atollo-<Version>-linux-<arch>.AppImage` in den Ordner des Kits, führe `chmod +x Atollo-<Version>-linux-<arch>.AppImage` aus und starte sie: Das Kit ist der Ordner der AppImage. Ohne FUSE (Container, manche minimalen Distributionen): Führe `./Atollo-<Version>-linux-<arch>.AppImage --appimage-extract-and-run` aus, oder nimm `Atollo-<Version>-linux-<arch>.tar.gz` mit installiertem WebKitGTK 4.1 (auf Ubuntu und Debian `sudo apt install libwebkit2gtk-4.1-0`): Entpacke `Atollo` in den Ordner des Kits und starte `./Atollo`. Die AppImage aktualisiert sich selbst; das `tar.gz` meldet, wenn eine neue Version zum Herunterladen bereitsteht.

## Seiten

| Seite | Inhalt |
| --- | --- |
| **Projekte** | die **Schnellübersicht** (was läuft, Favoriten und zuletzt benutzte Projekte mit Starten, Editor, Terminal und Agent, was zu beheben ist, ein Raster der Werkzeuge zum Installieren oder Aktualisieren mit ihrem Zustand, ein Klick öffnet die Seite jedes einzelnen in Werkzeuge), die Projektliste mit ▶ in jeder Zeile, Strg+P, um eines nach Namen zu öffnen, Strg+Umschalt+P für die Befehlspalette (ein Projekt starten oder stoppen, es öffnen, ein Agent darin, zu einer Seite wechseln, alles stoppen); **Neues Projekt**, ein Tab neben den Projekten: leer, aus deinen GitHub-Repositories (mit Klonen, Umbenennen und Löschen auf GitHub), von einer Git-Adresse oder aus einer von 63 Basen in Gruppen (Laravel, Next.js, Django, WordPress, Astro, Tauri…); Hinweise zum Workspace. Jedes Projekt hat eine eigene Seite, oben mit Starten, Editor, Terminal und Agent und darunter sieben Abschnitten: **Start** (Aktionen, Ports und ihre Ausgabe), **Website**, **Komponenten** (mit den Versionen), **Datenbanken** (die Datenbank des Projekts mit Benutzer und Passwort, Sicherungen, eine Sicherheitskopie vor jeder Wiederherstellung und jedem Versionswechsel), **Erweiterungen**, **Editor** (Einstellungen und MCP-Server), **Git** (mit „GitHub in der VM“) |
| **KI** | die ganze KI des Kits in zwei Bereichen. **Agenten**: die installierten Claude Code, Codex, Antigravity und Pi: Version, Anmeldung (mit dem Gemini-Schlüssel von Antigravity), Tariflimits und Verbrauch von heute und den letzten 7 Tagen; Anweisungen, Skills und MCP-Server gemeinsam in `data\agents`. **Lokale Modelle**: der Server von llama.cpp (Starten und Stoppen, Adresse, der Chat im Browser und sein Schlüssel), das Modell von Pi, der Katalog von Atollo mit GGUF-Modellen und „passt / an der Grenze / zu groß“ für RAM oder Videospeicher dieses PCs, jedes Hugging-Face-Repository (mit einem optionalen Token für zugangsbeschränkte Modelle), Downloads zum Anhalten und Fortsetzen; Modelle in `data\models`, nur auf diesem PC |
| **Datenbanken** | die MySQL- (mit phpMyAdmin) und MongoDB-Server (mit mongo-express) von Atollo: Starten und Stoppen, Administratorpasswort, Port, mehrere Versionen nebeneinander mit je eigenem Port, Daten in `data\db`, nur auf diesem PC; SQLite ohne Server |
| **Webserver** | die Caddy- und Apache-Server, die sich die Projekte teilen (Pools), und deine eigenen Profile, mit den `<Projekt>.localhost`-Websites, die sie ausliefern; nur auf diesem PC oder im lokalen Netzwerk, wenn die Einstellungen es erlauben; mit aktiver VM läuft die Website jedes Projekts in seiner VM, ohne auf dem PC |
| **Terminals** | Konsolen mit den Werkzeugen des Kits in jedem Ordner: „Alle Werkzeuge“ und eigene Terminals mit den Werkzeugen und Versionen, die du wählst; immer auf dem PC, ohne die CLIs der Agenten, mit einem Hinweis, dass Agenten nur in der VM laufen |
| **Werkzeuge** | Sprachen und Werkzeuge nach Art (System und Editor, KI-Agenten, lokale Modelle, Sprachen, Entwicklungswerkzeuge, Webserver, Datenbanken, Datenbank-Ansicht), mit Logo, Abhängigkeiten und Speicherplatz; installieren, aktualisieren, entfernen, mehrere Versionen (mit aktiver VM für das Linux der VM); der Rust-Compiler (GNU oder MSVC) auf der Seite von Rust; die Matrix Komponenten × Systeme, mit der Spalte der VM; die Matrix der Editor-Erweiterungen |
| **Anleitung** | die Anleitung in der App, in neun Kapiteln; jede Seite öffnet ihr eigenes Thema |
| **Einstellungen** | gespeichert, sobald du sie änderst: Sprache, Design, die temporären Dateien der Programme, die Pakete zum Neuinstallieren ohne Netzwerk, Netzwerk (Proxy, Zertifikate, lokales Netzwerk), Updates der App, Datenschutz; das **GitHub-Konto** von Atollo, eines für alles (App, Git, Editor, Konsolen, Agenten, GitHub CLI), und wer die Commits signiert; wo der Ordner von Atollo liegt; die Anweisungen der Frameworks; **Das Kit**: seine Ordner, sein Laufwerk, die Plattform und die Versionen, mit Zustand des Kits, Kopie des Kits, Diagnose exportieren und Problem melden |
| **Laufwerk entfernen** | schließt alles, was Atollo geöffnet hat, stoppt das Docker Sandboxes des Kits (und mit ihm die microVMs), holt die Anmeldung von Antigravity nach `data\` zurück und lässt das Laufwerk auswerfen |

Unten auf jeder Seite das Feld **Aktivität**: was läuft oder installiert wird (Aktionen der Projekte, jede Version der Datenbanken, Pools der Webserver, Komponenten), mit Ausgabe und Fehlern an einem Ort; „Alle beenden“ beendet nach einer Bestätigung alles. Die Statusleiste zählt dieselben Dinge und zeigt neben Aktivität die Pille der VM: grün **VM**, wenn die Projekte in ihren microVMs laufen, blau **VM… 40%**, während die VM eines Projekts vorbereitet wird, orange **Ohne VM**, wenn du den Grund beheben kannst, grau, wenn dieser PC sie nicht haben kann, rot **VM ⚠**, wenn eine Vorbereitung fehlgeschlagen ist; ein Klick öffnet die Gründe und ihre Lösungen und, sobald das Docker Sandboxes des Kits installiert ist, die Sandboxes des Kits mit ihrem Platz und einer Schaltfläche „Entfernen“. Jeder Download, jede Installation, jedes Update und jede Vorbereitung einer VM bleibt sichtbar, solange sie läuft: Die Statusleiste zeigt Titel, Schritt und einen kleinen Balken mit dem Prozentsatz („3 Aufgaben laufen“, wenn es mehr sind), ein Klick öffnet Schritte und Fehler in Aktivität; ein Terminal, eine Aktion oder ein Agent, der auf seine VM wartet, zeigt „VM wird vorbereitet…“ mit demselben Fortschritt. In Werkzeuge und im Schnellbereich zeigt jedes Werkzeug seine eigene Installation in seiner Zeile (in der Warteschlange, sein Balken, „Erneut versuchen“, „Abbrechen“): Du kannst inzwischen weitere anfordern, sie reihen sich ein.

Die Konsole von Atollo ist auf jedem System ein Tab im Terminal-Feld der App, mit der PowerShell 7 des Kits (unter Windows auf dem ConPTY von Microsoft, der Konsole von Windows Terminal, mit dem Kit heruntergeladen); sie öffnet nie das Terminal des PCs und hat nur die Umgebung von Atollo: Auf dem Host-PC installierte Programme stören nicht. Mit aktiver VM ist das Terminal eines Projekts dagegen eine `bash` in seiner microVM (`sbx exec`), im selben Feld; die Terminals des Kits bleiben auf dem PC. Wurde die gespeicherte Version eines Terminals entfernt, sagt es, welche es stattdessen nutzt (zum Beispiel PHP 8.2 → 8.4).

## Noch zu tun

Atollo ist in der Alpha (Version 0.7.2). Windows x64 ist das am meisten getestete System, echte Installationen in der CI eingeschlossen; macOS und Linux werden nur in der CI gebaut und getestet. Die vollständige Checkliste steht in [docs/app/TESTING.md](app/TESTING.md) (auf Englisch).

- **Die VM, noch fertigzustellen** ([D-175](app/DECISIONS.md), [D-178](app/DECISIONS.md)). Projekte in ihren microVMs sind nur unter Windows 11 x64 mit Docker Sandboxes 0.49.2 in echt getestet; macOS und Ubuntu sind ungetestet, ebenso „GitHub in der VM“ mit einem echten Konto. Noch zu tun: die MCP-Server des Kits für die Agenten in der VM (ihre Befehle sind Programme des PCs), in das MCP-Gateway von `sbx` zu bringen oder als fehlend zu melden.
- **Die Alpha, zum Ausprobieren.** [Die 0.7.2 Alpha](https://github.com/nabre/atollo/releases/tag/app-v0.7.2) ist auf GitHub für Windows x64, macOS auf Apple Silicon und Linux x64 und ARM64, die vier Systeme, auf denen Docker Sandboxes läuft ([D-176](app/DECISIONS.md)). Die veröffentlichten Dateien werden in der CI gebaut und getestet: Jetzt müssen sie auf sauberen Rechnern heruntergeladen und gestartet werden.
- **Alpha-Tests.** Gesucht sind Menschen, die Atollo in echten Projekten und auf verschiedenen PCs nutzen und melden, was nicht klappt: Installation der Komponenten, Projekte, Datenbanken, Agenten. Einstellungen › Das Kit › „Problem melden“ öffnet ein GitHub-Issue mit bereits eingetragener Version und System; häng das Zip von „Diagnose exportieren“ an (Geheimnisse maskiert, auf deinem PC erstellt, nie von der App gesendet).
- **macOS auf echten Macs** ([#5](https://github.com/nabre/atollo/issues/5), [#6](https://github.com/nabre/atollo/issues/6)). Die App für Apple Silicon wird in der CI gebaut und gestartet, nie auf einem frischen Mac: das im Browser geladene Zip, Quarantäne und Gatekeeper, App Translocation, ein Kit auf einem exFAT-Stick, die Anmeldungen der Agenten von einem Mac zum anderen mitgenommen.
- **Linux auf echten Rechnern** ([#5](https://github.com/nabre/atollo/issues/5)). x64 und ARM64 auf Ubuntu 24.04, Fedora 42 und Debian 13: AppImage mit und ohne FUSE, das `tar.gz` mit WebKitGTK 4.1, exFAT-Sticks oder mit `noexec` eingehängte Sticks, Anmeldung mit GNOME, KeePassXC oder KWallet.
- **Updates und Kit-Builds außerhalb von Windows.** Das Update der App von einer Version zur nächsten ist auf macOS und Linux noch nicht ausprobiert. Das Apache des Kits (2.4.69, D-116) ist im Katalog, aber nur in der CI ausprobiert; ebenso das PHP des Kits mit `pdo_sqlite` (8.4.26 und 8.5.11, D-133), das die App ab 0.3.0 nutzt.
- **Code-Signatur.** Auf macOS nur eine Ad-hoc-Signatur, ohne Notarisierung durch Apple: `xattr -dr com.apple.quarantine` ist nötig. Auf Windows erscheint 0.7.2 ohne Signatur: Die SignPath-Signatur muss noch aktiviert werden, bis dahin kann SmartScreen beim ersten Start warnen.

Du hast einen Mac mit Apple Silicon oder einen Linux-PC? Probier Atollo aus und erzähl uns, wie es lief: [Issue auf GitHub öffnen](https://github.com/nabre/atollo/issues).

## Bekannte Grenzen

- Auf Windows 10 und 11 (`x64`), macOS auf Apple Silicon und Linux (`x64` und `ARM64`): die Systeme, auf denen Docker Sandboxes läuft, der Kern von Atollo ([D-176](app/DECISIONS.md)). Auf jedem anderen System sagt Atollo das beim Start und hält an. Auf Linux ARM64 fehlt Android. Die Releases für macOS und Linux sind in der CI getestet, aber noch nicht auf sauberen Rechnern ([#5](https://github.com/nabre/atollo/issues/5)), und die Anmeldung der Agenten muss noch auf einem Mac getestet werden ([#6](https://github.com/nabre/atollo/issues/6)); Komponenten und Quellen pro System in [docs/app/CATALOG.md](app/CATALOG.md) (auf Englisch), die Matrix Komponente × System in [docs/app/TOOLS-MATRIX.html](app/TOOLS-MATRIX.html). Apache auf macOS und Linux ist ein eigener Build des Kits (D-116), mit PHP über `php-fpm`.
- Die VM (Docker Sandboxes) läuft nur, wo ihre [Voraussetzungen](#voraussetzungen-für-die-vm) erfüllt sind; anderswo (Windows 10, macOS vor 14, andere Linux-Distributionen, Virtualisierung aus, ein Kit auf FAT32 oder exFAT) laufen die Projekte auf dem PC mit der Warnung **Ohne VM**, und die Terminals des Editors und die Agenten starten nicht. Die Festplatte jeder Sandbox kann auf etwa 30 GB wachsen (Sparse-Dateien: Platz belegt nur, was geschrieben wird). Ein schon laufendes Docker Sandboxes auf dem PC (dort installiert oder aus einem anderen Kit) hält die VM des Kits aus, bis du es stoppst: Atollo stoppt es nie und benutzt es nie.
- Ohne VM gibt es keine Grenze zwischen einem Projekt und deinem Benutzer: Seine Programme können deine Dateien lesen und das Netz nutzen; Atollo filtert nur ihre Umgebung. Öffne in Atollo nur Projekte, denen du vertraust.
- Unter Linux braucht MySQL zwei Systembibliotheken, `libaio` und `libnuma`: auf Ubuntu 24.04 und Debian 13 `sudo apt install libaio1t64 libnuma1` (auf älteren Versionen `libaio1`), auf Fedora und RHEL `sudo dnf install libaio numactl-libs`. Fehlen sie, sagt Atollo es mit dem auszuführenden Befehl.
- Unter macOS und Linux funktioniert ein Atollo-Ordner auf exFAT oder FAT (ein für Windows formatiertes Laufwerk), aber diese Dateisysteme kennen keine Links: Wo ein Programm welche mitbringt, kopiert Atollo die Dateien (etwa die 142 internen Links von Git), was mehr Platz belegt. Auf einem Laufwerk nur für Mac oder Linux ist APFS oder ext4 besser.
- Außerhalb des Atollo-Ordners bleiben nur die Spuren von Windows, temporäre Übergangsdateien im System-Temp-Ordner (werden beim Beenden automatisch gelöscht; mit Einstellungen › Allgemein › Temporäre Dateien der Programme können ihn auch die Programme benutzen, solange sie laufen), die Anmeldung von GitHub CLI in der Anmeldeinformationsverwaltung, wenn du dich in einer Konsole mit `gh auth login` anmeldest (die in den Einstellungen bleibt in `data\gh`), die von Antigravity, solange `agy` geöffnet ist (nur für die Sitzung), auf macOS die Anmeldung von Claude Code im Schlüsselbund (außer du verbindest dich mit einem Token) und die Daten der Webansicht in deiner Library, und der Registry-Eintrag, den rustup unter Windows schreibt und Atollo entfernt. Für die VM: der Hypervisor (einmal von dir als Administrator eingeschaltet), die Docker-Anmeldung (`sbx login`, im Anmeldedatenspeicher des Systems, einmal pro PC), der Befehl (nicht das Token) von „GitHub in der VM“ für jede Sandbox im Anmeldedatenspeicher des Systems, eine Verzeichnisverbindung `sboxd-<id>` und ein Ordner `gvisor` im System-Temp-Ordner (die Sockets des Daemons) und unter Ubuntu das AppArmor-Profil des Shims des Kits; Sandboxes, Festplatten, Images, Einstellungen und Logs bleiben in `data`. Die vollständige Liste steht in [PRIVACY §4](PRIVACY.de.md#4-auf-deinem-gerät-gespeicherte-daten).
- Der Ordner eines Projekts wird für Claude Code, Codex und Pi vertrauenswürdig, sobald du darin einen Agenten aus Atollo öffnest (die anderen Ordner in `projects\` nicht): Öffne einen Agenten nur in Projekten, denen du vertraust. Die Token der Agenten (`CLAUDE_CODE_OAUTH_TOKEN`, `GEMINI_API_KEY`, `LLAMA_API_KEY` der lokalen Modelle) gehen nur an die von Atollo gestarteten Agenten, nicht an die Aktionen der Projekte, Skripte, Terminals oder den Editor; der Schlüssel der lokalen Modelle steht außerdem in Pis `models.json` (`data\home\.pi\agent`, nur für deinen Benutzer lesbar). Der Editor fragt dagegen beim ersten Öffnen jedes Projekts und öffnet nicht vertrauenswürdige im „eingeschränkten Modus“; seine Terminals laufen nur in der VM des Projekts.
- Die Nutzungslimits der Agenten stammen aus nicht dokumentierten Schnittstellen der Anbieter und können nach einem ihrer Updates nicht mehr funktionieren.
- „Laufwerk entfernen“ sieht keine PC-Programme, die nur eine Datei im Ordner von Atollo geöffnet halten.

## Aufbau

```
Atollo\
  Atollo.exe      die gesamte Logik
  projects\       deine Projekte
  data\           heruntergeladene Programme, Einstellungen, Anmeldungen, Cache, temporäre Dateien
                  (die Einstellungen jedes Projekts: data\projects\<Projekt>\dev-settings.json)
```

Für eine Sicherung gibt es Einstellungen › Das Kit › **Kopie des Kits**: Sie exportiert `projects\` und `data\` in einen Ordner oder ein Zip, ohne `data\apps\`, `data\cache\` und `data\models\` (die neu heruntergeladen werden) und ohne die Images und Festplatten der VMs (jede Sandbox wird aus ihrem Kit neu angelegt), mit den Datenbanken für die Kopie angehalten, und importiert eine solche Kopie beim nächsten Start in ein Kit. Von Hand kopierst du dieselben Ordner bei geschlossenem Atollo. Die Kopie enthält die Passwörter des Kits: bewahre sie sicher auf.

## Deinstallation

Atollo verwendet keinen Installer, registriert keine Systemdienste oder geplanten Aufgaben und verändert weder die Windows-Registrierung noch den `PATH`:

1. Öffne die Seite **Laufwerk entfernen** (oder schließe die Anwendung), um alle von Atollo gestarteten Prozesse zu beenden, auch das Docker Sandboxes des Kits und seine microVMs, und eine aktive Antigravity-Sitzung nach `data\` zurückzuholen.
2. Lösche den Ordner mit `Atollo.exe`, `data\` und `projects\` (nachdem du die Projekte gesichert hast, die du behalten möchtest): Die Sandboxes, ihre Festplatten und die Einstellungen von Docker Sandboxes gehen mit.
3. Temporäre Ordner unter `%TEMP%` werden beim Schließen automatisch entfernt; falls du `gh auth login` manuell in einer Konsole ausgeführt hast (statt über die Seite **Einstellungen**), kannst du den entsprechenden Eintrag in der Windows-**Anmeldeinformationsverwaltung** löschen.
4. Was die VM außerhalb des Ordners lässt (siehe [Bekannte Grenzen](#bekannte-grenzen)), entfernst du bei Bedarf von Hand: die Docker-Anmeldung und die Einträge `com.docker.sandboxes` in der Anmeldeinformationsverwaltung (oder im Schlüsselbund des Systems), die Verzeichnisverbindung `sboxd-<id>` und den Ordner `gvisor` im System-Temp-Ordner, das AppArmor-Profil unter Ubuntu; die Windows Hypervisor Platform bleibt eingeschaltet, bis du sie ausschaltest.

## Befehlszeile

```
Atollo.exe                           öffnet das Fenster
Atollo.exe env [--project <Name>]    zeigt die Umgebung der Programme von Atollo
                                     (mit einem Projekt, mit seinen Versionen)
Atollo.exe components [--target <System>]
                                     listet Sprachen und Werkzeuge, mit den installierten
Atollo.exe versions [<id>]           die Versionen auf der Platte; mit einer id die herunterladbaren
Atollo.exe check                     prüft, ob die installierten Programme antworten
    --kit                            der Zustand des Kits: Laufwerk, Platz, Pfad, Rechte, Bibliotheken
    --files                          die Dateien der Programme: fehlend oder seit der Installation geändert
Atollo.exe prepare [Optionen]        installiert und aktualisiert die gewählten Komponenten
                                     und entfernt die nicht gewählten
    --only a,b                       nur diese (und was sie brauchen), entfernt nichts
    --skip a,b                       diese nicht
    --update-all                     auch die bereits installierten aktualisieren
    --force                          neu herunterladen
    --line id=x.y                    legt diese Version neben die anderen (wiederholbar)
    --target <System>                bereitet für ein anderes System vor: win32-x64, darwin-arm64,
                                     linux-x64, linux-arm64
Atollo.exe sync [--project <Ordner>] [--verbose]
                                     gleicht Anweisungen, Skills und MCP-Server der Agenten ab
Atollo.exe keyring restore|save      Anmeldung von Antigravity (von agy.cmd des Kits benutzt)
Atollo.exe serve <Ordner> <Port>     Dateiserver für eine statische Website
Atollo.exe models                    die lokalen Modelle: die des Katalogs und die heruntergeladenen
Atollo.exe models pull <Modell>      lädt eine Katalog-id oder ein Modell von Hugging Face herunter
                                     (Besitzer/Name:Quantisierung), setzt nach Abbruch fort
Atollo.exe models rm <id>            löscht ein heruntergeladenes Modell
Atollo.exe llm start|stop|status     der Server der lokalen Modelle (Chat und Agenten)
Atollo.exe diagnostics               erstellt das Zip für eine Problemmeldung (Geheimnisse maskiert)
Atollo.exe version                   zeigt die Version
Atollo.exe help                      zeigt die Hilfe
```

In den Terminals des Kits heißen dieselben Befehle `atollo …` (`atollo components`, `atollo check --kit`): ein Starter in `data/kit/bin`, der die exe ausführt, mit der das Kit läuft (`ATOLLO_EXE`).

In PowerShell hängst du `| Out-Host` an (zum Beispiel `.\Atollo.exe check | Out-Host`): `Atollo.exe` ist ein Fensterprogramm, und ohne Pipe wartet PowerShell nicht darauf und liest weder seine Ausgabe noch seinen Exit-Code.

## Bauen

Voraussetzungen: stabiles Rust (mindestens 1.93) mit der MSVC-Toolchain (Visual Studio Build Tools, Workload C++), Node.js LTS.

```
npm ci
npm run build
```

Die JavaScript-Abhängigkeiten liegen nur im Stammordner (`node_modules` und `package-lock.json`): `ui` ist ein npm-Workspace.

Die exe liegt in `target\release\Atollo.exe`. Es wird kein Installer erzeugt (`--no-bundle`).

Prüfungen vor einer Änderung:

```
npm run lint
npm test
npm -w ui run build
```

`npm run lint` führt `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint und Prettier aus; `npm test` führt `cargo test` und die Tests der Oberfläche aus; `npm -w ui run build` prüft die Typen und baut die Oberfläche. Einzelheiten (auf Englisch) in [docs/app/TESTING.md](app/TESTING.md). Wie der Code aufgebaut ist und wohin eine Änderung gehört (auf Englisch): [docs/app/ARCHITECTURE.md](app/ARCHITECTURE.md).

Entwicklung mit automatischem Neuladen: `npm run dev`.

## Wenn Windows die exe blockiert

Solange die Codesignierung für die heruntergeladene Version noch nicht aktiv ist: Zeigt SmartScreen „Der Computer wurde durch Windows geschützt“, wähle **Weitere Informationen** › **Trotzdem ausführen**. Wurde die exe aus dem Internet geladen: Rechtsklick › **Eigenschaften** › **Zulassen**. Auf Firmen-PCs mit AppLocker brauchst du die Erlaubnis der IT-Abteilung.

## Code signing policy

Free code signing on Windows provided by [SignPath.io](https://signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

- **Projektrollen (Committer, Reviewer und Approver):** [nabre](https://github.com/nabre)
- **Herkunft der Binärdateien:** Die ausführbaren Dateien der offiziellen Releases werden ausschließlich durch die automatisierte GitHub-Actions-Pipeline ([`.github/workflows/app.yml`](../.github/workflows/app.yml)) direkt aus dem öffentlichen Quellcode dieses Repositories gebaut und signiert, ohne manuelle Eingriffe an den Binärdateien.

## Privacy policy

Der vollständige Text, mit jeder Netzwerkverbindung und jeder Datei, die die App aufbewahrt, steht in **[Datenschutzerklärung und Nutzungsbedingungen](PRIVACY.de.md)**. Kurz gesagt:

- **Keine Telemetrie oder Datenerhebung:** Die Anwendung erhebt keine personenbezogenen Daten, Nutzungsstatistiken, Telemetriedaten oder Absturzberichte und kontaktiert keinerlei Server des Projektautors. Zudem deaktiviert sie die standardmäßige Telemetrie verwalteter Komponenten (wie PowerShell 7 und des VSCodium-Servers).
- **Lokale Speicherung:** Einstellungen, Projekte, lokale Datenbanken, Protokolle, Anmeldeinformationen sowie die Sandboxes und Festplatten der microVMs werden lokal im Ordner `data\` (und `projects\`) von Atollo auf dem Gerät des Nutzers gespeichert; die wenigen Ausnahmen (temporäre Dateien, Antigravitys Anmeldung während es läuft, die Docker-Anmeldung und die anderen Spuren von Docker Sandboxes) stehen in der Erklärung.
- **Docker Sandboxes:** Die VM braucht ein Docker-Konto, angemeldet mit `sbx login`. Atollo startet das `sbx` des Kits mit abgeschalteter Docker-Telemetrie (`SBX_NO_TELEMETRY=1`), abgeschaltetem Diagnose-Upload, abgeschalteter SSH-Agent-Weiterleitung und der Netzwerkrichtlinie `balanced`; `sbx` selbst spricht mit Docker gemäß der [Datenschutzerklärung von Docker](https://www.docker.com/legal/docker-privacy-policy/).
- **Netzwerkverbindungen:** Die Anwendung selbst verbindet sich nur mit GitHub, um nach eigenen Updates zu suchen (Einstellungen › Allgemein › Automatisches Update, oder `ATOLLO_NO_UPDATE`), mit den offiziellen Quellen der Komponenten, die sie herunterlädt, mit der Open-VSX-Registry für Editor-Erweiterungen und – wenn der Nutzer sich bei GitHub anmeldet (Einstellungen › GitHub-Konto) oder die Seite KI öffnet – mit GitHub und den Anbietern der Agenten (**Claude Code**, **Codex**, **Antigravity** und dem in **Pi** gewählten Cloud-Anbieter) gemäß deren eigenen Datenschutzrichtlinien. Lokale Modelle kommen von Hugging Face (`huggingface.co` und sein CDN), die Programme llama.cpp und Pi aus ihren GitHub-Releases (`ggml-org/llama.cpp`, `earendil-works/pi`); ein lokales Modell läuft auf dem PC, und seine Prompts verlassen ihn nie.

## Lizenz

MIT: bereitgestellt „wie besehen“, ohne Gewährleistung. Siehe [LICENSE](../LICENSE) und die [Nutzungsbedingungen](PRIVACY.de.md#8-nutzungsbedingungen).
