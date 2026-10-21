# Atollo: Datenschutzerklärung und Nutzungsbedingungen

[English](PRIVACY.md) · [Italiano](PRIVACY.it.md) · [Français](PRIVACY.fr.md) · Deutsch

Gültig ab: 21. Oktober 2026

Diese Seite erklärt, was die Desktop-App Atollo (`Atollo.exe`, die macOS-`.app` und das Linux-AppImage) und ihre Website mit deinen Daten tun. Atollo ist Open Source: Alles, was hier steht, lässt sich im Code dieses Repositorys überprüfen.

## 1. Kurz gesagt

- Atollo hat **keine Telemetrie, keine Nutzungsstatistiken und keine Absturzberichte**. Es gibt kein Konto und keinen eigenen Server.
- Alles, was Atollo speichert, bleibt **im Atollo-Ordner** auf deinem Gerät (`data/` und `projects/`), mit den wenigen Ausnahmen aus Abschnitt 4.
- Atollo verbindet sich nur mit dem Netz, um **nach eigenen Updates zu suchen**, um **die Werkzeuge, die du nutzt, von ihren offiziellen Quellen herunterzuladen**, und um **mit GitHub und den Anbietern der KI-Agenten zu sprechen, wenn du diese Funktionen nutzt**. Die Tabelle in Abschnitt 3 listet jede Verbindung und wie du sie abschaltest.
- Die Werkzeuge und KI-Agenten, die du in Atollo startest (Claude Code, Codex, Antigravity, Git, npm…), sind Programme Dritter mit eigenen Datenschutzerklärungen.

## 2. Was Atollo nicht erhebt

Atollo erhebt, sendet und verkauft nicht:

- personenbezogene Daten, Namen, E-Mail-Adressen oder Gerätekennungen;
- Nutzungsstatistiken, Klicks, Seitenaufrufe oder genutzte Funktionen;
- Absturzberichte oder Protokolle;
- den Inhalt deiner Projekte, Dateien, Datenbanken oder Unterhaltungen mit KI-Agenten.

Niemand, der das Projekt Atollo betreibt, erhält Daten von der App. Das Projekt betreibt keine Server: Updates und Releases liegen auf GitHub.

Atollo schaltet außerdem in seiner eigenen Umgebung die Telemetrie und Update-Prüfungen mehrerer Programme ab, die es verwaltet: Der integrierte Editor (VSCodium) startet mit abgeschalteter Telemetrie und ohne automatische Erweiterungs-Updates; die Umgebung setzt `GOTELEMETRY=off`, `DOTNET_CLI_TELEMETRY_OPTOUT=1`, `POWERSHELL_TELEMETRY_OPTOUT=1`, `POWERSHELL_UPDATECHECK=Off`, `PIP_DISABLE_PIP_VERSION_CHECK=1`, `GH_NO_UPDATE_NOTIFIER=1` (für die Aufrufe der App an GitHub CLI), `DISABLE_UPDATES=1` (Claude Code) und `AGY_CLI_DISABLE_AUTO_UPDATE=true` (Antigravity). Andere Programme behalten ihre Voreinstellungen.

## 3. Die Netzwerkverbindungen der App

Jede Anfrage, die die App selbst stellt, trägt einen `User-Agent` der Form `Atollo/<Version> (+https://github.com/nabre/atollo)` und gibt, wie jede Internetverbindung, deine IP-Adresse an den kontaktierten Server preis. Die Anfragen nutzen den Proxy aus den Einstellungen, sonst `HTTPS_PROXY`/`HTTP_PROXY`, sonst den Proxy des Systems.

