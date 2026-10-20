# Atollo : politique de confidentialité et conditions d'utilisation

[English](PRIVACY.md) · [Italiano](PRIVACY.it.md) · Français · [Deutsch](PRIVACY.de.md)

En vigueur depuis le : 20 octobre 2026

Cette page explique ce que l'application de bureau Atollo (`Atollo.exe`, la `.app` macOS et l'AppImage Linux) et son site font de vos données. Atollo est open source : tout ce qui est écrit ici peut être vérifié dans le code de ce dépôt.

## 1. En bref

- Atollo n'a **ni télémétrie, ni statistiques d'utilisation, ni rapports de plantage**. Il n'a ni compte ni serveur à lui.
- Tout ce qu'Atollo enregistre reste **dans le dossier d'Atollo** sur votre appareil (`data/` et `projects/`), à quelques exceptions près, listées à la section 4.
- Atollo ne se connecte au réseau que pour **rechercher ses propres mises à jour**, **télécharger les outils que vous utilisez depuis leurs sources officielles**, et dialoguer avec **GitHub et les fournisseurs des agents IA quand vous utilisez ces fonctions**. Le tableau de la section 3 liste chaque connexion et comment la désactiver.
- Les outils et agents IA que vous lancez dans Atollo (Claude Code, Codex, Antigravity, Git, npm…) sont des programmes tiers avec leurs propres politiques de confidentialité.

## 2. Ce qu'Atollo ne collecte pas

Atollo ne collecte, n'envoie et ne vend pas :

- de données personnelles, noms, adresses e-mail ou identifiants d'appareil ;
- de statistiques d'utilisation, clics, pages vues ou fonctions utilisées ;
- de rapports de plantage ni de journaux ;
- le contenu de vos projets, fichiers, bases de données ou conversations avec les agents IA.

Personne parmi ceux qui gèrent le projet Atollo ne reçoit de données de l'application. Le projet n'exploite aucun serveur : mises à jour et versions sont hébergées sur GitHub.

Atollo désactive aussi, dans son propre environnement, la télémétrie et les vérifications de mise à jour de plusieurs programmes qu'il gère : l'éditeur intégré (VSCodium) démarre avec la télémétrie désactivée et sans mise à jour automatique des extensions ; l'environnement définit `GOTELEMETRY=off`, `DOTNET_CLI_TELEMETRY_OPTOUT=1`, `POWERSHELL_TELEMETRY_OPTOUT=1`, `POWERSHELL_UPDATECHECK=Off`, `PIP_DISABLE_PIP_VERSION_CHECK=1`, `GH_NO_UPDATE_NOTIFIER=1` (pour les appels de l'application à GitHub CLI), `DISABLE_UPDATES=1` (Claude Code) et `AGY_CLI_DISABLE_AUTO_UPDATE=true` (Antigravity). Les autres programmes gardent leurs réglages par défaut.

## 3. Les connexions réseau de l'application

Chaque requête que l'application fait d'elle-même porte un `User-Agent` de la forme `Atollo/<version> (+https://github.com/nabre/atollo)` et, comme toute connexion à Internet, révèle votre adresse IP au serveur contacté. Les requêtes passent par le proxy défini dans les Paramètres, sinon `HTTPS_PROXY`/`HTTP_PROXY`, sinon le proxy du système.

