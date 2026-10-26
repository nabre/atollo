<p align="center">
  <a href="https://nabre.github.io/atollo/fr/">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="brand/wordmark-dark.svg">
      <img src="brand/wordmark-light.svg" alt="Atollo" width="280">
    </picture>
  </a>
</p>

<h1 align="center">Atollo — Universal Portable Dev & AI Workspace</h1>

<p align="center">
  Un environnement de développement portable et sans installation pour Windows x64, les Mac Apple Silicon et Linux x64 et ARM64 (les systèmes où tourne Docker Sandboxes), qui fait tourner chaque projet dans sa propre microVM Docker Sandboxes.<br>
  Éditeur, plus de 20 langages et runtimes, bases de données locales, modèles d'IA locaux et les agents IA Claude Code, Codex, Antigravity et Pi dans un dossier à emporter sur une clé USB.
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
  <a href="https://nabre.github.io/atollo/fr/"><b>Site</b></a> ·
  <a href="https://github.com/nabre/atollo/releases/latest"><b>Télécharger</b></a> ·
  <a href="https://nabre.github.io/atollo/fr/guide/"><b>Guide</b></a> ·
  <a href="https://github.com/nabre/atollo/issues"><b>Issues</b></a> ·
  <a href="PRIVACY.fr.md"><b>Confidentialité</b></a>
</p>

<p align="center"><a href="../README.md">English</a> · <a href="README.it.md">Italiano</a> · Français · <a href="README.de.md">Deutsch</a></p>

**Atollo** est un *Universal Portable Dev & AI Workspace* autonome (`Atollo.exe`). Son cœur est le configurateur portable de **Docker Sandboxes** (`sbx`) : chaque projet tourne dans sa propre microVM, avec son terminal, ses actions, son site, les terminaux de l'éditeur et les agents Claude Code, Codex et Pi, et ne voit que son propre dossier. Autour de ce cœur, il réunit **Éditeur** (VSCodium intégré), **Gestionnaire de langages et d'outils** (plus de 20 langages et outils téléchargés à la demande, aussi pour le Linux de la VM), **Serveurs/Bases de données locales** (MySQL, MongoDB, SQLite, Caddy/Apache) et **Orchestrateur d'agents IA** (Claude Code, Codex, Antigravity et Pi avec instructions, skills et serveurs MCP synchronisés, plus les modèles de langage locaux avec llama.cpp) dans un seul dossier portable. Aucune installation, aucun droit d'administrateur, aucun service système hors du dossier de l'espace de travail.

Chaque programme se trouve dans l'une de trois boîtes :

