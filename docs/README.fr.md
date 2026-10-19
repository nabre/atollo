# Atollo — Universal Portable Dev & AI Workspace

[Italiano](../README.md) · [English](README.en.md) · Français · [Deutsch](README.de.md)

**Atollo** est un *Universal Portable Dev & AI Workspace* autonome (`Atollo.exe`) qui réunit **Éditeur** (VSCodium intégré), **Gestionnaire de langages et d'outils** (plus de 20 langages et outils téléchargés à la demande), **Serveurs/Bases de données locales** (MySQL, MongoDB, SQLite, Caddy/Apache) et **Orchestrateur d'agents IA** (Claude Code, Codex et Antigravity avec instructions, skills et serveurs MCP synchronisés) dans un seul dossier portable. Aucune installation, aucun droit d'administrateur, aucun service système hors du dossier de l'espace de travail.

Au premier démarrage, l'application télécharge d'elle-même les composants système : PowerShell 7, GitHub CLI et l'éditeur. Le reste se choisit dans la page « Outils », ou arrive quand il sert : Node.js avec une action `npm`, uv avec un serveur MCP `uvx`, et ainsi de suite.

> En italien : guide [app/GUIDA.md](app/GUIDA.md), décisions [app/DECISIONS.md](app/DECISIONS.md), nouveautés [app/CHANGELOG.md](app/CHANGELOG.md) ; travail ouvert : [issues](https://github.com/nabre/atollo/issues).

## Utilisation

1. Copiez l'exe d'Atollo (par exemple `Atollo-0.2.0-win32-x64.exe`) dans un dossier vide : une clé USB, un disque externe ou un dossier de votre profil (espaces, parenthèses ou `&` dans le nom sont acceptés).
2. Double-cliquez : l'application crée `projects\` et `data\`. Au premier démarrage, choisissez agents, langages et outils : l'application les télécharge et les vérifie (somme de contrôle ou signature).
3. Créez ou clonez un projet dans la page **Projets**. L'éditeur, le terminal et les agents s'ouvrent toujours dans le dossier d'un projet, jamais ailleurs.

Depuis la [page des versions publiées](https://github.com/nabre/atollo/releases) : `Atollo-<version>-win32-x64.exe` ou `Atollo-<version>-win32-arm64.exe` pour Windows, `Atollo-<version>-darwin-arm64.app.zip` (Apple Silicon) ou `Atollo-<version>-darwin-x64.app.zip` (Intel) pour macOS, `Atollo-<version>-linux-x64` ou `Atollo-<version>-linux-arm64` (`.AppImage` et `.tar.gz`) pour Linux ; chaque fichier a son `.sha256`. Le nom indique la version : quand l'app se met à jour seule, l'exe et l'AppImage prennent le nom de la nouvelle version. Dans les exemples ci-dessous, `Atollo.exe` désigne votre exe.

**macOS** (12 ou ultérieur). L'application n'a qu'une signature ad hoc, sans notarisation d'Apple :
1. Décompressez le zip et déplacez `Atollo.app` avec le Finder dans le dossier du kit (par exemple `/Volumes/KIT/Atollo/`).
2. Dans le Terminal, retirez la quarantaine du navigateur : `xattr -dr com.apple.quarantine /Volumes/KIT/Atollo/Atollo.app`.
3. Ouvrez `Atollo.app` depuis ce dossier : `projects/` et `data/` sont créés à côté, jamais dedans.

Sans l'étape 2, macOS lance l'application depuis une copie temporaire (App Translocation) et Atollo s'arrête en affichant la même commande, au lieu de créer le kit dans un dossier qui disparaît.

**Linux** (x64 et ARM64, glibc 2.35 ou ultérieure : Ubuntu 22.04, Debian 12 et plus récentes). Copiez `Atollo-<version>-linux-<arch>.AppImage` dans le dossier du kit, `chmod +x Atollo-<version>-linux-<arch>.AppImage` et lancez-la : le kit est le dossier de l'AppImage. Sans FUSE (conteneurs, certaines distributions minimales) : `./Atollo-<version>-linux-<arch>.AppImage --appimage-extract-and-run`, ou bien `Atollo-<version>-linux-<arch>.tar.gz` avec WebKitGTK 4.1 installé (sur Ubuntu et Debian `sudo apt install libwebkit2gtk-4.1-0`) : extrayez `Atollo` dans le dossier du kit et lancez `./Atollo`. L'AppImage se met à jour seule ; le `tar.gz` signale qu'une nouvelle version est à télécharger.

## Pages

| Page | Contenu |
| --- | --- |
| **Projets** | les projets avec Lancer, Éditeur, Terminal et agent ; **Nouveau projet** : vide, depuis vos dépôts GitHub (avec clonage, renommage et suppression sur GitHub) ou depuis une adresse Git ; avis sur l'environnement. Chaque projet a sa page : **Résumé** (l'action principale avec sa sortie, langages, bases de données, agent, Git), **Ressources** (composants, base de données du projet avec utilisateur et mot de passe, extensions de l'éditeur), **Avancé** (actions, ports, site statique, Git, instructions des agents) |
| **Agents** | Claude Code, Codex et Antigravity installés : version, connexion, limites du forfait et consommation du jour et des 7 derniers jours ; instructions, skills et serveurs MCP partagés dans `data\agents` |
| **Bases de données** | les serveurs MySQL (avec phpMyAdmin) et MongoDB d'Atollo : démarrage et arrêt, mot de passe de l'administrateur, port, données dans `data\db`, sur ce PC seulement ; SQLite sans serveur |
| **Outils** | langages et outils avec leur logo, leurs dépendances et l'espace disque ; installer, mettre à jour, supprimer ; la matrice des extensions de l'éditeur |
| **Paramètres** | langue, terminal, réseau (proxy, certificats, réseau local), clé Gemini, versions de Python et PHP, compilateur Rust (GNU ou MSVC), mises à jour de l'application ; le **compte GitHub** d'Atollo, un seul pour tout (application, Git, éditeur, consoles, agents, GitHub CLI) ; où se trouve le dossier d'Atollo |
| **Retirer le lecteur** | ferme tout ce qu'Atollo a ouvert, ramène la connexion d'Antigravity dans `data\` et permet d'éjecter le lecteur |

En bas de chaque page, le panneau **Activité** : ce qui tourne ou s'installe (projets, bases de données, composants), avec la sortie et les erreurs au même endroit.

La console d'Atollo est la console classique de Windows avec PowerShell 7 intégré et uniquement l'environnement isolé d'Atollo : les programmes installés sur le PC hôte n'interfèrent pas.

## Limites connues

- Sur Windows 10 et 11 (`x64` et `ARM64`), macOS (Apple Silicon et Intel) et Linux (`x64` et `ARM64`). Sur Windows ARM64 l'éditeur, PHP, Apache, MySQL, MongoDB, Android SDK, Julia et TinyTeX tournent en émulation x64 et R n'est pas disponible ; sur Linux ARM64 Android manque. Les versions publiées pour macOS et Linux, à partir du prochain tag, sont essayées en CI mais pas encore sur des machines propres ([#5](https://github.com/nabre/atollo/issues/5)), et la connexion des agents reste à essayer sur un Mac ([#6](https://github.com/nabre/atollo/issues/6)) ; composants et sources par système dans [app/CATALOG.md](app/CATALOG.md) (en italien), la matrice composant × système dans [app/MATRICE-STRUMENTI.html](app/MATRICE-STRUMENTI.html). Apache sur macOS et Linux est un build du kit (D-116), avec PHP via `php-fpm`.
- Sous Linux, MySQL a besoin de deux bibliothèques du système, `libaio` et `libnuma` : sur Ubuntu 24.04 et Debian 13 `sudo apt install libaio1t64 libnuma1` (sur les versions précédentes `libaio1`), sur Fedora et RHEL `sudo dnf install libaio numactl-libs`. Si elles manquent, Atollo le dit avec la commande à exécuter.
- Sur macOS et Linux, un dossier Atollo sur exFAT ou FAT (une clé formatée pour Windows) fonctionne, mais ces systèmes de fichiers n'ont pas de liens : là où un programme en apporte, Atollo copie les fichiers (par exemple les 142 liens internes de Git), ce qui prend plus de place. Sur une clé utilisée seulement avec un Mac ou Linux, mieux vaut APFS ou ext4.
- Hors du dossier d'Atollo, il ne reste que les traces de Windows, les fichiers temporaires de transit dans le dossier temporaire du système (supprimés automatiquement à la fermeture), la connexion de GitHub CLI dans le Gestionnaire d'identification si vous la faites dans une console avec `gh auth login` (celle faite dans les Paramètres reste dans `data\gh`) et celle d'Antigravity pendant qu'`agy` est ouvert (pour la session seulement).
- Les dossiers de `projects\` sont considérés comme fiables par Claude Code et Codex : n'ouvrez que des projets de confiance. L'éditeur, lui, demande à la première ouverture de chaque projet et ouvre en « mode restreint » ceux auxquels vous ne faites pas confiance.
- Les limites d'utilisation des agents viennent d'interfaces non documentées des fournisseurs et peuvent cesser de fonctionner après une de leurs mises à jour.
- « Retirer le lecteur » ne voit pas les programmes du PC qui gardent seulement un fichier du dossier d'Atollo ouvert.

## Structure

```
Atollo\
  Atollo.exe      toute la logique
  projects\       vos projets
  data\           programmes téléchargés, paramètres, connexions, cache, fichiers temporaires
                  (les paramètres de chaque projet : data\projects\<projet>\dev-settings.json)
```

Pour une sauvegarde, copiez `projects\` et `data\` (sans `data\apps\` ni `data\cache\`, qui se téléchargent à nouveau).

## Désinstallation

Atollo n'utilise pas d'installateur, n'enregistre aucun service système ni tâche planifiée et ne modifie ni le registre ni le `PATH` de Windows :

1. Ouvrez la page **Retirer le lecteur** (ou fermez l'application) pour arrêter tous les processus lancés par Atollo et rapatrier dans `data\` la session éventuelle d'Antigravity.
2. Supprimez le dossier contenant `Atollo.exe`, `data\` et `projects\` (après avoir sauvegardé les projets que vous souhaitez conserver).
3. Les éventuels dossiers temporaires dans `%TEMP%` sont automatiquement supprimés à la fermeture ; si vous aviez exécuté `gh auth login` manuellement depuis une console (au lieu de la page **Paramètres**), vous pouvez supprimer l'entrée correspondante du **Gestionnaire d'identification** de Windows.

## Ligne de commande

```
Atollo.exe                           ouvre la fenêtre
Atollo.exe env                       affiche l'environnement que reçoivent les programmes d'Atollo
Atollo.exe components                liste les langages et outils, avec ceux installés
Atollo.exe check                     vérifie que les programmes installés répondent
Atollo.exe prepare [options]         installe, met à jour et supprime les composants choisis
    --only a,b                       seulement ceux-ci (et ce dont ils ont besoin)
    --skip a,b                       pas ceux-ci
    --update-all                     met aussi à jour ceux déjà installés
    --force                          télécharge à nouveau
Atollo.exe sync [--project <dossier>] [--verbose]
                                     aligne instructions, skills et MCP des trois agents
Atollo.exe keyring restore|save      connexion d'Antigravity (utilisée par agy.cmd)
Atollo.exe serve <dossier> <port>    serveur de fichiers pour un site statique
Atollo.exe version                   affiche la version
Atollo.exe help                      affiche l'aide
```

Depuis PowerShell, ajoutez `| Out-Host` (par exemple `.\Atollo.exe check | Out-Host`) : `Atollo.exe` est un programme à fenêtre, et sans pipe PowerShell ne l'attend pas et ne lit ni sa sortie ni son code de retour.

## Compiler

Prérequis : Rust stable (au moins 1.93) avec la chaîne MSVC (Visual Studio Build Tools, charge de travail C++), Node.js LTS.

```
npm ci
npm run build
```

Les dépendances JavaScript sont seulement à la racine (`node_modules` et `package-lock.json`) : `ui` est un workspace npm.

L'exe se trouve dans `target\release\Atollo.exe`. Aucun installateur n'est créé (`--no-bundle`).

Contrôles avant une modification :

```
npm run lint
npm test
npm -w ui run build
```

`npm run lint` exécute `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint et Prettier ; `npm test` exécute `cargo test` et les tests de l'interface ; `npm -w ui run build` vérifie les types et compile l'interface. Détails (en italien) dans [app/TESTING.md](app/TESTING.md). Organisation du code et où placer une modification (en italien) : [app/ARCHITECTURE.md](app/ARCHITECTURE.md).

Développement avec rechargement automatique : `npm run dev`.

## Si Windows bloque l'exe

Tant que le certificat de signature n'est pas actif sur la version téléchargée, si SmartScreen affiche « Windows a protégé votre ordinateur », choisissez **Informations complémentaires** › **Exécuter quand même**. Si l'exe a été téléchargé depuis Internet : clic droit › **Propriétés** › **Débloquer**. Sur les PC d'entreprise avec AppLocker, il faut l'autorisation du service informatique.

## Code signing policy

Free code signing on Windows provided by [SignPath.io](https://signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

- **Rôles du projet (Committer, Reviewer et Approver) :** [nabre](https://github.com/nabre)
- **Provenance des binaires :** les exécutables des versions officielles sont compilés et signés exclusivement par le pipeline automatisé GitHub Actions ([`.github/workflows/app.yml`](../.github/workflows/app.yml)) à partir du code source public de ce dépôt, sans intervention manuelle sur les binaires.

## Privacy policy

Atollo (`Atollo.exe`) respecte pleinement la vie privée de l'utilisateur :

- **Aucune télémétrie ni collecte de données :** l'application ne collecte aucune donnée personnelle, statistique d'utilisation, télémétrie ou rapport de plantage, et ne contacte aucun serveur géré par l'auteur du projet. Elle désactive également la télémétrie intégrée des composants gérés (tels que PowerShell 7 et le serveur VSCodium).
- **Stockage local :** les paramètres, projets, bases de données locales, journaux et identifiants sont enregistrés exclusivement en local dans le dossier `data\` (et `projects\`) d'Atollo sur l'appareil de l'utilisateur.
- **Connexions réseau :** l'application n'effectue des requêtes réseau que vers les sources amont officielles pour vérifier les mises à jour et télécharger les composants demandés par l'utilisateur (`api.github.com`, miroirs officiels des langages/outils, registre Open VSX pour les extensions de l'éditeur) et, lorsque l'utilisateur configure et utilise l'intégration GitHub, les agents IA (**Claude Code**, **Codex**, **Antigravity**) ou une clé API Gemini, directement vers les API des fournisseurs respectifs selon leurs propres politiques de confidentialité.

## Licence

MIT.
