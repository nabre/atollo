# Atollo — Universal Portable Dev & AI Workspace

[Italiano](../README.md) · [English](README.en.md) · [Français](README.fr.md) · Deutsch

**Atollo** ist ein eigenständiger *Universal Portable Dev & AI Workspace* (`Atollo.exe`), der **Editor** (integriertes VSCodium), **Toolchain-/Runtime-Manager** (über 20 bei Bedarf heruntergeladene Sprachen und Werkzeuge), **lokale Server/Datenbanken** (MySQL, MongoDB, SQLite, Caddy/Apache) und **KI-Agenten-Orchestrierung** (Claude Code, Codex und Antigravity mit synchronisierten Anweisungen, Skills und MCP-Servern) in einem einzigen portablen Ordner vereint. Keine Installation, keine Administratorrechte, keine Systemdienste außerhalb des Workspace-Ordners.

Beim ersten Start lädt die App die Systemkomponenten selbst herunter: PowerShell 7, GitHub CLI und den Editor. Alles andere wird auf der Seite „Werkzeuge“ gewählt oder kommt, wenn es gebraucht wird: Node.js mit einer `npm`-Aktion, uv mit einem `uvx`-MCP-Server und so weiter.

> Auf Italienisch: Anleitung [app/GUIDA.md](app/GUIDA.md), Entscheidungen [app/DECISIONS.md](app/DECISIONS.md), Neuigkeiten [app/CHANGELOG.md](app/CHANGELOG.md); offene Arbeit: [Issues](https://github.com/nabre/atollo/issues).

## Verwendung

1. Die exe von Atollo (zum Beispiel `Atollo-0.2.0-win32-x64.exe`) in einen leeren Ordner kopieren: einen USB-Stick, eine externe Festplatte oder einen Ordner im eigenen Profil (Leerzeichen, Klammern oder `&` im Namen sind kein Problem).
2. Doppelklicken: die App legt `projects\` und `data\` an. Beim ersten Start Agenten, Sprachen und Werkzeuge wählen: die App lädt sie herunter und prüft sie (Prüfsumme oder Signatur).
3. Auf der Seite **Projekte** ein Projekt anlegen oder klonen. Editor, Terminal und Agenten öffnen sich immer im Ordner eines Projekts, nie anderswo.

Von der [Release-Seite](https://github.com/nabre/atollo/releases): `Atollo-<Version>-win32-x64.exe` oder `Atollo-<Version>-win32-arm64.exe` für Windows, `Atollo-<Version>-darwin-arm64.app.zip` (Apple Silicon) oder `Atollo-<Version>-darwin-x64.app.zip` (Intel) für macOS, `Atollo-<Version>-linux-x64` oder `Atollo-<Version>-linux-arm64` (`.AppImage` und `.tar.gz`) für Linux; jede Datei hat ihre `.sha256`. Der Name nennt die Version: Wenn sich die App selbst aktualisiert, bekommen die exe und die AppImage den Namen der neuen Version. In den Beispielen unten steht `Atollo.exe` für Ihre exe.

**macOS** (12 oder neuer). Die App hat nur eine Ad-hoc-Signatur, ohne Notarisierung durch Apple:
1. Das Zip entpacken und `Atollo.app` mit dem Finder in den Ordner des Kits verschieben (zum Beispiel `/Volumes/KIT/Atollo/`).
2. Im Terminal die Quarantäne des Browsers entfernen: `xattr -dr com.apple.quarantine /Volumes/KIT/Atollo/Atollo.app`.
3. `Atollo.app` aus diesem Ordner öffnen: `projects/` und `data/` entstehen daneben, nie darin.

Ohne Schritt 2 startet macOS die App aus einer temporären Kopie (App Translocation), und Atollo hält mit demselben Befehl an, statt das Kit in einem Ordner anzulegen, der wieder verschwindet.

**Linux** (x64 und ARM64, glibc 2.35 oder neuer: Ubuntu 22.04, Debian 12 und neuer). `Atollo-<Version>-linux-<arch>.AppImage` in den Ordner des Kits kopieren, `chmod +x Atollo-<Version>-linux-<arch>.AppImage` und starten: das Kit ist der Ordner der AppImage. Ohne FUSE (Container, manche minimalen Distributionen): `./Atollo-<Version>-linux-<arch>.AppImage --appimage-extract-and-run` ausführen, oder `Atollo-<Version>-linux-<arch>.tar.gz` mit installiertem WebKitGTK 4.1 verwenden (auf Ubuntu und Debian `sudo apt install libwebkit2gtk-4.1-0`): `Atollo` in den Ordner des Kits entpacken und `./Atollo` starten. Die AppImage aktualisiert sich selbst; das `tar.gz` meldet, wenn eine neue Version zum Herunterladen bereitsteht.

## Seiten

| Seite | Inhalt |
| --- | --- |
| **Projekte** | die Projekte mit Starten, Editor, Terminal und Agent; **Neues Projekt**: leer, aus deinen GitHub-Repositories (mit Klonen, Umbenennen und Löschen auf GitHub) oder von einer Git-Adresse; Hinweise zum Workspace. Jedes Projekt hat eine eigene Seite: **Übersicht** (die Hauptaktion mit ihrer Ausgabe, Sprachen, Datenbanken, Agent, Git), **Ressourcen** (Komponenten, die Datenbank des Projekts mit Benutzer und Passwort, Editor-Erweiterungen), **Erweitert** (Aktionen, Ports, statische Website, Git, Anweisungen der Agenten) |
| **Agenten** | die installierten Claude Code, Codex und Antigravity: Version, Anmeldung, Tariflimits und Verbrauch von heute und den letzten 7 Tagen; Anweisungen, Skills und MCP-Server gemeinsam in `data\agents` |
| **Datenbanken** | die MySQL- (mit phpMyAdmin) und MongoDB-Server von Atollo: Starten und Stoppen, Administratorpasswort, Port, Daten in `data\db`, nur auf diesem PC; SQLite ohne Server |
| **Werkzeuge** | Sprachen und Werkzeuge mit Logo, Abhängigkeiten und Speicherplatz; installieren, aktualisieren, entfernen; die Matrix der Editor-Erweiterungen |
| **Einstellungen** | Sprache, Terminal, Netzwerk (Proxy, Zertifikate, lokales Netz), Gemini-Schlüssel, Versionen von Python und PHP, Rust-Compiler (GNU oder MSVC), Updates der App; das **GitHub-Konto** von Atollo, eines für alles (App, Git, Editor, Konsolen, Agenten, GitHub CLI); wo der Ordner von Atollo liegt |
| **Laufwerk entfernen** | schließt alles, was Atollo geöffnet hat, holt die Anmeldung von Antigravity nach `data\` zurück und lässt das Laufwerk auswerfen |

Unten auf jeder Seite das Feld **Aktivität**: was läuft oder installiert wird (Projekte, Datenbanken, Komponenten), mit Ausgabe und Fehlern an einem Ort.

Die Konsole von Atollo ist die klassische Windows-Konsole mit integrierter PowerShell 7 und ausschließlich der isolierten Umgebung von Atollo: auf dem Host-PC installierte Programme stören nicht.

## Noch zu tun

Atollo ist in der Alpha (Version 0.2.0). Windows x64 ist das am meisten getestete System, echte Installationen in der CI eingeschlossen; macOS, Linux und Windows ARM64 werden nur in der CI gebaut und getestet. Die vollständige Checkliste steht in [app/TESTING.md](app/TESTING.md) (auf Italienisch).

- **Das erste öffentliche Release.** Auf GitHub ist noch kein Release veröffentlicht: Die Dateien für die sechs Systeme kommen mit dem ersten Tag. Bis dahin wird Atollo aus dem Quellcode gebaut ([Bauen](#bauen)).
- **Alpha-Tests.** Gesucht sind Menschen, die Atollo in echten Projekten und auf verschiedenen PCs nutzen und melden, was nicht klappt: Installation der Komponenten, Projekte, Datenbanken, Agenten.
- **macOS auf echten Macs** ([#5](https://github.com/nabre/atollo/issues/5), [#6](https://github.com/nabre/atollo/issues/6)). Apple Silicon und Intel werden in der CI gebaut und gestartet, nie auf einem frischen Mac: das im Browser geladene Zip, Quarantäne und Gatekeeper, App Translocation, ein Kit auf einem exFAT-Stick, die Anmeldungen der Agenten von einem Mac zum anderen mitgenommen.
- **Linux auf echten Rechnern** ([#5](https://github.com/nabre/atollo/issues/5)). x64 und ARM64 auf Ubuntu 24.04, Fedora 42 und Debian 13: AppImage mit und ohne FUSE, das `tar.gz` mit WebKitGTK 4.1, exFAT-Sticks oder mit `noexec` eingehängte Sticks, Anmeldung mit GNOME, KeePassXC oder KWallet.
- **Updates und Apache außerhalb von Windows.** Das Update der App von einer Version zur nächsten ist auf macOS und Linux noch nicht ausprobiert. Apache kommt auf diese Systeme, sobald sein Build für das Kit veröffentlicht ist: Bis dahin laufen Websites mit Caddy.
- **Code-Signatur.** Auf macOS nur eine Ad-hoc-Signatur, ohne Notarisierung durch Apple: `xattr -dr com.apple.quarantine` ist nötig. Auf Windows muss die SignPath-Signatur beim ersten Release noch bestätigt werden; ohne sie kann SmartScreen beim ersten Start warnen.

Du hast einen Mac, einen Linux-PC oder einen PC mit Windows ARM64? Probier Atollo aus und erzähl uns, wie es lief: [Issue auf GitHub öffnen](https://github.com/nabre/atollo/issues).

## Bekannte Grenzen

- Auf Windows 10 und 11 (`x64` und `ARM64`), macOS (Apple Silicon und Intel) und Linux (`x64` und `ARM64`). Auf Windows ARM64 laufen Editor, PHP, Apache, MySQL, MongoDB, Android SDK, Julia, TinyTeX und R in x64-Emulation; auf Linux ARM64 fehlt Android. Die Releases für macOS und Linux, ab dem nächsten Tag, sind in der CI getestet, aber noch nicht auf sauberen Rechnern ([#5](https://github.com/nabre/atollo/issues/5)), und die Anmeldung der Agenten muss noch auf einem Mac getestet werden ([#6](https://github.com/nabre/atollo/issues/6)); Komponenten und Quellen pro System in [app/CATALOG.md](app/CATALOG.md) (auf Italienisch), die Matrix Komponente × System in [app/MATRICE-STRUMENTI.html](app/MATRICE-STRUMENTI.html). Apache auf macOS und Linux ist ein eigener Build des Kits (D-116), mit PHP über `php-fpm`.
- Unter Linux braucht MySQL zwei Systembibliotheken, `libaio` und `libnuma`: auf Ubuntu 24.04 und Debian 13 `sudo apt install libaio1t64 libnuma1` (auf älteren Versionen `libaio1`), auf Fedora und RHEL `sudo dnf install libaio numactl-libs`. Fehlen sie, sagt Atollo es mit dem auszuführenden Befehl.
- Unter macOS und Linux funktioniert ein Atollo-Ordner auf exFAT oder FAT (ein für Windows formatiertes Laufwerk), aber diese Dateisysteme kennen keine Links: Wo ein Programm welche mitbringt, kopiert Atollo die Dateien (etwa die 142 internen Links von Git), was mehr Platz belegt. Auf einem Laufwerk nur für Mac oder Linux ist APFS oder ext4 besser.
- Außerhalb des Atollo-Ordners bleiben nur die Spuren von Windows, temporäre Übergangsdateien im System-Temp-Ordner (werden beim Beenden automatisch gelöscht), die Anmeldung von GitHub CLI in der Anmeldeinformationsverwaltung, wenn Sie sich in einer Konsole mit `gh auth login` anmelden (die in den Einstellungen bleibt in `data\gh`), und die von Antigravity, solange `agy` geöffnet ist (nur für die Sitzung).
- Die Ordner in `projects\` gelten für Claude Code und Codex als vertrauenswürdig: nur Projekte öffnen, denen man vertraut. Der Editor fragt dagegen beim ersten Öffnen jedes Projekts und öffnet nicht vertrauenswürdige im „eingeschränkten Modus“.
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

Für eine Sicherung genügt es, `projects\` und `data\` zu kopieren (ohne `data\apps\` und `data\cache\`, die neu heruntergeladen werden).

## Deinstallation

Atollo verwendet keinen Installer, registriert keine Systemdienste oder geplanten Aufgaben und verändert weder die Windows-Registrierung noch den `PATH`:

1. Öffnen Sie die Seite **Laufwerk entfernen** (oder schließen Sie die Anwendung), um alle von Atollo gestarteten Prozesse zu beenden und eine aktive Antigravity-Sitzung nach `data\` zurückzuholen.
2. Löschen Sie den Ordner mit `Atollo.exe`, `data\` und `projects\` (nachdem Sie Projekte, die Sie behalten möchten, gesichert haben).
3. Temporäre Ordner unter `%TEMP%` werden beim Schließen automatisch entfernt; falls Sie `gh auth login` manuell in einer Konsole ausgeführt haben (statt über die Seite **Einstellungen**), können Sie den entsprechenden Eintrag in der Windows-**Anmeldeinformationsverwaltung** löschen.

## Befehlszeile

```
Atollo.exe                           öffnet das Fenster
Atollo.exe env                       zeigt die Umgebung der Programme von Atollo
Atollo.exe components                listet Sprachen und Werkzeuge, mit den installierten
Atollo.exe check                     prüft, ob die installierten Programme antworten
Atollo.exe prepare [Optionen]        installiert, aktualisiert und entfernt die gewählten Komponenten
    --only a,b                       nur diese (und was sie brauchen)
    --skip a,b                       diese nicht
    --update-all                     auch die bereits installierten aktualisieren
    --force                          neu herunterladen
Atollo.exe sync [--project <Ordner>] [--verbose]
                                     gleicht Anweisungen, Skills und MCP der drei Agenten ab
Atollo.exe keyring restore|save      Anmeldung von Antigravity (von agy.cmd benutzt)
Atollo.exe serve <Ordner> <Port>     Dateiserver für eine statische Website
Atollo.exe version                   zeigt die Version
Atollo.exe help                      zeigt die Hilfe
```

In PowerShell `| Out-Host` anhängen (zum Beispiel `.\Atollo.exe check | Out-Host`): `Atollo.exe` ist ein Fensterprogramm, und ohne Pipe wartet PowerShell nicht darauf und liest weder seine Ausgabe noch seinen Exit-Code.

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

`npm run lint` führt `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint und Prettier aus; `npm test` führt `cargo test` und die Tests der Oberfläche aus; `npm -w ui run build` prüft die Typen und baut die Oberfläche. Einzelheiten (auf Italienisch) in [app/TESTING.md](app/TESTING.md). Wie der Code aufgebaut ist und wohin eine Änderung gehört (auf Italienisch): [app/ARCHITECTURE.md](app/ARCHITECTURE.md).

Entwicklung mit automatischem Neuladen: `npm run dev`.

## Wenn Windows die exe blockiert

Solange die Codesignierung für die heruntergeladene Version noch nicht aktiv ist: Zeigt SmartScreen „Der Computer wurde durch Windows geschützt“, **Weitere Informationen** › **Trotzdem ausführen** wählen. Wurde die exe aus dem Internet geladen: Rechtsklick › **Eigenschaften** › **Zulassen**. Auf Firmen-PCs mit AppLocker ist die Erlaubnis der IT-Abteilung nötig.

## Code signing policy

Free code signing on Windows provided by [SignPath.io](https://signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

- **Projektrollen (Committer, Reviewer und Approver):** [nabre](https://github.com/nabre)
- **Herkunft der Binärdateien:** Die ausführbaren Dateien der offiziellen Releases werden ausschließlich durch die automatisierte GitHub-Actions-Pipeline ([`.github/workflows/app.yml`](../.github/workflows/app.yml)) direkt aus dem öffentlichen Quellcode dieses Repositories gebaut und signiert, ohne manuelle Eingriffe an den Binärdateien.

## Privacy policy

Atollo (`Atollo.exe`) respektiert die Privatsphäre der Nutzer vollständig:

- **Keine Telemetrie oder Datenerhebung:** Die Anwendung erhebt keine personenbezogenen Daten, Nutzungsstatistiken, Telemetriedaten oder Absturzberichte und kontaktiert keinerlei Server des Projektautors. Zudem deaktiviert sie die standardmäßige Telemetrie verwalteter Komponenten (wie PowerShell 7 und des VSCodium-Servers).
- **Lokale Speicherung:** Einstellungen, Projekte, lokale Datenbanken, Protokolle und Anmeldeinformationen werden ausschließlich lokal im Ordner `data\` (und `projects\`) von Atollo auf dem Gerät des Nutzers gespeichert.
- **Netzwerkverbindungen:** Die Anwendung stellt Netzwerkverbindungen nur zu offiziellen Upstream-Quellen her, um nach Updates zu suchen und vom Nutzer angeforderte Komponenten herunterzuladen (`api.github.com`, offizielle Sprach-/Werkzeug-Mirrors, Open-VSX-Registry für Editor-Erweiterungen), sowie – wenn der Nutzer die GitHub-Integration, KI-Agenten (**Claude Code**, **Codex**, **Antigravity**) oder einen Gemini-API-Schlüssel einrichtet und nutzt – direkt zu den jeweiligen Anbieter-APIs gemäß deren eigenen Datenschutzrichtlinien.

## Lizenz

MIT.
