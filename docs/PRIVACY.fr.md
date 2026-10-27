# Atollo : politique de confidentialité et conditions d'utilisation

[English](PRIVACY.md) · [Italiano](PRIVACY.it.md) · Français · [Deutsch](PRIVACY.de.md)

En vigueur depuis le : 27 octobre 2026

Cette page explique ce que l'application de bureau Atollo (`Atollo.exe`, la `.app` macOS et l'AppImage Linux) et son site font de vos données. Atollo est open source : tout ce qui est écrit ici peut être vérifié dans le code de ce dépôt.

## 1. En bref

- Atollo n'a **ni télémétrie, ni statistiques d'utilisation, ni rapports de plantage**. Il n'a ni compte ni serveur à lui.
- Tout ce qu'Atollo enregistre reste **dans le dossier d'Atollo** sur votre appareil (`data/` et `projects/`), à quelques exceptions près, listées à la section 4.
- Atollo ne se connecte au réseau que pour **rechercher ses propres mises à jour**, **télécharger les outils que vous utilisez depuis leurs sources officielles**, et dialoguer avec **GitHub et les fournisseurs des agents IA quand vous utilisez ces fonctions**. Le tableau de la section 3 liste chaque connexion et comment la désactiver.
- Les outils et agents IA que vous lancez dans Atollo (Claude Code, Codex, Antigravity, Pi, Git, npm…) sont des programmes tiers avec leurs propres politiques de confidentialité.
- Les **modèles d'IA locaux** tournent sur votre ordinateur : avec un modèle local, vos requêtes et vos fichiers ne le quittent jamais.
- Les projets tournent dans les microVM de **Docker Sandboxes** (`sbx`, de Docker), installé dans le kit. Il faut un **compte Docker**, connecté avec `sbx login` sur chaque ordinateur ; Atollo lance `sbx` avec la télémétrie et l'envoi du diagnostic de Docker désactivés. Ce que Docker Sandboxes garde hors du dossier d'Atollo est listé à la section 4.
- **Les agents IA ne tournent que dans ces microVM** (Claude Code et Codex chacun dans sa propre sandbox, Pi et Antigravity dans la VM du projet) ; sans la VM, Atollo ne lance aucun agent. **Aucun secret que garde le kit n'est mis dans une VM** : votre jeton GitHub (seulement avec « GitHub dans la VM » activé pour un projet) et la clé API Gemini (pour Antigravity) n'atteignent que le proxy de Docker Sandboxes, qui les ajoute aux requêtes sortant de cette VM ; la seule exception encore ouverte est la clé des modèles locaux donnée à Pi (section 4). Les règles sont dans [docs/app/LEVELS.md](app/LEVELS.md).

## 2. Ce qu'Atollo ne collecte pas

Atollo ne collecte, n'envoie et ne vend pas :

