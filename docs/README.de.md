# Atollo

[Italiano](../README.md) · [English](README.en.md) · [Français](README.fr.md) · Deutsch

Eine portable Entwicklungsumgebung für Windows in einer einzigen Datei: `Atollo.exe`. Integrierter Editor (VSCodium), die Agenten Claude Code, Codex und Antigravity, Sprachen und Werkzeuge, die bei Bedarf heruntergeladen werden. Keine Installation, keine Administratorrechte, keine Datei außerhalb des Kit-Ordners.

Beim ersten Start lädt die App die Systemkomponenten selbst herunter: PowerShell 7, GitHub CLI, den Editor und was diese brauchen (Git für GitHub CLI). Alles andere wird auf der Seite „Sprachen und Werkzeuge“ gewählt oder kommt, wenn es gebraucht wird: Node.js mit einer `npm`-Aktion, uv mit einem `uvx`-MCP-Server und so weiter.

> In Entwicklung. Stand und Plan (auf Italienisch): [app/PLAN.md](app/PLAN.md), Neuigkeiten: [app/CHANGELOG.md](app/CHANGELOG.md).
> Angelehnt an [nabre/atollo](https://github.com/nabre/atollo), aber keine neue Version davon: ein eigenständiges Produkt.

## Verwendung

1. `Atollo.exe` in einen leeren Ordner kopieren: einen USB-Stick, eine externe Festplatte oder einen Ordner im eigenen Profil (Leerzeichen, Klammern oder `&` im Namen sind kein Problem).
2. Doppelklicken: die App legt `projects\` und `data\` an. Beim ersten Start Agenten, Sprachen und Werkzeuge wählen: die App lädt sie herunter und prüft sie (Prüfsumme oder Signatur).
3. Auf der Seite **Projekte** ein Projekt anlegen oder klonen. Editor, Terminal und Agenten öffnen sich immer im Ordner eines Projekts, nie anderswo.

## Seiten

| Seite | Inhalt |
| --- | --- |
| **Start** | letzte Projekte mit Editor, Terminal und Agent; Hinweise (Kit verschoben, andere Kopie des Kits geöffnet, laufende Installationen); wo das Kit liegt |
| **Projekte** | neues Projekt, Klonen von GitHub oder einer Git-Adresse; für jedes Projekt Typ, Sprachen (mit Logo) und erkannte Aktionen (`npm run dev`, `php -S`, `cargo run`, statische Website…), mit Port, Ausgabe und Öffnen im Browser; jedes Projekt hat eine eigene Seite (Start, Datenbanken, Erweiterungen, Agent, Git) |
| **GitHub** | das GitHub-Konto des Kits, eines für alles: diese Seite, Git (auch im Editor, in den Konsolen und in den Agenten) und GitHub CLI; die Repositorys des Kontos und seiner Organisationen, mit Klonen nach `projects\`, Öffnen im Editor, Umbenennen und Löschen auf GitHub |
| **Agenten** | die installierten Claude Code, Codex und Antigravity: Version, Anmeldung, Tariflimits und Verbrauch von heute und der letzten 7 Tage; gemeinsame Anweisungen, Skills und MCP-Server in `data\agents` |
| **Sprachen und Werkzeuge** | die Komponenten mit Logo, Abhängigkeiten und Speicherplatz; installieren, aktualisieren, entfernen |
| **Datenbanken** | MariaDB (MySQL-kompatibel) und MongoDB: Starten und Stoppen, Verbindungszeichenfolge, Port, Daten in `data\db`, nur auf diesem PC; SQLite ohne Server |
| **Einstellungen** | Sprache, Terminal, Netzwerk (Proxy, Zertifikate, lokales Netz), Gemini-Schlüssel, Versionen von Python und PHP, Rust-Compiler (GNU oder MSVC), Updates der App |
| **Laufwerk entfernen** | schließt alles, was das Kit geöffnet hat, holt die Anmeldung von Antigravity ins Kit zurück und lässt den Stick auswerfen |

Die Konsole des Kits ist das portable Windows Terminal des Kits (oder die klassische Windows-Konsole, wenn sie in den Einstellungen gewählt ist oder Windows Terminal nicht installiert ist) mit der PowerShell 7 des Kits und nur der Umgebung des Kits: auf dem PC installierte Programme sind nicht sichtbar.

## Was vom Atollo bleibt

Das Konsolenmenü von [nabre/atollo](https://github.com/nabre/atollo) entfällt: jeder Eintrag hat eine Seite in der App („Proxy und Einstellungen“ ist die Seite Einstellungen), und die Logik der Skripte steckt in Rust in der exe. Die Komponenten sind dieselben (plus PowerShell 7 und der integrierte Editor, minus VSCodium Desktop), ebenso der Abgleich von Anweisungen, Skills und MCP zwischen den drei Agenten und das Umziehen, wenn das Kit Ordner oder Laufwerksbuchstaben wechselt. Kits des Atollo werden nicht importiert.

## Bekannte Grenzen

- Nur Windows 10 und 11, x64 und ARM64 (auf ARM64 laufen Editor, PHP, Apache und TinyTeX in x64-Emulation; R ist nicht verfügbar).
- Außerhalb des Kits bleiben nur die Spuren von Windows, die Anmeldung von GitHub CLI in der Anmeldeinformationsverwaltung, wenn Sie sich in einer Konsole mit `gh auth login` anmelden (die der GitHub-Seite bleibt im Kit, in `data\gh`), und die von Antigravity, solange `agy` geöffnet ist (nur für die Sitzung).
- Die Ordner in `projects\` gelten für Claude Code und Codex als vertrauenswürdig: nur Projekte öffnen, denen man vertraut. Der Editor fragt dagegen beim ersten Öffnen jedes Projekts und öffnet nicht vertrauenswürdige im „eingeschränkten Modus“.
- Die Nutzungslimits der Agenten stammen aus nicht dokumentierten Schnittstellen der Anbieter und können nach einem ihrer Updates nicht mehr funktionieren.
- „Laufwerk entfernen“ sieht keine PC-Programme, die nur eine Datei des Kits geöffnet halten.
- Die exe ist nicht signiert (siehe unten).

## Aufbau

```
Atollo\
  Atollo.exe     die gesamte Logik
  projects\       deine Projekte
  data\           heruntergeladene Programme, Einstellungen, Anmeldungen, Cache, temporäre Dateien
                  (die Kit-Einstellungen jedes Projekts: data\app\projects\<Projekt>\dev-settings.json)
```

Für eine Sicherung genügt es, `projects\` und `data\` zu kopieren (ohne `data\apps\` und `data\cache\`, die neu heruntergeladen werden).

## Befehlszeile

```
Atollo.exe                          öffnet das Fenster
Atollo.exe env                      zeigt die Umgebung der Programme des Kits
Atollo.exe components               listet Sprachen und Werkzeuge, mit den installierten
Atollo.exe check                    prüft, ob die installierten Programme antworten
Atollo.exe prepare [Optionen]       installiert, aktualisiert und entfernt die gewählten Komponenten
    --only a,b                       nur diese (und was sie brauchen)
    --skip a,b                       diese nicht
    --update-all                     auch die bereits installierten aktualisieren
    --force                          neu herunterladen
Atollo.exe sync [--project <Ordner>] [--verbose]
                                     gleicht Anweisungen, Skills und MCP der drei Agenten ab
Atollo.exe keyring restore|save     Anmeldung von Antigravity (von agy.cmd des Kits benutzt)
Atollo.exe serve <Ordner> <Port>    Dateiserver für eine statische Website
Atollo.exe version                  zeigt die Version
Atollo.exe help                     zeigt die Hilfe
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

`npm run lint` führt `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint und Prettier aus; `npm test` führt `cargo test` und die Tests der Oberfläche aus; `npm -w ui run build` prüft die Typen und baut die Oberfläche. Einzelheiten (auf Italienisch) in [app/TESTING.md](app/TESTING.md).

Entwicklung mit automatischem Neuladen: `npm run dev`.

## Wenn Windows die exe blockiert

Die exe ist noch nicht signiert. Zeigt SmartScreen „Der Computer wurde durch Windows geschützt“, **Weitere Informationen** › **Trotzdem ausführen** wählen. Wurde die exe aus dem Internet geladen: Rechtsklick › **Eigenschaften** › **Zulassen**. Auf Firmen-PCs mit AppLocker ist die Erlaubnis der IT-Abteilung nötig.

## Lizenz

MIT.