- **La VM** : tout ce qui exécute le code d'un projet. Le terminal du projet, ses actions (`npm run dev`…) et leurs sites (un conteneur dans la VM, son port publié sur le PC), les sites des pools de Serveurs Web, un projet créé depuis un terminal ou un framework, les terminaux, les tâches et les profils d'agent de l'éditeur, et les agents : Claude Code et Codex dans les sandbox d'agent de `sbx`, Pi dans la VM du projet (sa build pour Linux). Dedans : le dossier du projet, en écriture, et les outils du kit pour Linux aux versions du projet, en lecture seule. Vers le PC : seulement le port de la base de données du projet et, si le projet le permet, GitHub.
- **Le kit sur le PC** : les programmes du kit, jamais un agent : les terminaux du kit (« Tous les outils » et les vôtres, dans n'importe quel dossier, avec les outils du kit mais sans les CLI des agents, et un avertissement que les agents ne tournent que dans la VM), les serveurs de bases de données et leurs outils (phpMyAdmin, mongo-express, copies), llama.cpp, la fenêtre de l'éditeur, Git pour cloner, `sbx` lui-même et l'application. Antigravity, un éditeur de bureau, reste ici pour les instructions, skills et serveurs MCP synchronisés et ne démarre jamais sur un projet.
- **Sans VM** : quand la VM n'est pas disponible, le terminal, les actions, les sites et les pools du projet tournent sur le PC avec un environnement scellé et sans les CLI des agents ; les terminaux, tâches et profils d'agent de l'éditeur ne démarrent pas (le terminal dit pourquoi et comment corriger ; la fenêtre de l'éditeur s'ouvre quand même) et aucun agent ne démarre. La barre d'état le dit avec l'avertissement **Sans VM**, les raisons et comment corriger chacune.

| Quoi | Avec la VM | Sans VM |
| --- | --- | --- |
| Terminaux du kit (« Tous les outils » et les vôtres, n'importe quel dossier) | sur le PC avec les outils du kit, sans les CLI des agents, avec un avertissement que les agents ne tournent que dans la VM | pareil |
| Terminal du projet, actions, leurs sites, pools de Serveurs Web, créer un projet depuis un terminal ou un framework | dans la VM du projet (ports publiés sur le PC, bases de données du kit atteintes par leur port) | sur le PC avec l'environnement scellé, sans les CLI des agents |
| Terminaux, tâches et profils d'agent de l'éditeur | dans la VM du projet, avec les versions des outils du projet | ne démarrent pas : le terminal dit pourquoi et comment corriger ; la fenêtre de l'éditeur s'ouvre quand même |
| Agents | Claude Code et Codex dans leurs sandbox, Pi dans la VM du projet ; Antigravity ne démarre jamais sur un projet | aucun agent ne démarre |

Au premier démarrage, l'application télécharge d'elle-même les composants requis : PowerShell 7, GitHub CLI, l'éditeur et Docker Sandboxes (0.49.2 ou plus récent, environ 470 Mo, sur les systèmes où Docker le publie). Le reste se choisit dans la page « Outils », ou arrive quand il sert : Node.js avec une action `npm`, uv avec un serveur MCP `uvx`, et ainsi de suite.

<picture>
  <source media="(prefers-color-scheme: light)" srcset="../site/shots/fr/shot-projects-light.png">
  <img src="../site/shots/fr/shot-projects-dark.png" alt="La page Projets d'Atollo : panneau rapide et activité" width="100%">
</picture>

## Fonctionnalités

- **Chaque projet dans sa propre microVM.** Avec Docker Sandboxes, chaque projet a sa propre microVM (noyau, réseau derrière le proxy de `sbx` avec la politique `balanced` d'Atollo, son propre Docker) : le terminal du projet (`sbx exec`), ses actions, son site (le serveur statique, Caddy ou Apache du kit devient un conteneur dans la VM, `httpd:2.4` ou `php:<version>-apache`, avec les alias du projet lui-même, son port publié sur `127.0.0.1` du PC, ou sur le réseau local si les Paramètres le permettent), les sites des pools de Serveurs Web, les projets créés depuis un terminal ou un framework, Claude Code et Codex dans les sandbox d'agent de `sbx` et Pi dans la VM elle-même. La VM monte le dossier du projet en écriture et les outils du kit pour Linux en lecture seule, aux versions choisies par le projet : téléchargés une fois dans le kit et vérifiés par SHA-256, ils voyagent avec le kit, marchent hors ligne, démarrent vite et la VM ne peut pas les modifier ni n'a besoin des sites de téléchargement ; les paquets du projet (`node_modules`, `vendor`, `.venv`), c'est le projet qui les installe dans la VM. Seul le `sbx` du kit est utilisé, jamais un `sbx` installé sur le PC ; ses sandbox, disques, images, réglages et journaux restent dans `data`. La fenêtre de la VM (un clic sur VM dans la barre d'état) liste les sandbox du kit, à qui appartient chacune, son espace disque réel, et les supprime. **Arrêter** termine une action dans sa VM et retire son port. Si le kit change de lettre de lecteur ou de dossier, la sandbox d'un projet est refaite toute seule : fichiers, bases de données, outils et connexions sont dans le kit, seuls le dossier personnel de la VM et ses images téléchargées sont perdus.
- **Sans VM ? Un avertissement clair.** Quand le PC ne peut pas faire tourner la VM (un système non pris en charge, l'hyperviseur désactivé, `sbx` non connecté à Docker, un autre Docker Sandboxes déjà actif, un disque FAT32 ou exFAT…), la barre d'état affiche **Sans VM** à côté d'Activité ; un clic donne chaque raison et sa correction : **Lancer** ouvre `sbx login` dans le terminal d'Atollo, **Installer dans le kit** (si une installation a échoué, avec sa raison ; Atollo réessaie aussi tout seul toutes les heures) et, pour les étapes administrateur (Windows Hypervisor Platform, le groupe `kvm`, le profil AppArmor), la commande exacte avec un bouton **Copier**, que vous exécutez vous-même, une fois (Atollo ne les lance jamais et ne demande jamais d'élévation) ; **Vérifier à nouveau** redemande. Pendant ce temps le terminal, les actions et les sites du projet tournent sur le PC avec un environnement scellé : sans les variables qui mènent à votre vrai profil, à votre agent SSH et à la session D-Bus, sans les CLI des agents, et avec la connexion à la base de données de l'utilisateur du projet, jamais le root du kit. Les terminaux de l'éditeur et les agents ne démarrent pas.
- **Les terminaux du kit sur le PC.** « Tous les outils » et vos propres terminaux, dans n'importe quel dossier, tournent toujours sur le PC avec les outils du kit, avec ou sans VM, mais sans les CLI des agents (`claude`, `codex`, `agy`, `pi`) : un avertissement dans le terminal dit que les agents ne tournent que dans la VM, depuis la page du projet ou l'éditeur.
- **Éditeur.** VSCodium intégré avec les thèmes Atollo sombre et clair, dans une fenêtre mesurée sur le workbench de VSCodium 1.135 : les mêmes tailles, les mêmes couleurs et les icônes codicon. Sa fenêtre tourne sur le PC. Ses terminaux (chaque profil du kit), ses tâches et ses profils d'agent (Claude Code, Codex, Pi) tournent dans la VM du projet, dans le dossier où ils s'ouvrent, avec les versions des outils du projet ; Antigravity ne démarre jamais sur un projet. Fermé par défaut : sans VM, ou si Atollo n'a pas noté si la VM est active, si la VM du projet n'a pas été préparée ou si le dossier n'est pas celui d'un projet, le terminal n'ouvre rien et dit pourquoi et comment corriger ; la fenêtre de l'éditeur s'ouvre quand même. Une limite déclarée : les adaptateurs du débogueur de l'éditeur lancent les programmes sur le PC.
- **Langages et outils à la demande.** Plus de 20 langages et outils (Node.js, Python, uv, PHP, Rust, Go, Java, .NET, Dart, Julia, Ruby, C/C++, Bun, Deno, Typst, TinyTeX, R, Android SDK…), téléchargés quand il le faut et vérifiés par somme SHA-256 ou signature. Plusieurs versions côte à côte, une par projet. Avec la VM active, les outils choisis s'installent pour le Linux de la VM et le PC ne reçoit que les composants de l'application (les outils déjà sur le PC restent, pour un PC sans VM) ; la matrice composants × systèmes indique ce qui tourne « dans la VM ».
- **Bases de données et serveurs web locaux.** MySQL avec phpMyAdmin, MongoDB avec mongo-express et SQLite, démarrés et arrêtés d'un clic, avec base, utilisateur et mot de passe pour chaque projet ; plusieurs versions côte à côte, chacune gardant son port. Les serveurs restent sur le PC, dans le kit : la VM d'un projet n'atteint que le port de sa base de données, en tant qu'utilisateur du projet (le mot de passe dans un fichier lisible par vous seul, jamais sur une ligne de commande) ; la connexion root du kit n'entre jamais dans une VM. Caddy et Apache pour les sites sur `<projet>.localhost`, sur chaque système, seuls ou dans un serveur partagé par plusieurs projets (un pool) : avec la VM le site de chaque projet tourne dans sa VM, sans elle sur le PC. Un port déjà pris est refusé, avec le nom de qui le tient. Aucun service système.
- **GitHub dans la VM, par projet.** La section Git d'un projet a « GitHub dans la VM », désactivé par défaut : activé, `sbx` demande le jeton au kit par une commande (`Atollo secret github`) et son proxy l'ajoute aux requêtes vers GitHub ; le jeton n'est jamais copié et n'entre jamais dans la VM. Jamais donné à un terminal du kit.
- **Quatre agents IA, une seule configuration.** Claude Code, Codex, Antigravity et Pi : installation, connexion et lancement. Instructions, skills et serveurs MCP s'écrivent une fois dans `data/agents` et `atollo sync` les aligne pour les quatre. Les serveurs MCP prêts sont à un clic : GitHub, Playwright et Fetch pour tout le kit, les fichiers du projet et sa base de données MySQL ou MongoDB pour un projet, sans jeton ni mot de passe écrit dans les fichiers. Limites du forfait et consommation du jour et des 7 derniers jours. Les agents ne tournent que dans la VM : Claude Code et Codex dans les sandbox d'agent de `sbx`, avec les connexions que `sbx` gère, les outils du kit du projet dans le `PATH` et sa base de données (le port ouvert pour leur sandbox), et Pi dans la VM du projet avec sa build pour Linux, depuis la page de l'application comme depuis l'éditeur ; les serveurs MCP du kit ne les atteignent pas encore. Antigravity, un éditeur de bureau, ne démarre jamais sur un projet : il reste dans le kit pour les instructions, skills et serveurs MCP synchronisés. Sans VM aucun agent ne démarre, et jamais sur le PC à sa place.
- **Modèles d'IA locaux.** Des modèles de langage qui tournent sur le PC, sans réseau et sans compte : modèles GGUF du catalogue d'Atollo ou de n'importe quel dépôt Hugging Face, téléchargements qui reprennent là où ils se sont arrêtés (même après la fermeture d'Atollo) et vérifiés par SHA-256. Le serveur llama.cpp (CPU, ou GPU avec Vulkan ou Metal) n'écoute que sur 127.0.0.1, avec une clé ; Pi l'utilise sans autre réglage, et son chat s'ouvre dans le navigateur. Une seule copie dans `data/models`, partagée par Windows, macOS et Linux.
- **Un dossier pour quatre systèmes.** Le même dossier, ou le même SSD externe, passe de Windows à macOS à Linux avec projets, bases de données et réglages des agents intacts : chaque machine exécute les programmes natifs de son système. « Retirer le lecteur » ferme tout ce qu'Atollo a ouvert et laisse éjecter le disque.

## Documentation

La documentation détaillée est en anglais (et en italien) :

| Document | Contenu |
| --- | --- |
| [Guide](https://nabre.github.io/atollo/fr/guide/) | chaque page de l'application, pas à pas, avec captures d'écran, en français sur le site (aussi dans l'application) |
| [Catalogue des composants](app/CATALOG.md) | les composants, la source de chacun, la disponibilité par système |
| [Matrice outils × systèmes](app/TOOLS-MATRIX.html) | chaque composant sur les quatre cibles : natif ou absent |
| [Architecture](app/ARCHITECTURE.md) | l'organisation du code et où placer une modification |
| [Tests](app/TESTING.md) | contrôles automatiques et liste de vérification manuelle par système |
| [Décisions](app/DECISIONS.md) et [nouveautés](app/CHANGELOG.md) | les raisons de chaque choix et ce qui a changé, en italien |

Le travail ouvert est suivi dans les [issues](https://github.com/nabre/atollo/issues).

## Utilisation

1. Copiez l'exe d'Atollo (par exemple `Atollo-0.7.2-win32-x64.exe`) dans un dossier vide : une clé USB, un disque externe ou un dossier de votre profil (espaces, parenthèses ou `&` dans le nom sont acceptés).
2. Double-cliquez : l'application crée `projects\` et `data\`. Au premier démarrage, choisissez agents, langages et outils : l'application les télécharge et les vérifie (somme de contrôle ou signature), Docker Sandboxes compris.
3. Connectez-vous à Docker une fois par PC : avec un compte Docker (gratuit), cliquez sur **Sans VM** dans la barre d'état puis sur **Lancer** à côté de `sbx login` : la commande s'ouvre dans le terminal d'Atollo avec le `sbx` du kit, et le lien de connexion y apparaît (un clic l'ouvre dans votre navigateur). Les autres raisons de la même liste disent ce dont le PC a encore besoin.
4. Créez ou clonez un projet dans la page **Projets**. L'éditeur, le terminal et les agents d'un projet s'ouvrent toujours dans son dossier ; avec la VM active, le terminal, les actions, les terminaux de l'éditeur et les agents s'ouvrent dans la microVM du projet.

### Prérequis de la VM

- Windows 11 (x64) avec **Windows Hypervisor Platform** activé ;
- macOS 14 ou plus récent sur Apple silicon ;
- Ubuntu 24.04 ou plus récent (x64 ou ARM64 ; pas ses dérivées) avec `/dev/kvm` utilisable par votre utilisateur (le groupe `kvm`) et, sur les Ubuntu récents, qui restreignent les espaces de noms utilisateur, le profil AppArmor pour le shim de la microVM du kit (à refaire si le kit change de dossier) ;
- la virtualisation activée dans le firmware (BIOS/UEFI) ; dans une machine virtuelle, la virtualisation imbriquée ;
- un compte Docker gratuit (`sbx login`, une fois par PC, sans administrateur) ;
- le kit sur NTFS, APFS ou ext4 : les disques des VM ne tiennent pas sur FAT32 ou exFAT.

Le reste, Atollo le fait tout seul : il installe Docker Sandboxes dans le kit (en réessayant toutes les heures en cas d'échec), démarre le démon du kit, règle la politique réseau et les réglages du démon, nettoie la jonction du démon et refait la sandbox d'un projet après un déplacement du kit. Les étapes administrateur (Windows Hypervisor Platform et un redémarrage, le groupe `kvm` et une nouvelle connexion, le profil AppArmor) sont affichées avec la commande exacte et un bouton **Copier**, par exemple `Enable-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform -All` dans un PowerShell administrateur : Atollo ne les lance jamais. Aucun logiciel ne peut donner la VM à Windows 10, à macOS avant 14, à un Linux autre qu'Ubuntu 24.04 ou plus récent, à un PC dont la virtualisation est désactivée dans le firmware, à une machine virtuelle sans virtualisation imbriquée ou à un kit sur FAT32 ou exFAT : là, l'avertissement **Sans VM** reste. Docker Desktop n'est pas nécessaire.

Depuis la [page des versions publiées](https://github.com/nabre/atollo/releases) : `Atollo-<version>-win32-x64.exe` pour Windows, `Atollo-<version>-darwin-arm64.app.zip` pour macOS (Apple Silicon), `Atollo-<version>-linux-x64` ou `Atollo-<version>-linux-arm64` (`.AppImage` et `.tar.gz`) pour Linux ; chaque fichier a son `.sha256`. Le nom indique la version : quand l'app se met à jour seule, l'exe et l'AppImage prennent le nom de la nouvelle version. Dans les exemples ci-dessous, `Atollo.exe` désigne votre exe.

**macOS** (12 ou ultérieur, Apple Silicon). L'application n'a qu'une signature ad hoc, sans notarisation d'Apple :
1. Décompressez le zip et déplacez `Atollo.app` avec le Finder dans le dossier du kit (par exemple `/Volumes/KIT/Atollo/`).
2. Dans le Terminal, retirez la quarantaine du navigateur : `xattr -dr com.apple.quarantine /Volumes/KIT/Atollo/Atollo.app`.
3. Ouvrez `Atollo.app` depuis ce dossier : `projects/` et `data/` sont créés à côté, jamais dedans.

Sans l'étape 2, macOS lance l'application depuis une copie temporaire (App Translocation) et Atollo s'arrête en affichant la même commande, au lieu de créer le kit dans un dossier qui disparaît.

**Linux** (x64 et ARM64, glibc 2.35 ou ultérieure : Ubuntu 22.04, Debian 12 et plus récentes). Copiez `Atollo-<version>-linux-<arch>.AppImage` dans le dossier du kit, `chmod +x Atollo-<version>-linux-<arch>.AppImage` et lancez-la : le kit est le dossier de l'AppImage. Sans FUSE (conteneurs, certaines distributions minimales) : `./Atollo-<version>-linux-<arch>.AppImage --appimage-extract-and-run`, ou bien `Atollo-<version>-linux-<arch>.tar.gz` avec WebKitGTK 4.1 installé (sur Ubuntu et Debian `sudo apt install libwebkit2gtk-4.1-0`) : extrayez `Atollo` dans le dossier du kit et lancez `./Atollo`. L'AppImage se met à jour seule ; le `tar.gz` signale qu'une nouvelle version est à télécharger.

## Pages

| Page | Contenu |
| --- | --- |
| **Projets** | le **panneau rapide** (ce qui tourne, projets favoris et récents avec Démarrer, Éditeur, Terminal et agent, ce qu'il faut corriger), la liste des projets avec ▶ sur chaque ligne, Ctrl+P pour en ouvrir un par son nom, Ctrl+Shift+P pour la palette de commandes (démarrer ou arrêter un projet, l'ouvrir, un agent dedans, aller à une page, tout arrêter) ; **Nouveau projet**, un onglet à côté des projets : vide, depuis vos dépôts GitHub (avec clonage, renommage et suppression sur GitHub), depuis une adresse Git ou depuis l'une des 63 bases en groupes (Laravel, Next.js, Django, WordPress, Astro, Tauri…) ; avis sur l'environnement. Chaque projet a sa page, avec Démarrer, Éditeur, Terminal et agent en haut et sept sections : **Lancement** (actions, ports et leur sortie), **Site web**, **Composants** (avec les versions), **Bases de données** (base de données du projet avec utilisateur et mot de passe, copies, une copie de sécurité avant chaque restauration et changement de version), **Extensions**, **Éditeur** (réglages et serveurs MCP), **Git** (avec « GitHub dans la VM ») |
| **IA** | toute l'IA du kit en deux volets. **Agents**: Claude Code, Codex, Antigravity et Pi installés : version, connexion (avec la clé Gemini d'Antigravity), limites du forfait et consommation du jour et des 7 derniers jours ; instructions, skills et serveurs MCP partagés dans `data\agents`. **Modèles locaux**: le serveur llama.cpp (démarrage et arrêt, adresse, le chat dans le navigateur et sa clé), le modèle de Pi, le catalogue d'Atollo des modèles GGUF avec « tient / à la limite / trop grand » pour la RAM ou la mémoire vidéo de ce PC, n'importe quel dépôt Hugging Face (avec un jeton facultatif pour les modèles à accès restreint), téléchargements à suspendre et reprendre ; modèles dans `data\models`, sur ce PC seulement  |
| **Bases de données** | les serveurs MySQL (avec phpMyAdmin) et MongoDB (avec mongo-express) d'Atollo : démarrage et arrêt, mot de passe de l'administrateur, port, plusieurs versions côte à côte chacune sur son port, données dans `data\db`, sur ce PC seulement ; SQLite sans serveur |
| **Serveurs Web** | les serveurs Caddy et Apache partagés par les projets (pools) et vos propres profils, avec les sites `<projet>.localhost` qu'ils servent ; sur ce PC seulement, ou sur le réseau local si les Paramètres le permettent ; avec la VM active le site de chaque projet tourne dans sa VM, sans elle sur le PC |
| **Terminaux** | des consoles avec les outils du kit dans n'importe quel dossier : « Tous les outils » et vos propres terminaux, avec les outils et les versions que vous choisissez ; toujours sur le PC, sans les CLI des agents, avec un avertissement que les agents ne tournent que dans la VM |
| **Outils** | langages et outils par type (système et éditeur, agents IA, modèles locaux, langages, outils de développement, serveurs web, bases de données, visualisation des bases de données), avec leur logo, leurs dépendances et l'espace disque ; installer, mettre à jour, supprimer, plusieurs versions (avec la VM active, pour le Linux de la VM) ; le compilateur Rust (GNU ou MSVC) dans la page de Rust ; la matrice composants × systèmes, avec la colonne de la VM ; la matrice des extensions de l'éditeur |
| **Guide** | le guide de l'application, en neuf chapitres ; chaque page ouvre son propre sujet |
| **Paramètres** | enregistrés dès que vous les changez : langue, thème, les fichiers temporaires des programmes, les paquets gardés pour réinstaller sans réseau, réseau (proxy, certificats, réseau local), mises à jour de l'application, confidentialité ; le **compte GitHub** d'Atollo, un seul pour tout (application, Git, éditeur, consoles, agents, GitHub CLI), et qui signe les commits ; où se trouve le dossier d'Atollo ; les instructions des frameworks ; **Le kit** : ses dossiers, son disque, la plateforme et les versions, avec Santé du kit, Copie du kit, Exporter le diagnostic et Signaler un problème |
| **Retirer le lecteur** | ferme tout ce qu'Atollo a ouvert, arrête le Docker Sandboxes du kit (et avec lui les microVM), ramène la connexion d'Antigravity dans `data\` et permet d'éjecter le lecteur |

En bas de chaque page, le panneau **Activité** : ce qui tourne ou s'installe (actions des projets, chaque version des bases de données, pools des serveurs web, composants), avec la sortie et les erreurs au même endroit ; « Tout arrêter » arrête tout après une confirmation. La barre d'état compte les mêmes choses, et à côté d'Activité elle affiche **VM** quand les projets tournent dans leurs microVM, ou **Sans VM** sur la couleur d'avertissement : un clic ouvre les raisons et leurs corrections et, une fois le Docker Sandboxes du kit installé, les sandbox du kit avec leur espace et un bouton « Supprimer ».

La console d'Atollo est un onglet du panneau des terminaux de l'application sur tous les systèmes, avec le PowerShell 7 du kit (sous Windows sur le ConPTY de Microsoft, la console de Windows Terminal, téléchargé avec le kit) ; elle n'ouvre jamais le terminal du PC et n'a que l'environnement d'Atollo : les programmes installés sur le PC hôte n'interfèrent pas. Avec la VM active, le terminal d'un projet est en revanche un `bash` dans sa microVM (`sbx exec`), dans le même panneau ; les terminaux du kit restent sur le PC. Si la version enregistrée d'un terminal a été supprimée, il indique celle qu'il utilise à sa place (par exemple PHP 8.2 → 8.4).

## Reste à faire

Atollo est en alpha (version 0.7.2). Windows x64 est le système le plus testé, installations réelles en CI comprises ; macOS et Linux ne sont construits et testés qu'en CI. La liste complète est dans [docs/app/TESTING.md](app/TESTING.md) (en anglais).

- **La VM, à terminer** ([D-175](app/DECISIONS.md), [D-178](app/DECISIONS.md)). Les projets dans leurs microVM ne sont essayés en conditions réelles que sur Windows 11 x64 avec Docker Sandboxes 0.49.2 ; macOS et Ubuntu ne sont pas essayés, pas plus que « GitHub dans la VM » avec un vrai compte. Reste à faire : les serveurs MCP du kit pour les agents dans la VM (leurs commandes sont des programmes du PC), à porter dans la passerelle MCP de `sbx` ou à signaler absents.
- **L'alpha, à essayer.** [La 0.7.2 alpha](https://github.com/nabre/atollo/releases/tag/app-v0.7.2) est sur GitHub pour Windows x64, macOS sur Apple silicon et Linux x64 et ARM64, les quatre systèmes où tourne Docker Sandboxes ([D-176](app/DECISIONS.md)). Les fichiers publiés sont construits et testés en CI : il reste à les télécharger et à les lancer sur des machines vierges.
- **Tests alpha.** Il faut des personnes qui utilisent Atollo sur de vrais projets et des PC différents, et qui signalent ce qui ne va pas : installation des composants, projets, bases de données, agents. Paramètres › Le kit › « Signaler un problème » ouvre une issue GitHub avec la version et le système déjà remplis ; joignez-y le zip de « Exporter le diagnostic » (secrets masqués, créé sur votre PC, jamais envoyé par l'application).
- **macOS sur de vrais Mac** ([#5](https://github.com/nabre/atollo/issues/5), [#6](https://github.com/nabre/atollo/issues/6)). L'app pour Apple Silicon est construite et lancée en CI, jamais sur un Mac vierge : le zip téléchargé avec un navigateur, la quarantaine et Gatekeeper, App Translocation, un kit sur une clé exFAT, les connexions des agents emportées d'un Mac à l'autre.
- **Linux sur de vraies machines** ([#5](https://github.com/nabre/atollo/issues/5)). x64 et ARM64 sur Ubuntu 24.04, Fedora 42 et Debian 13 : AppImage avec et sans FUSE, le `tar.gz` avec WebKitGTK 4.1, clés exFAT ou montées `noexec`, connexions avec GNOME, KeePassXC ou KWallet.
- **Mises à jour et builds du kit hors de Windows.** La mise à jour de l'application d'une version à l'autre sur macOS et Linux n'est pas encore essayée. L'Apache du kit (2.4.69, D-116) est dans le catalogue mais essayé seulement en CI ; de même pour le PHP du kit avec `pdo_sqlite` (8.4.26 et 8.5.11, D-133), que l'application utilise à partir de la 0.3.0.
- **Signature du code.** Sous macOS, seulement une signature ad hoc, sans notarisation Apple : il faut `xattr -dr com.apple.quarantine`. Sous Windows, la 0.7.2 sort sans signature : la signature SignPath reste à activer, et d'ici là SmartScreen peut avertir au premier lancement.

Vous avez un Mac Apple Silicon ou un PC Linux ? Essayez Atollo et racontez-nous comment ça s'est passé : [ouvrez une issue sur GitHub](https://github.com/nabre/atollo/issues).

## Limites connues

- Sur Windows 10 et 11 (`x64`), macOS sur Apple Silicon et Linux (`x64` et `ARM64`) : les systèmes où tourne Docker Sandboxes, le cœur d'Atollo ([D-176](app/DECISIONS.md)). Sur tout autre système, Atollo le dit au démarrage et s'arrête. Sur Linux ARM64 Android manque. Les versions publiées pour macOS et Linux sont essayées en CI mais pas encore sur des machines propres ([#5](https://github.com/nabre/atollo/issues/5)), et la connexion des agents reste à essayer sur un Mac ([#6](https://github.com/nabre/atollo/issues/6)) ; composants et sources par système dans [docs/app/CATALOG.md](app/CATALOG.md) (en anglais), la matrice composant × système dans [docs/app/TOOLS-MATRIX.html](app/TOOLS-MATRIX.html). Apache sur macOS et Linux est un build du kit (D-116), avec PHP via `php-fpm`.
- La VM (Docker Sandboxes) ne tourne que là où ses [prérequis](#prérequis-de-la-vm) sont remplis ; ailleurs (Windows 10, macOS avant 14, autres distributions Linux, virtualisation désactivée, un kit sur FAT32 ou exFAT) les projets tournent sur le PC avec l'avertissement **Sans VM**, et les terminaux de l'éditeur et les agents ne démarrent pas. Le disque de chaque sandbox peut grandir jusqu'à environ 30 Go (fichiers creux : seul ce qui est écrit occupe de la place). Un Docker Sandboxes déjà actif sur le PC (installé sur le PC, ou d'un autre kit) garde la VM du kit éteinte jusqu'à ce que vous l'arrêtiez : Atollo ne l'arrête jamais et ne l'utilise jamais.
- Sans VM, il n'y a pas de frontière entre un projet et votre utilisateur : ses programmes peuvent lire vos fichiers et utiliser le réseau ; Atollo ne filtre que leur environnement. N'ouvrez dans Atollo que des projets de confiance.
- Sous Linux, MySQL a besoin de deux bibliothèques du système, `libaio` et `libnuma` : sur Ubuntu 24.04 et Debian 13 `sudo apt install libaio1t64 libnuma1` (sur les versions précédentes `libaio1`), sur Fedora et RHEL `sudo dnf install libaio numactl-libs`. Si elles manquent, Atollo le dit avec la commande à exécuter.
- Sur macOS et Linux, un dossier Atollo sur exFAT ou FAT (une clé formatée pour Windows) fonctionne, mais ces systèmes de fichiers n'ont pas de liens : là où un programme en apporte, Atollo copie les fichiers (par exemple les 142 liens internes de Git), ce qui prend plus de place. Sur une clé utilisée seulement avec un Mac ou Linux, mieux vaut APFS ou ext4.
- Hors du dossier d'Atollo, il ne reste que les traces de Windows, les fichiers temporaires de transit dans le dossier temporaire du système (supprimés automatiquement à la fermeture ; avec Paramètres › Général › Fichiers temporaires des programmes, les programmes peuvent aussi l'utiliser pendant qu'ils tournent), la connexion de GitHub CLI dans le Gestionnaire d'identification si vous la faites dans une console avec `gh auth login` (celle faite dans les Paramètres reste dans `data\gh`), celle d'Antigravity pendant qu'`agy` est ouvert (pour la session seulement), sur macOS la connexion de Claude Code dans le Trousseau (sauf si vous vous connectez avec un jeton) et les données de la vue web dans votre Bibliothèque, et l'entrée de registre que rustup écrit sous Windows, qu'Atollo supprime. Pour la VM : l'hyperviseur (activé une fois par vous en administrateur), la connexion à Docker (`sbx login`, dans le magasin d'identifiants du système, une fois par PC), la commande (pas le jeton) de « GitHub dans la VM » pour chaque sandbox dans le magasin d'identifiants du système, une jonction `sboxd-<id>` et un dossier `gvisor` dans le dossier temporaire du système (les sockets du démon) et, sous Ubuntu, le profil AppArmor du shim du kit ; sandbox, disques, images, réglages et journaux restent dans `data`. La liste complète est dans [PRIVACY §4](PRIVACY.fr.md#4-les-données-enregistrées-sur-votre-appareil).
- Le dossier d'un projet devient fiable pour Claude Code, Codex et Pi quand vous y ouvrez un agent depuis Atollo (les autres dossiers de `projects\` non) : n'ouvrez un agent que dans des projets de confiance. Les jetons des agents (`CLAUDE_CODE_OAUTH_TOKEN`, `GEMINI_API_KEY`, `LLAMA_API_KEY` des modèles locaux) ne vont qu'aux agents lancés par Atollo, ni aux actions des projets, ni aux scripts, ni aux terminaux, ni à l'éditeur ; la clé des modèles locaux est aussi écrite dans le `models.json` de Pi (`data\home\.pi\agent`, lisible par votre seul utilisateur). L'éditeur, lui, demande à la première ouverture de chaque projet et ouvre en « mode restreint » ceux auxquels vous ne faites pas confiance ; ses terminaux ne tournent que dans la VM du projet.
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

Pour une sauvegarde, utilisez Paramètres › Le kit › **Copie du kit** : elle exporte `projects\` et `data\` dans un dossier ou un zip, sans `data\apps\`, `data\cache\` ni `data\models\` (qui se téléchargent à nouveau) ni les images et les disques des VM (chaque sandbox est refaite depuis son kit), avec les bases de données arrêtées pendant la copie, et importe une telle copie dans un kit à son démarrage suivant. À la main, copiez les mêmes dossiers avec Atollo fermé. La copie contient les mots de passe du kit : gardez-la en lieu sûr.

## Désinstallation

Atollo n'utilise pas d'installateur, n'enregistre aucun service système ni tâche planifiée et ne modifie ni le registre ni le `PATH` de Windows :

1. Ouvrez la page **Retirer le lecteur** (ou fermez l'application) pour arrêter tous les processus lancés par Atollo, y compris le Docker Sandboxes du kit et ses microVM, et rapatrier dans `data\` la session éventuelle d'Antigravity.
2. Supprimez le dossier contenant `Atollo.exe`, `data\` et `projects\` (après avoir sauvegardé les projets que vous souhaitez conserver) : les sandbox, leurs disques et les réglages de Docker Sandboxes partent avec lui.
3. Les éventuels dossiers temporaires dans `%TEMP%` sont automatiquement supprimés à la fermeture ; si vous aviez exécuté `gh auth login` manuellement depuis une console (au lieu de la page **Paramètres**), vous pouvez supprimer l'entrée correspondante du **Gestionnaire d'identification** de Windows.
4. Ce que la VM laisse hors du dossier (voir [Limites connues](#limites-connues)), vous le supprimez à la main si vous le souhaitez : la connexion à Docker et les entrées `com.docker.sandboxes` du Gestionnaire d'identification (ou du trousseau du système), la jonction `sboxd-<id>` et le dossier `gvisor` dans le dossier temporaire du système, le profil AppArmor sous Ubuntu ; Windows Hypervisor Platform reste activé jusqu'à ce que vous le désactiviez.

## Ligne de commande

```
Atollo.exe                           ouvre la fenêtre
Atollo.exe env [--project <nom>]     affiche l'environnement que reçoivent les programmes d'Atollo
                                     (avec un projet, avec ses versions)
Atollo.exe components [--target <système>]
                                     liste les langages et outils, avec ceux installés
Atollo.exe versions [<id>]           les versions sur le disque ; avec un id, celles à télécharger
Atollo.exe check                     vérifie que les programmes installés répondent
    --kit                            la santé du kit : disque, espace, chemin, permissions, bibliothèques
    --files                          les fichiers des programmes : manquants ou modifiés depuis l'installation
Atollo.exe prepare [options]         installe et met à jour les composants choisis
                                     et supprime ceux qui ne le sont pas
    --only a,b                       seulement ceux-ci (et ce dont ils ont besoin), ne supprime rien
    --skip a,b                       pas ceux-ci
    --update-all                     met aussi à jour ceux déjà installés
    --force                          télécharge à nouveau
    --line id=x.y                    ajoute cette version à côté des autres (répétable)
    --target <système>               prépare pour un autre système : win32-x64, darwin-arm64,
                                     linux-x64, linux-arm64
Atollo.exe sync [--project <dossier>] [--verbose]
                                     aligne les instructions, skills et serveurs MCP des agents
Atollo.exe keyring restore|save      connexion d'Antigravity (utilisée par agy.cmd du kit)
Atollo.exe serve <dossier> <port>    serveur de fichiers pour un site statique
Atollo.exe models                    les modèles locaux : ceux du catalogue et les téléchargés
Atollo.exe models pull <modèle>      télécharge un id du catalogue ou un modèle de Hugging Face
                                     (propriétaire/nom:quantification), reprend s'il est interrompu
Atollo.exe models rm <id>            supprime un modèle téléchargé
Atollo.exe llm start|stop|status     le serveur des modèles locaux (chat et agents)
Atollo.exe diagnostics               crée le zip à joindre à un signalement (secrets masqués)
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

`npm run lint` exécute `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint et Prettier ; `npm test` exécute `cargo test` et les tests de l'interface ; `npm -w ui run build` vérifie les types et compile l'interface. Détails (en anglais) dans [docs/app/TESTING.md](app/TESTING.md). Organisation du code et où placer une modification (en anglais) : [docs/app/ARCHITECTURE.md](app/ARCHITECTURE.md).

Développement avec rechargement automatique : `npm run dev`.

## Si Windows bloque l'exe

Tant que le certificat de signature n'est pas actif sur la version téléchargée, si SmartScreen affiche « Windows a protégé votre ordinateur », choisissez **Informations complémentaires** › **Exécuter quand même**. Si l'exe a été téléchargé depuis Internet : clic droit › **Propriétés** › **Débloquer**. Sur les PC d'entreprise avec AppLocker, il faut l'autorisation du service informatique.

## Code signing policy

Free code signing on Windows provided by [SignPath.io](https://signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

- **Rôles du projet (Committer, Reviewer et Approver) :** [nabre](https://github.com/nabre)
- **Provenance des binaires :** les exécutables des versions officielles sont compilés et signés exclusivement par le pipeline automatisé GitHub Actions ([`.github/workflows/app.yml`](../.github/workflows/app.yml)) à partir du code source public de ce dépôt, sans intervention manuelle sur les binaires.

## Privacy policy

Le texte complet, avec chaque connexion réseau et chaque fichier que l'application conserve, se trouve dans **[Politique de confidentialité et conditions d'utilisation](PRIVACY.fr.md)**. En bref :

- **Aucune télémétrie ni collecte de données :** l'application ne collecte aucune donnée personnelle, statistique d'utilisation, télémétrie ou rapport de plantage, et ne contacte aucun serveur géré par l'auteur du projet. Elle désactive également la télémétrie intégrée des composants gérés (tels que PowerShell 7 et le serveur VSCodium).
- **Stockage local :** les paramètres, projets, bases de données locales, journaux, identifiants et les sandbox et disques des microVM sont enregistrés en local dans le dossier `data\` (et `projects\`) d'Atollo sur l'appareil de l'utilisateur ; les quelques exceptions (fichiers temporaires, la connexion d'Antigravity pendant son exécution, la connexion à Docker et les autres traces de Docker Sandboxes) sont listées dans la politique.
- **Docker Sandboxes :** la VM demande un compte Docker, connecté avec `sbx login`. Atollo lance le `sbx` du kit avec la télémétrie de Docker désactivée (`SBX_NO_TELEMETRY=1`), l'envoi du diagnostic désactivé, le transfert de l'agent SSH désactivé et la politique réseau `balanced` ; `sbx` lui-même parle avec Docker selon la [politique de confidentialité de Docker](https://www.docker.com/legal/docker-privacy-policy/).
- **Connexions réseau :** l'application elle-même ne se connecte qu'à GitHub pour rechercher ses propres mises à jour (Paramètres › Général › Mise à jour automatique, ou `ATOLLO_NO_UPDATE`), aux sources officielles des composants qu'elle télécharge, au registre Open VSX pour les extensions de l'éditeur et, lorsque l'utilisateur se connecte à GitHub (Paramètres › Compte GitHub) ou ouvre la page IA, à GitHub et aux fournisseurs des agents (**Claude Code**, **Codex**, **Antigravity** et le fournisseur cloud choisi dans **Pi**) selon leurs propres politiques de confidentialité. Les modèles locaux viennent de Hugging Face (`huggingface.co` et son CDN), les programmes llama.cpp et Pi de leurs versions publiées sur GitHub (`ggml-org/llama.cpp`, `earendil-works/pi`) ; un modèle local tourne sur le PC, et ses prompts ne le quittent jamais.

## Licence

MIT : fourni « tel quel », sans garantie. Voir [LICENSE](../LICENSE) et les [conditions d'utilisation](PRIVACY.fr.md#8-conditions-dutilisation).