- de données personnelles, noms, adresses e-mail ou identifiants d'appareil ;
- de statistiques d'utilisation, clics, pages vues ou fonctions utilisées ;
- de rapports de plantage ni de journaux (le zip du diagnostic de la section 4 est créé sur votre appareil et n'en sort que si vous le joignez vous-même) ;
- le contenu de vos projets, fichiers, bases de données ou conversations avec les agents IA.

Personne parmi ceux qui gèrent le projet Atollo ne reçoit de données de l'application. Le projet n'exploite aucun serveur : mises à jour et versions sont hébergées sur GitHub.

Atollo désactive aussi, dans son propre environnement, la télémétrie et les vérifications de mise à jour de plusieurs programmes qu'il gère : l'éditeur intégré (VSCodium) démarre avec la télémétrie désactivée et sans mise à jour automatique des extensions ; l'environnement définit `GOTELEMETRY=off`, `DOTNET_CLI_TELEMETRY_OPTOUT=1`, `POWERSHELL_TELEMETRY_OPTOUT=1`, `POWERSHELL_UPDATECHECK=Off`, `PIP_DISABLE_PIP_VERSION_CHECK=1`, `GH_NO_UPDATE_NOTIFIER=1` (pour les appels de l'application à GitHub CLI), `DISABLE_UPDATES=1` (Claude Code), `AGY_CLI_DISABLE_AUTO_UPDATE=true` (Antigravity), `PI_SKIP_VERSION_CHECK=1` et `PI_TELEMETRY=0` (Pi). Chaque exécution du Docker Sandboxes du kit reçoit `SBX_NO_TELEMETRY=1` (les rapports d'utilisation de Docker désactivés ; qu'il arrête tout envoi reste à vérifier en conditions réelles, et `sbx` garde sa file locale d'événements dans le kit) et `LLMMAN_RUNTIME=path` (son assistant de modèles locaux ne télécharge rien), et Atollo maintient trois réglages sur son démon : `diagnostics.autoUpload=no` (aucun paquet de diagnostic envoyé à Docker), `ssh.agentForwardingEnabled=false` (les VM n'utilisent jamais vos clés SSH) et `browser.enabled=false` (une VM n'ouvre jamais votre navigateur). Les autres programmes gardent leurs réglages par défaut.

## 3. Les connexions réseau de l'application

Chaque requête que l'application fait d'elle-même porte un `User-Agent` de la forme `Atollo/<version> (+https://github.com/nabre/atollo)` et, comme toute connexion à Internet, révèle votre adresse IP au serveur contacté. Les requêtes passent par le proxy défini dans les Paramètres, sinon `HTTPS_PROXY`/`HTTP_PROXY`, sinon le proxy du système.

| Quand | Vers qui | Ce qui est envoyé | Comment la désactiver |
|---|---|---|---|
| Environ 20 secondes après le démarrage, et quand vous cliquez sur « Rechercher des mises à jour » | GitHub (`api.github.com`, puis le téléchargement de la version depuis `github.com`) | La demande de la liste des versions de `nabre/atollo` ; si une version plus récente existe, son téléchargement. Si une variable `GITHUB_TOKEN` ou `GH_TOKEN` est définie sur l'ordinateur, elle est envoyée à `api.github.com` uniquement (pour éviter la limite de requêtes de GitHub). | Paramètres › Général › **Mise à jour automatique** désactivée, ou la variable `ATOLLO_NO_UPDATE` définie sur l'ordinateur. Le bouton reste utilisable à la demande. |
| Environ 10 secondes après le démarrage, au plus une fois par jour et par outil | La source officielle de chaque outil qui propose plusieurs versions (par exemple `nodejs.org`, `go.dev`, `api.adoptium.net`, `api.github.com`) | La demande de la liste des versions disponibles. Vers `api.github.com` seulement : le jeton ci-dessus, ou celui de votre compte GitHub si vous êtes connecté (section 4). | La variable `ATOLLO_NO_UPDATE`. |
| Juste après la ligne précédente, au plus une fois par jour ; et quand vous cliquez sur « Vérifier maintenant » dans le volet « Mises à jour » de la page Outils | La source officielle de chaque outil installé, celle qu'interroge sa mise à jour (par exemple `nodejs.org`, `go.dev`, `api.adoptium.net`, `api.github.com`) | La demande de la dernière version de l'outil, une par outil installé ; les réponses de GitHub passent par le cache de la section 4. Vers `api.github.com` seulement : le même jeton que la ligne ci-dessus. | Paramètres › Général › **Mise à jour automatique** désactivée, ou la variable `ATOLLO_NO_UPDATE`. « Vérifier maintenant » reste utilisable à la demande. |
| Au premier démarrage, quand vous validez la page de premier démarrage (après avoir choisi la langue et le réseau) ; ensuite, automatiquement, quand un composant requis manque ou qu'une mise à jour de l'application en demande de plus récents | Sources officielles de PowerShell 7, GitHub CLI, de l'éditeur et de Docker Sandboxes (versions sur GitHub ; Docker Sandboxes depuis `docker/sbx-releases`, sur Windows x64, les Mac Apple silicon et Linux) | Des requêtes de téléchargement ; les sommes de contrôle sont vérifiées localement. | Pas d'interrupteur : ces composants sont requis. Rien d'autre n'est installé sans action de votre part. |
| Quand vous installez ou mettez à jour un outil (page Outils) ou qu'une action en a besoin (par exemple Node.js pour `npm`) | La source officielle de l'outil, par exemple `nodejs.org`, `registry.npmjs.org`, `go.dev`, `static.rust-lang.org`, `api.adoptium.net`, `repo.maven.apache.org`, `services.gradle.org`, `builds.dotnet.microsoft.com`, `windows.php.net`, `getcomposer.org`, `cdn.mysql.com`, `downloads.mongodb.org`, `sqlite.org`, `files.phpmyadmin.net`, `conda.anaconda.org`, `api.anaconda.org`, `dl.google.com`, `storage.googleapis.com`, `raw.githubusercontent.com`, `www.apachelounge.com`, `dl.static-php.dev`, `curl.se`, `aka.ms`, `api.nuget.org`, `julialang-s3.julialang.org`, `ziglang.org`, le serveur de mise à jour d'Antigravity de Google, les versions sur GitHub (dont `ggml-org/llama.cpp` pour le serveur des modèles locaux et `earendil-works/pi` pour Pi) | Des requêtes de téléchargement ; les sommes de contrôle sont vérifiées en local. | Ne pas installer l'outil. |
| Seulement quand vous téléchargez un modèle local (page IA, volet Modèles locaux, ou `models pull`) ou consultez un dépôt Hugging Face | Hugging Face : `huggingface.co` (l'API, pour le dernier commit d'un dépôt et ses fichiers GGUF), puis son CDN pour les fichiers du modèle | Des requêtes pour la liste du dépôt et pour les fichiers du modèle à un commit fixé ; un téléchargement suspendu ou interrompu reprend avec une requête `Range`. Votre jeton Hugging Face, seulement si vous en avez enregistré un pour les modèles à accès restreint, et seulement vers `huggingface.co` (il ne passe pas au CDN). Le SHA-256 de chaque fichier est vérifié en local. | Ne pas télécharger de modèles. |
| En arrière-plan après le démarrage de l'éditeur, et à l'ouverture d'un projet | La galerie d'extensions de l'éditeur (Open VSX) ; CodeLLDB depuis sa version sur GitHub | Les demandes des extensions utilisées par le kit et vos projets. | Ne pas ouvrir l'éditeur, ou retirer les extensions de l'éditeur. |
| Seulement quand vous vous connectez à GitHub (Paramètres › Compte GitHub) et utilisez vos dépôts (Projets › « Depuis vos dépôts GitHub ») | GitHub (`github.com`, `api.github.com`), via GitHub CLI | La connexion (un code à usage unique à saisir sur `github.com/login/device`), puis le jeton de votre compte avec les requêtes pour lister, cloner, renommer ou supprimer vos dépôts. | Ne pas vous connecter, ou vous déconnecter dans Paramètres › Compte GitHub. |
| Une fois, à la connexion ou à un démarrage avec la connexion faite, seulement si le nom et l'e-mail des commits sont vides | GitHub (`api.github.com/user`), via GitHub CLI | Le jeton de votre compte, pour lire le nom, l'identifiant et le numéro du compte : ils deviennent le nom de vos commits et l'adresse privée `noreply` de GitHub. | Remplir Paramètres › Compte GitHub › « Qui signe les commits », ou ne pas vous connecter. |
| Quand un agent utilise le serveur MCP **github** du kit (Agents › Serveurs MCP du kit), seulement si vous êtes connecté à GitHub | Le serveur MCP de GitHub (`api.githubcopilot.com`), via `Atollo mcp github`, le pont que l'agent lance | Les requêtes de l'agent aux outils de GitHub (issues, pull requests, dépôts) avec le jeton de votre compte GitHub, que le pont ajoute lui-même : l'agent ne le voit jamais. | N'ajoutez pas le serveur github, ou retirez-le des serveurs MCP du kit. |
| Seulement tant que la page IA est affichée (à l'ouverture, puis toutes les 2 minutes), pour chaque agent auquel vous êtes connecté | Claude Code : `api.anthropic.com` ; Codex : `chatgpt.com` ; Antigravity : `cloudcode-pa.googleapis.com` | Le jeton de connexion de l'agent, envoyé uniquement au fournisseur de cet agent, pour lire les limites d'utilisation de votre forfait. Le jeton est seulement lu, jamais renouvelé, et jamais écrit dans le journal. | Ne pas ouvrir la page IA, ou ne pas vous connecter à l'agent. |

Ne sont **pas** des connexions faites par Atollo lui-même :

- les programmes que vous lancez dans l'éditeur, les consoles ou via des actions (`git`, `npm`, `pip`, `cargo`, les agents IA…) se connectent là où ils sont conçus pour le faire ;
- **Docker Sandboxes** (`sbx`, section 5), qu'Atollo lance pour les microVM, parle lui-même avec Docker : la connexion de votre compte Docker (`sbx login`, dont le lien s'ouvre dans votre navigateur quand vous cliquez dessus dans le terminal d'Atollo) et les images dont les VM ont besoin (la sandbox `shell`, `php:<version>-apache` ou `httpd:2.4` pour un site dans une VM). Les programmes d'une VM n'atteignent le réseau qu'à travers le proxy de `sbx`, avec la politique réseau `balanced` de Docker, qu'Atollo définit sur le `sbx` du kit ; vers votre ordinateur, la VM d'un projet n'atteint que le port de sa propre base de données (et, tant que Pi y utilise un modèle local, le serveur des modèles locaux). Avec « GitHub dans la VM » activé pour un projet (désactivé par défaut), le proxy ajoute votre jeton GitHub aux requêtes de cette VM vers GitHub : `sbx` obtient le jeton en lançant `Atollo secret github`, qui ne répond qu'au `sbx` du kit avec un nonce enregistré par Atollo, et le jeton n'entre jamais dans la VM. De même, quand Antigravity tourne dans la VM d'un projet et que les Paramètres contiennent une clé API Gemini, le proxy ajoute la clé aux requêtes de cette VM vers les services Gemini de Google (`Atollo secret gemini`), et la clé n'entre jamais dans la VM ;
- les liens sur lesquels vous cliquez s'ouvrent dans votre navigateur, de même que « Signaler un problème » (section 4).

Serveurs locaux : l'éditeur et les serveurs des projets n'écoutent que sur cet ordinateur (`127.0.0.1`/`localhost`), sauf si vous activez **Réseau local** dans les Paramètres pour les serveurs des projets. Avec le réseau local activé, le QR code de l'adresse d'un site pour le téléphone est créé par Atollo sur cet ordinateur : aucun service sur internet ne voit l'adresse, et trouver l'IP de ce PC sur le réseau local n'envoie aucun paquet. Le serveur des modèles locaux (llama.cpp) n'écoute toujours que sur `127.0.0.1`, sur son propre port (9931 et suivants), et ne répond qu'aux requêtes portant sa clé aléatoire ; `llm status` n'envoie pas cette clé à ce qui écoute sur le port. Les modèles tournent sur cet ordinateur : avec un modèle local, rien ne part sur le réseau. Un site qui tourne dans la microVM d'un projet a son port publié sur `127.0.0.1` de cet ordinateur, ou sur son réseau local quand **Réseau local** est activé ; en le désactivant, cette publication est retirée au prochain démarrage du site (si elle ne peut pas l'être, le site ne démarre pas). Arrêter le site retire son port.

## 4. Les données enregistrées sur votre appareil

Atollo est portable : il garde ses données à côté de l'exécutable, dans le dossier `data/`, et vos projets dans `projects/`. Rien n'est envoyé en ligne. Supprimer le dossier d'Atollo supprime tout.

| Quoi | Où (dans le dossier d'Atollo) |
|---|---|
| Paramètres : langue, thème, proxy, certificats, mise à jour automatique, compilateur Rust | `data/app/settings.json` |
| Les réglages de Git, avec le nom et l'e-mail qui signent vos commits | `data/home/.gitconfig` |
| La **clé API Gemini**, si vous en saisissez une (page IA, Antigravity › Connexion) (lisible par votre seul utilisateur, voir ci-dessous) | `data/app/secrets/gemini-api-key` |
| Les certificats racine du système plus votre fichier PEM supplémentaire, seulement si vous en définissez un | `data/app/ca-bundle.pem` |
| Journaux de l'application (secrets masqués), renouvelés au-delà de 1 Mo | `data/app/logs/` |
| Zip du diagnostic, seulement quand vous en créez un (secrets masqués, voir ci-dessous) | `data/app/diagnostics/` |
| Outils installés et leurs versions | `data/apps/`, `data/app/versions/` |
| La dernière version de chaque outil installé, avec l'heure de la dernière vérification | `data/app/updates.json` |
| Réponses de l'API GitHub en cache | `data/cache/github/` |
| Stockage de l'interface (disposition de la fenêtre) et cache de la web view | `data/webview2/` (Windows et Linux) |
| Compte GitHub (jeton de GitHub CLI, enregistré comme fichier pour voyager avec le kit) | `data/gh/` |
| Identifiants Git pour d'autres hôtes (Git Credential Manager, chiffrés avec DPAPI de Windows pour votre utilisateur) | `data/home/.gcm/` (Windows) |
| Connexions et sessions des agents IA dans les VM, écrites par les agents eux-mêmes | les disques des VM (ligne « Docker Sandboxes » plus bas) |
| Connexions et sessions que les agents d'un Atollo précédent ont écrites sur cet ordinateur (seulement lues ; « Se déconnecter » les retire) | `data/home/.claude/`, `data/home/.codex/`, `data/home/.gemini/` |
| La connexion Google d'Antigravity et le jeton longue durée de Claude Code gardés par un Atollo précédent, s'il y en a : seulement lus, jamais déplacés ni donnés à un programme ; « Se déconnecter » les retire | `data/antigravity/keyring.json`, `data/agents/claude/oauth-token` |
| Mots de passe des bases de données (aléatoires, générés par Atollo) | `data/app/db/secrets.json`, `data/app/db/mysql.cnf`, `data/projects/<projet>/private/db.json` |
| Modèles locaux (fichiers GGUF) et ce qu'Atollo en a téléchargé | `data/models/gguf/<propriétaire>--<dépôt>/` (avec `atollo.json`) |
| Réglages des modèles locaux (port, démarrage automatique, le modèle de Pi) et les préréglages du serveur | `data/app/llm.json`, `data/app/llm/presets.ini` |
| La **clé du serveur des modèles locaux** (aléatoire, générée par Atollo, lisible par votre seul utilisateur) | `data/app/secrets/llama-key` |
| Le **jeton Hugging Face**, si vous en saisissez un pour les modèles à accès restreint (lisible par votre seul utilisateur) | `data/app/secrets/hf-token` |
| Réglages, connexions (`auth.json`) et sessions de Pi, et le `models.json` du kit avec l'adresse et la clé du serveur local (lisible par votre seul utilisateur) | `data/home/.pi/agent/` |
| Données du serveur de l'éditeur et son jeton d'accès par session | `data/vscodium-server/` |
| Docker Sandboxes : ses réglages et journaux, les images et les disques des VM (le home de chaque VM, `/tmp` et son propre Docker) | le home du kit : `data/home/AppData/Local/DockerSandboxes/` sous Windows, sous `data/home/` sous macOS et Linux |
| Le kit de chaque sandbox : sa spécification, à qui elle est (un projet ou l'un de ses agents ; celle d'un terminal du kit seulement pour un Atollo précédent, retirée d'elle-même), ses montages et ses ports ouverts ; pour un projet avec une base de données, le `db.env` avec la connexion à la base du projet donnée à sa VM (lisible par votre seul utilisateur) ; le dossier `guest` que la VM voit en lecture seule : ce qu'`atollo` affiche dans la VM (le niveau, le projet, les outils, adresses et utilisateurs des bases, jamais un mot de passe) | `data/app/sandboxes/<sandbox>/` |
| Les notes des commandes « GitHub dans la VM » et de la clé Gemini qu'Atollo a enregistrées dans Docker Sandboxes, avec leurs nonces (lisibles par votre seul utilisateur) | `data/app/sandboxes/secrets.json` |
| Les requêtes des tâches et des profils d'agents de l'éditeur et des commandes d'agents tapées dans un terminal (l'origine, le dossier, les mots tapés ; aucun secret), chacune retirée une fois traitée ; effacées à chaque démarrage de l'application, jamais vues par une VM | `data/app/levels/requests/` |
| Les scripts que ces requêtes lancent, et les commandes d'agents des terminaux de l'ordinateur qui transmettent à la VM, réécrits à chaque démarrage de l'application | `data/kit/launch.ps1`, `data/kit/launch.sh`, `data/kit/forward/` |
| Vos projets | `projects/` |
| Une copie du kit en attente d'importation, seulement après « Importer depuis… » | `.atollo-import/` |
| Ce qu'une copie importée a remplacé, gardé jusqu'à ce que vous le supprimiez | `.atollo-before-import-<date-heure>/` |

Les mots de passe des bases de données (les `db.env` des sandbox compris), la clé API Gemini, le fichier du jeton Claude, la clé des modèles locaux et le jeton Hugging Face sont créés lisibles par votre seul utilisateur, là où le disque gère les permissions (NTFS, APFS, ext4…). Les autres fichiers, `settings.json` compris, sont des fichiers ordinaires. Sur les clés USB formatées en FAT32 ou exFAT, il n'y a aucune permission : **quiconque possède la clé peut lire tous les fichiers ci-dessus**. Gardez la clé en lieu sûr, ou utilisez un disque avec permissions.

Paramètres › Le kit › **« Copie du kit »** écrit `projects/` et `data/` (sans les programmes, les caches, les modèles locaux ni les images et les disques des VM) dans un dossier ou un zip **que vous choisissez**, éventuellement hors du dossier d'Atollo. La copie contient les secrets du kit **en clair** (`data/app/db/secrets.json`, `data/app/secrets/`, les connexions des agents et de GitHub) : gardez-la en lieu sûr. Atollo ne l'envoie jamais nulle part.

Les agents IA ne tournent que dans les microVM, jamais sur votre ordinateur : aucun secret du kit n'est mis dans une VM, ni dans ses variables ni dans ses fichiers. La clé API Gemini n'atteint que le proxy de Docker Sandboxes, enregistrée pour la sandbox du projet quand Antigravity y démarre, qui l'ajoute aux requêtes d'Antigravity vers Google ; le jeton GitHub de même, seulement avec « GitHub dans la VM » activé. Le fichier du jeton Claude enregistré par un Atollo précédent n'est donné à rien. La seule exception encore ouverte : quand Pi utilise un modèle local, la clé des modèles locaux va à la session de Pi dans la VM du projet (`LLAMA_API_KEY`) et dans le `models.json` de Pi, pour que Pi atteigne le serveur sur votre ordinateur ; une clé réservée à cette sandbox est prévue. Le jeton Hugging Face n'est envoyé qu'à `huggingface.co`.

Exceptions, hors du dossier d'Atollo :

- **Fichiers temporaires** : les fichiers en transit vont dans `<temp du système>/Atollo/` ; selon Paramètres › Général › **Fichiers temporaires des programmes**, les programmes peuvent aussi utiliser le dossier temporaire du système (par défaut quand le kit est sur une clé USB, un disque réseau ou en FAT32/exFAT).
- **Connexion d'Antigravity d'un Atollo précédent** : un Atollo précédent plaçait la connexion d'Antigravity dans le magasin d'identifiants du système pendant qu'Antigravity tournait sur l'ordinateur (sous Windows pour la seule session d'ouverture) ; une connexion qu'une de ses exécutions y a laissée sans se terminer proprement y reste. Atollo ne la déplace plus : Antigravity se connecte dans la VM du projet.
- **Web view de macOS** : sur macOS, la web view du système garde ses données là où macOS le décide (dans la Bibliothèque de votre utilisateur).
- **rustup** écrit une entrée dans le registre de Windows à chaque installation ; Atollo la supprime.
- **Docker Sandboxes**, parce que `sbx` ou le système ne laissent pas d'autre place (tout le reste est dans `data/`) :
  - l'hyperviseur (Windows Hypervisor Platform, `/dev/kvm`), activé une fois par vous en administrateur, jamais par Atollo ;
  - votre connexion à Docker (`sbx login`), que `sbx` garde dans le magasin d'identifiants du système, une fois par ordinateur ;
  - pour chaque projet avec « GitHub dans la VM » activé, une entrée dans le magasin d'identifiants du système (sous Windows le Gestionnaire d'identifiants, `com.docker.sandboxes:…`) contenant la commande que `sbx` lance pour obtenir le jeton, pas le jeton lui-même ; la valeur que `sbx` obtient peut rester dans sa mémoire selon sa propre politique de rafraîchissement ;
  - une jonction `sboxd-<id>` et un dossier `gvisor` dans le dossier temporaire du système, pour les sockets du démon, dont les chemins doivent être courts ;
  - sous Ubuntu, un profil AppArmor pour le shim de la microVM du kit (`/etc/apparmor.d/atollo-sbx-shim`), que vous ajoutez avec la commande `sudo` affichée par Atollo.

### Signaler un problème

Paramètres › Le kit › « Signaler un problème » a deux boutons ; rien ne s'y lance tout seul.

- **« Exporter le diagnostic »** (ou `Atollo diagnostics` en ligne de commande) crée un zip **sur votre appareil**, dans `data/app/diagnostics/` : la version d'Atollo, le système du kit et de l'ordinateur (build de Windows, RAM), le disque du kit (système de fichiers, type, espace libre), le dossier du kit, l'environnement du kit, la vérification des programmes installés, les composants, les ports et les derniers 256 Ko de chaque journal de `data/app/logs/`. Avant l'écriture du zip, chaque secret qu'Atollo conserve (les fichiers de `data/app/secrets/`, le jeton de Claude Code, les mots de passe des bases de données, les connexions des agents dans le kit, le jeton GitHub) est remplacé par `***` partout où il apparaît, en plus du masquage des variables dont le nom semble secret ; le dossier de votre profil devient `~`, et les noms de votre utilisateur et de l'ordinateur (`USERNAME`, `COMPUTERNAME`, `HOSTNAME`…) deviennent `***`. **L'application n'envoie jamais le zip** : ouvrez-le, lisez-le et joignez-le vous-même si vous le souhaitez. La sortie d'un projet dans les journaux peut quand même contenir des choses qu'Atollo ne sait pas privées.
- **« Signaler un problème »** ouvre dans votre navigateur `github.com/nabre/atollo/issues/new`, avec le formulaire des problèmes et deux champs remplis via l'adresse : la version d'Atollo et le système (système du kit, build de Windows, système de fichiers et type du disque du kit). Rien d'autre de votre appareil n'est dans l'adresse, et rien n'arrive à GitHub tant que vous ne vous connectez pas et ne créez pas l'issue. Dès lors, l'issue, avec ce que vous joignez, est publique et la [déclaration de confidentialité de GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement) s'applique.

## 5. Outils et agents IA tiers

Atollo installe et lance des programmes faits par d'autres. Quand vous les utilisez, ils dialoguent directement avec leurs propres services et **leurs politiques de confidentialité et conditions s'appliquent**. Les agents IA ne tournent que dans les microVM de Docker Sandboxes : Claude Code et Codex depuis les images d'agent de Docker, Pi et Antigravity comme leurs versions Linux du kit. Atollo ne sert pas de proxy à ce trafic, ne le lit pas et ne l'enregistre pas, hormis les sessions et connexions que ces programmes écrivent eux-mêmes dans le dossier du kit (section 4).

- **Claude Code** (Anthropic) : [anthropic.com/legal/privacy](https://www.anthropic.com/legal/privacy)
- **Codex** (OpenAI) : [openai.com/policies/privacy-policy](https://openai.com/policies/privacy-policy/)
- **Antigravity** et l'API Gemini (Google) : [policies.google.com/privacy](https://policies.google.com/privacy)
- **Pi** (Earendil) : [pi.dev](https://pi.dev) ; avec un fournisseur cloud (connexion avec `/login`), la politique de ce fournisseur s'applique
- **Hugging Face**, d'où se téléchargent les modèles locaux : [huggingface.co/privacy](https://huggingface.co/privacy)
- **llama.cpp** (ggml-org), le serveur des modèles locaux : il tourne sur votre ordinateur et n'envoie rien ; [github.com/ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp)
- **Docker Sandboxes** (Docker), les microVM des projets, avec votre compte Docker : [docker.com/legal/docker-privacy-policy](https://www.docker.com/legal/docker-privacy-policy/)
- **GitHub** et GitHub CLI : [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
- **VSCodium** et Open VSX : [open-vsx.org](https://open-vsx.org/)
- Les registres de paquets (npm, PyPI, crates.io, Maven Central, NuGet…) et les sites de téléchargement de chaque outil, selon leurs propres règles.

Les agents IA envoient à leurs fournisseurs vos requêtes et les fichiers qu'ils lisent : c'est ainsi qu'ils fonctionnent. Choisissez ce que vous ouvrez avec eux. Pi fait exception quand il utilise un modèle local : requêtes et fichiers restent alors sur votre ordinateur ; avec un fournisseur cloud choisi dans Pi, ils vont à ce fournisseur.

## 6. Le site

Le site [nabre.github.io/atollo](https://nabre.github.io/atollo/fr/) est une page statique hébergée sur GitHub Pages, avec les pages du Guide sous [`/fr/guide/`](https://nabre.github.io/atollo/fr/guide/) (le même Guide que dans l'application, qui le lit depuis l'application elle-même, sans connexion).

- Ni statistiques, ni pistage, ni cookies, ni publicité. Polices et images sont servies par le site lui-même.
- Le site retient le thème et la langue choisis dans le `localStorage` de votre navigateur (`atollo-theme`, `atollo-lang`) ; ils ne quittent pas le navigateur.
- Pour que le bouton de téléchargement pointe vers le bon fichier, votre navigateur demande à `api.github.com` la liste des versions d'Atollo.
- GitHub, en tant qu'hébergeur, peut enregistrer des données techniques comme les adresses IP : voir le [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).
- Le site est vérifié dans Google Search Console par un fichier de vérification ; cela n'ajoute aucun pistage pour les visiteurs.

## 7. Enfants

Atollo est un outil pour développeurs et ne s'adresse pas aux enfants. Il ne collecte de données sur personne, enfants compris.

## 8. Conditions d'utilisation

Atollo est un logiciel libre publié sous [licence MIT](../LICENSE). Il est fourni **« tel quel », sans garantie d'aucune sorte**, expresse ou implicite ; les auteurs ne sont responsables d'aucune réclamation, d'aucun dommage ni d'aucune autre responsabilité découlant de son utilisation. Vous êtes responsable de l'usage que vous en faites et du respect des licences et conditions des outils et services tiers qu'il installe ou auxquels il se connecte (section 5).

## 9. Modifications

Quand cette politique change, la nouvelle version est publiée dans ce fichier avec une nouvelle date d'entrée en vigueur ; les versions précédentes restent dans l'historique du dépôt.

## 10. Contact

Questions ou préoccupations : ouvrez une issue sur [github.com/nabre/atollo/issues](https://github.com/nabre/atollo/issues).