| Wann | An wen | Was gesendet wird | Wie du sie abschaltest |
|---|---|---|---|
| Etwa 20 Sekunden nach dem Start und wenn du „Nach Updates suchen“ drückst | GitHub (`api.github.com`, dann der Release-Download von `github.com`) | Die Abfrage der Release-Liste von `nabre/atollo`; gibt es eine neuere Version, deren Download. Ist auf dem Computer eine Variable `GITHUB_TOKEN` oder `GH_TOKEN` gesetzt, wird sie nur an `api.github.com` gesendet (gegen GitHubs Anfragelimit). | Einstellungen › Allgemein › **Automatisches Update** aus, oder die Variable `ATOLLO_NO_UPDATE` auf dem Computer setzen. Der Knopf funktioniert weiterhin auf Anfrage. |
| Etwa 10 Sekunden nach dem Start, höchstens einmal am Tag pro Werkzeug | Die offizielle Quelle jedes Werkzeugs mit mehreren Versionen (zum Beispiel `nodejs.org`, `go.dev`, `api.adoptium.net`, `api.github.com`) | Die Abfrage der verfügbaren Versionen. Nur an `api.github.com`: das Token oben oder das deines GitHub-Kontos, wenn du angemeldet bist (Abschnitt 4). | Die Variable `ATOLLO_NO_UPDATE`. |
| Beim ersten Start, wenn du die Startseite bestätigst (nach der Wahl von Sprache und Netzwerk); danach automatisch, wenn eine erforderliche Komponente fehlt oder ein App-Update neuere braucht | Offizielle Quellen von PowerShell 7, GitHub CLI und dem Editor (Releases auf GitHub) | Download-Anfragen. | Kein Schalter: Diese Komponenten sind erforderlich. Sonst wird nichts ohne dein Zutun installiert. |
| Wenn du ein Werkzeug installierst oder aktualisierst (Seite Werkzeuge) oder eine Aktion eines braucht (zum Beispiel Node.js für `npm`) | Die offizielle Quelle des Werkzeugs, zum Beispiel `nodejs.org`, `registry.npmjs.org`, `go.dev`, `static.rust-lang.org`, `api.adoptium.net`, `repo.maven.apache.org`, `services.gradle.org`, `builds.dotnet.microsoft.com`, `windows.php.net`, `getcomposer.org`, `cdn.mysql.com`, `downloads.mongodb.org`, `sqlite.org`, `files.phpmyadmin.net`, `conda.anaconda.org`, `api.anaconda.org`, `dl.google.com`, `storage.googleapis.com`, `raw.githubusercontent.com`, `www.apachelounge.com`, `dl.static-php.dev`, `curl.se`, `aka.ms`, `api.nuget.org`, `julialang-s3.julialang.org`, `ziglang.org`, Googles Update-Server für Antigravity, Releases auf GitHub | Download-Anfragen; Prüfsummen werden lokal geprüft. | Das Werkzeug nicht installieren. |
| Im Hintergrund nach dem Start des Editors und beim Öffnen eines Projekts | Die Erweiterungsgalerie des Editors (Open VSX); CodeLLDB aus seinem GitHub-Release | Die Anfragen der Erweiterungen, die das Kit und deine Projekte nutzen. | Den Editor nicht öffnen oder die Erweiterungen im Editor entfernen. |
| Nur wenn du dich bei GitHub anmeldest (Einstellungen › GitHub-Konto) und deine Repositorys nutzt (Projekte › „Aus deinen GitHub-Repositories“) | GitHub (`github.com`, `api.github.com`), über GitHub CLI | Die Anmeldung (ein Einmalcode, den du auf `github.com/login/device` eingibst), dann das Token deines Kontos mit den Anfragen zum Auflisten, Klonen, Umbenennen oder Löschen deiner Repositorys. | Nicht anmelden oder in Einstellungen › GitHub-Konto abmelden. |
| Einmal, bei der Anmeldung oder einem Start mit Anmeldung, nur wenn Name und E-Mail der Commits leer sind | GitHub (`api.github.com/user`), über GitHub CLI | Das Token deines Kontos, um Name, Login und Nummer des Kontos zu lesen: Sie werden der Name deiner Commits und die private `noreply`-Adresse von GitHub. | Einstellungen › GitHub-Konto › „Wer die Commits signiert“ ausfüllen, oder nicht anmelden. |
| Nur solange die Seite Agenten angezeigt wird (beim Öffnen, dann alle 2 Minuten), für jeden Agenten, bei dem du angemeldet bist | Claude Code: `api.anthropic.com`; Codex: `chatgpt.com`; Antigravity: `cloudcode-pa.googleapis.com` | Das Anmeldetoken des Agenten, nur an dessen Anbieter gesendet, um die Nutzungslimits deines Tarifs zu lesen. Das Token wird nur gelesen, nie erneuert und nie ins Protokoll geschrieben. | Die Seite Agenten nicht öffnen oder dich beim Agenten nicht anmelden. |

**Keine** Verbindungen von Atollo selbst sind:

- Programme, die du im Editor, in den Konsolen oder über Aktionen startest (`git`, `npm`, `pip`, `cargo`, die KI-Agenten…), verbinden sich dorthin, wofür sie gebaut sind;
- Links, die du anklickst, öffnen sich in deinem Browser.

Lokale Server: Der Editor und die Server der Projekte lauschen nur auf diesem Computer (`127.0.0.1`/`localhost`), außer du schaltest in den Einstellungen **Lokales Netzwerk** für die Server der Projekte ein.

## 4. Auf deinem Gerät gespeicherte Daten

Atollo ist portabel: Es hält seine Daten neben der ausführbaren Datei im Ordner `data/` und deine Projekte in `projects/`. Nichts wird hochgeladen. Wer den Atollo-Ordner löscht, löscht alles.

| Was | Wo (im Atollo-Ordner) |
|---|---|
| Einstellungen: Sprache, Design, Proxy, Zertifikate, automatisches Update, Rust-Compiler | `data/app/settings.json` |
| Die Einstellungen von Git, mit dem Namen und der E-Mail, die deine Commits signieren | `data/home/.gitconfig` |
| Der **Gemini-API-Schlüssel**, falls du einen einträgst (nur für deinen Benutzer lesbar, siehe unten) | `data/app/secrets/gemini-api-key` |
| Die Stammzertifikate des Systems plus deine zusätzliche PEM-Datei, nur wenn du eine festlegst | `data/app/ca-bundle.pem` |
| Protokolle der App (Geheimnisse maskiert), rotiert ab 1 MB | `data/app/logs/` |
| Installierte Werkzeuge und ihre Versionen | `data/apps/`, `data/app/versions/` |
| Zwischengespeicherte Antworten der GitHub-API | `data/cache/github/` |
| Speicher der Oberfläche (Fensteraufteilung) und Cache der Web-View | `data/webview2/` (Windows und Linux) |
| GitHub-Konto (Token von GitHub CLI, als Datei gespeichert, damit es mit dem Kit reist) | `data/gh/` |
| Git-Anmeldedaten für andere Hosts (Git Credential Manager, mit Windows-DPAPI für deinen Benutzer verschlüsselt) | `data/home/.gcm/` (Windows) |
| Anmeldungen und Sitzungen der KI-Agenten, von den Agenten selbst geschrieben | `data/home/.claude/`, `data/home/.codex/`, `data/home/.gemini/` |
| Antigravitys Google-Anmeldung, nach jedem Lauf aus dem Anmeldespeicher des Systems entfernt | `data/antigravity/keyring.json` |
| Das langlebige Token von Claude Code, wenn du „Mit einem Token verbinden“ nutzt (macOS) | `data/agents/claude/oauth-token` |
| Datenbank-Passwörter (zufällig, von Atollo erzeugt) | `data/app/db/secrets.json`, `data/app/db/mysql.cnf`, `data/projects/<Projekt>/private/db.json` |
| Daten des Editor-Servers und sein Zugriffstoken pro Sitzung | `data/vscodium-server/` |
| Deine Projekte | `projects/` |

Die Datenbank-Passwörter, der Gemini-API-Schlüssel und die Datei des Claude-Tokens werden nur für deinen Benutzer lesbar angelegt, wo das Laufwerk Berechtigungen kennt (NTFS, APFS, ext4…). Die übrigen Dateien, `settings.json` eingeschlossen, sind gewöhnliche Dateien. Auf USB-Sticks mit FAT32 oder exFAT gibt es keine Berechtigungen: **Wer den Stick hat, kann alle oben genannten Dateien lesen**. Bewahre den Stick sicher auf oder nutze ein Laufwerk mit Berechtigungen.

Das Claude-Token und der Gemini-API-Schlüssel werden (als `CLAUDE_CODE_OAUTH_TOKEN` und `GEMINI_API_KEY`) nur den von Atollo gestarteten Agenten übergeben, nicht den Aktionen der Projekte, Skripten, Terminals oder dem Editor.

Ausnahmen, außerhalb des Atollo-Ordners:

- **Temporäre Dateien**: Dateien in Bewegung landen in `<System-Temp>/Atollo/`; je nach Einstellungen › Allgemein › **Temporäre Dateien der Programme** können auch die Programme den temporären Ordner des Systems nutzen (standardmäßig, wenn das Kit auf einem USB-Stick, einem Netzlaufwerk oder unter FAT32/exFAT liegt).
- **Anmeldung von Antigravity**: Während Antigravity läuft, liegt seine Anmeldung im Anmeldespeicher des Systems (unter Windows nur für die Anmeldesitzung), danach kommt sie zurück ins Kit und wird dort entfernt. Anmeldedaten, die schon auf dem Computer waren, werden wiederhergestellt; eine versiegelte Notiz (`keyring-before.json`, nur auf diesem Computer lesbar) bleibt bis zur nächsten Sicherung im Kit.
- **Claude Code unter macOS** hält seine Anmeldung im Schlüsselbund, außer du nutzt „Mit einem Token verbinden“.
- **Web-View unter macOS**: Unter macOS speichert die System-Web-View ihre Daten dort, wo macOS es festlegt (in der Library deines Benutzers).
- **rustup** schreibt bei jeder Installation einen Eintrag in die Windows-Registry; Atollo entfernt ihn.

## 5. Werkzeuge und KI-Agenten Dritter

Atollo installiert und startet Programme anderer Hersteller. Wenn du sie nutzt, sprechen sie direkt mit ihren eigenen Diensten, und **es gelten deren Datenschutzerklärungen und Bedingungen**. Atollo leitet diesen Verkehr nicht weiter, liest ihn nicht und speichert ihn nicht, abgesehen von den Sitzungen und Anmeldungen, die diese Programme selbst in den Ordner des Kits schreiben (Abschnitt 4).

- **Claude Code** (Anthropic): [anthropic.com/legal/privacy](https://www.anthropic.com/legal/privacy)
- **Codex** (OpenAI): [openai.com/policies/privacy-policy](https://openai.com/policies/privacy-policy/)
- **Antigravity** und die Gemini-API (Google): [policies.google.com/privacy](https://policies.google.com/privacy)
- **GitHub** und GitHub CLI: [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
- **VSCodium** und Open VSX: [open-vsx.org](https://open-vsx.org/)
- Die Paket-Registries (npm, PyPI, crates.io, Maven Central, NuGet…) und die Download-Seiten der einzelnen Werkzeuge, nach ihren eigenen Regeln.

Die KI-Agenten senden deine Prompts und die Dateien, die sie lesen, an ihre Anbieter: So funktionieren sie. Entscheide selbst, was du mit ihnen öffnest.

## 6. Website

Die Website [nabre.github.io/atollo](https://nabre.github.io/atollo/de/) ist eine statische Seite auf GitHub Pages.

- Keine Statistiken, kein Tracking, keine Cookies, keine Werbung. Schriften und Bilder kommen von der Website selbst.
- Die Website merkt sich das gewählte Design und die Sprache im `localStorage` deines Browsers (`atollo-theme`, `atollo-lang`); sie verlassen den Browser nicht.
- Damit der Download-Knopf auf die richtige Datei zeigt, fragt dein Browser `api.github.com` nach der Liste der Atollo-Releases.
- GitHub kann als Host technische Daten wie IP-Adressen protokollieren: siehe das [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).
- Die Website ist mit einer Verifizierungsdatei in der Google Search Console bestätigt; das fügt für Besucher kein Tracking hinzu.

## 7. Kinder

Atollo ist ein Werkzeug für Entwickler und richtet sich nicht an Kinder. Es erhebt von niemandem Daten, Kinder eingeschlossen.

## 8. Nutzungsbedingungen

Atollo ist freie Software unter der [MIT-Lizenz](../LICENSE). Es wird **„wie besehen“, ohne jegliche Gewährleistung** bereitgestellt, weder ausdrücklich noch stillschweigend; die Autoren haften nicht für Ansprüche, Schäden oder sonstige Verpflichtungen aus seiner Nutzung. Du bist dafür verantwortlich, wie du es nutzt, und für die Einhaltung der Lizenzen und Bedingungen der Werkzeuge und Dienste Dritter, die es installiert oder mit denen es sich verbindet (Abschnitt 5).

## 9. Änderungen

Wenn sich diese Erklärung ändert, wird die neue Fassung in dieser Datei mit neuem Gültigkeitsdatum veröffentlicht; frühere Fassungen bleiben in der Historie des Repositorys.

## 10. Kontakt

Fragen oder Anliegen: eröffne ein Issue auf [github.com/nabre/atollo/issues](https://github.com/nabre/atollo/issues).