| Quand | Vers qui | Ce qui est envoyé | Comment la désactiver |
|---|---|---|---|
| Environ 20 secondes après le démarrage, et quand vous cliquez sur « Rechercher des mises à jour » | GitHub (`api.github.com`, puis le téléchargement de la version depuis `github.com`) | La demande de la liste des versions de `nabre/atollo` ; si une version plus récente existe, son téléchargement. Si une variable `GITHUB_TOKEN` ou `GH_TOKEN` est définie sur l'ordinateur, elle est envoyée à `api.github.com` uniquement (pour éviter la limite de requêtes de GitHub). | Paramètres › Général › **Mise à jour automatique** désactivée, ou la variable `ATOLLO_NO_UPDATE` définie sur l'ordinateur. Le bouton reste utilisable à la demande. |
| Environ 10 secondes après le démarrage, au plus une fois par jour et par outil | La source officielle de chaque outil qui propose plusieurs versions (par exemple `nodejs.org`, `go.dev`, `api.adoptium.net`, `api.github.com`) | La demande de la liste des versions disponibles. Vers `api.github.com` seulement : le jeton ci-dessus, ou celui de votre compte GitHub si vous êtes connecté (section 4). | La variable `ATOLLO_NO_UPDATE`. |
| Au premier démarrage, quand vous validez la page de premier démarrage (après avoir choisi la langue et le réseau) ; ensuite, automatiquement, quand un composant requis manque ou qu'une mise à jour de l'application en demande de plus récents | Sources officielles de PowerShell 7, GitHub CLI et de l'éditeur (versions sur GitHub) | Des requêtes de téléchargement. | Pas d'interrupteur : ces composants sont requis. Rien d'autre n'est installé sans action de votre part. |
| Quand vous installez ou mettez à jour un outil (page Outils) ou qu'une action en a besoin (par exemple Node.js pour `npm`) | La source officielle de l'outil, par exemple `nodejs.org`, `registry.npmjs.org`, `go.dev`, `static.rust-lang.org`, `api.adoptium.net`, `repo.maven.apache.org`, `services.gradle.org`, `builds.dotnet.microsoft.com`, `windows.php.net`, `getcomposer.org`, `cdn.mysql.com`, `downloads.mongodb.org`, `sqlite.org`, `files.phpmyadmin.net`, `conda.anaconda.org`, `dl.google.com`, le serveur de mise à jour d'Antigravity de Google, les versions sur GitHub | Des requêtes de téléchargement ; les sommes de contrôle sont vérifiées en local. | Ne pas installer l'outil. |
| En arrière-plan après le démarrage de l'éditeur, et à l'ouverture d'un projet | La galerie d'extensions de l'éditeur (Open VSX) ; CodeLLDB depuis sa version sur GitHub | Les demandes des extensions utilisées par le kit et vos projets. | Ne pas ouvrir l'éditeur, ou retirer les extensions de l'éditeur. |
| Seulement quand vous utilisez la page GitHub | GitHub (`github.com`, `api.github.com`), via GitHub CLI | La connexion (un code à usage unique à saisir sur `github.com/login/device`), puis le jeton de votre compte avec les requêtes pour lister, cloner, renommer ou supprimer vos dépôts. | Ne pas vous connecter, ou vous déconnecter depuis la page GitHub. |
| Une fois, à la connexion ou à un démarrage avec la connexion faite, seulement si le nom et l'e-mail des commits sont vides | GitHub (`api.github.com/user`), via GitHub CLI | Le jeton de votre compte, pour lire le nom, l'identifiant et le numéro du compte : ils deviennent le nom de vos commits et l'adresse privée `noreply` de GitHub. | Remplir Paramètres › Compte GitHub › « Qui signe les commits », ou ne pas vous connecter. |
| Seulement tant que la page Agents est affichée (à l'ouverture, puis toutes les 2 minutes), pour chaque agent auquel vous êtes connecté | Claude Code : `api.anthropic.com` ; Codex : `chatgpt.com` ; Antigravity : `cloudcode-pa.googleapis.com` | Le jeton de connexion de l'agent, envoyé uniquement au fournisseur de cet agent, pour lire les limites d'utilisation de votre forfait. Le jeton est seulement lu, jamais renouvelé, et jamais écrit dans le journal. | Ne pas ouvrir la page Agents, ou ne pas vous connecter à l'agent. |

Ne sont **pas** des connexions faites par Atollo lui-même :

- les programmes que vous lancez dans l'éditeur, les consoles ou via des actions (`git`, `npm`, `pip`, `cargo`, les agents IA…) se connectent là où ils sont conçus pour le faire ;
- les liens sur lesquels vous cliquez s'ouvrent dans votre navigateur.

Serveurs locaux : l'éditeur et les serveurs des projets n'écoutent que sur cet ordinateur (`127.0.0.1`/`localhost`), sauf si vous activez **Réseau local** dans les Paramètres pour les serveurs des projets.

## 4. Les données enregistrées sur votre appareil

Atollo est portable : il garde ses données à côté de l'exécutable, dans le dossier `data/`, et vos projets dans `projects/`. Rien n'est envoyé en ligne. Supprimer le dossier d'Atollo supprime tout.

| Quoi | Où (dans le dossier d'Atollo) |
|---|---|
| Paramètres : langue, proxy, certificats, mise à jour automatique, versions | `data/app/settings.json` |
| La **clé API Gemini**, si vous en saisissez une (lisible par votre seul utilisateur, voir ci-dessous) | `data/app/secrets/gemini-api-key` |
| Les certificats racine du système plus votre fichier PEM supplémentaire, seulement si vous en définissez un | `data/app/ca-bundle.pem` |
| Journaux de l'application (secrets masqués), renouvelés au-delà de 1 Mo | `data/app/logs/` |
| Outils installés et leurs versions | `data/apps/`, `data/app/versions/` |
| Réponses de l'API GitHub en cache | `data/cache/github/` |
| Stockage de l'interface (disposition de la fenêtre) et cache de la web view | `data/webview2/` (Windows et Linux) |
| Compte GitHub (jeton de GitHub CLI, enregistré comme fichier pour voyager avec le kit) | `data/gh/` |
| Identifiants Git pour d'autres hôtes (Git Credential Manager, chiffrés avec DPAPI de Windows pour votre utilisateur) | `data/home/.gcm/` (Windows) |
| Connexions et sessions des agents IA, écrites par les agents eux-mêmes | `data/home/.claude/`, `data/home/.codex/`, `data/home/.gemini/` |
| La connexion Google d'Antigravity, retirée du magasin d'identifiants du système après chaque exécution | `data/antigravity/keyring.json` |
| Le jeton longue durée de Claude Code, si vous utilisez « Connecter avec un jeton » (macOS) | `data/agents/claude/oauth-token` |
| Mots de passe des bases de données (aléatoires, générés par Atollo) | `data/app/db/secrets.json`, `data/app/db/mysql.cnf`, `data/projects/<projet>/private/db.json` |
| Données du serveur de l'éditeur et son jeton d'accès par session | `data/vscodium-server/` |
| Vos projets | `projects/` |

Les mots de passe des bases de données, la clé API Gemini et le fichier du jeton Claude sont créés lisibles par votre seul utilisateur, là où le disque gère les permissions (NTFS, APFS, ext4…). Les autres fichiers, `settings.json` compris, sont des fichiers ordinaires. Sur les clés USB formatées en FAT32 ou exFAT, il n'y a aucune permission : **quiconque possède la clé peut lire tous les fichiers ci-dessus**. Gardez la clé en lieu sûr, ou utilisez un disque avec permissions.

Exceptions, hors du dossier d'Atollo :

- **Fichiers temporaires** : les fichiers en transit vont dans `<temp du système>/Atollo/` ; selon Paramètres › Général › **Dossier temporaire des programmes**, les programmes peuvent aussi utiliser le dossier temporaire du système (par défaut quand le kit est sur une clé USB, un disque réseau ou en FAT32/exFAT).
- **Connexion d'Antigravity** : pendant qu'Antigravity tourne, sa connexion est placée dans le magasin d'identifiants du système (sous Windows pour la seule session d'ouverture), puis revient dans le kit et en est retirée. Les identifiants déjà présents sur l'ordinateur sont remis tels quels ; une note scellée (`keyring-before.json`, lisible seulement sur cet ordinateur) reste dans le kit jusqu'à l'enregistrement suivant.
- **Claude Code sur macOS** garde sa propre connexion dans le Trousseau, sauf si vous utilisez « Connecter avec un jeton ».
- **Web view de macOS** : sur macOS, la web view du système garde ses données là où macOS le décide (dans la Bibliothèque de votre utilisateur).
- **rustup** écrit une entrée dans le registre de Windows à chaque installation ; Atollo la supprime.

## 5. Outils et agents IA tiers

Atollo installe et lance des programmes faits par d'autres. Quand vous les utilisez, ils dialoguent directement avec leurs propres services et **leurs politiques de confidentialité et conditions s'appliquent**. Atollo ne sert pas de proxy à ce trafic, ne le lit pas et ne l'enregistre pas, hormis les sessions et connexions que ces programmes écrivent eux-mêmes dans le dossier du kit (section 4).

- **Claude Code** (Anthropic) : [anthropic.com/legal/privacy](https://www.anthropic.com/legal/privacy)
- **Codex** (OpenAI) : [openai.com/policies/privacy-policy](https://openai.com/policies/privacy-policy/)
- **Antigravity** et l'API Gemini (Google) : [policies.google.com/privacy](https://policies.google.com/privacy)
- **GitHub** et GitHub CLI : [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
- **VSCodium** et Open VSX : [open-vsx.org](https://open-vsx.org/)
- Les registres de paquets (npm, PyPI, crates.io, Maven Central, NuGet…) et les sites de téléchargement de chaque outil, selon leurs propres règles.

Les agents IA envoient à leurs fournisseurs vos requêtes et les fichiers qu'ils lisent : c'est ainsi qu'ils fonctionnent. Choisissez ce que vous ouvrez avec eux.

## 6. Le site

Le site [nabre.github.io/atollo](https://nabre.github.io/atollo/fr/) est une page statique hébergée sur GitHub Pages.

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
