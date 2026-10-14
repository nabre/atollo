# Dev-Kit

[Italiano](../README.md) · [English](README.en.md) · Français · [Deutsch](README.de.md)

Un environnement de développement portable pour Windows en un seul fichier : `Dev-Kit.exe`. Éditeur (VSCodium) intégré, agents Claude Code, Codex et Antigravity, langages et outils téléchargés à la demande. Aucune installation, aucun droit d'administrateur, aucun fichier hors du dossier du kit.

Au premier démarrage, l'application télécharge d'elle-même les composants système : PowerShell 7, GitHub CLI, l'éditeur et ce dont ils ont besoin (Git pour GitHub CLI). Le reste se choisit dans la page « Langages et outils », ou arrive quand il sert : Node.js avec une action `npm`, uv avec un serveur MCP `uvx`, et ainsi de suite.

> En développement. État et plan (en italien) : [app/PLAN.md](app/PLAN.md), nouveautés : [app/CHANGELOG.md](app/CHANGELOG.md).
> Inspiré de [nabre/dev-kit](https://github.com/nabre/dev-kit), dont il n'est pas une nouvelle version : c'est un produit distinct.

## Utilisation

1. Copiez `Dev-Kit.exe` dans un dossier vide : une clé USB, un disque externe ou un dossier de votre profil (espaces, parenthèses ou `&` dans le nom sont acceptés).
2. Double-cliquez. Au premier démarrage, choisissez agents, langages et outils : l'application les télécharge et les vérifie (somme de contrôle ou signature), puis crée `projects\` et `data\`.
3. Créez ou clonez un projet dans la page **Projets**. L'éditeur, le terminal et les agents s'ouvrent toujours dans le dossier d'un projet, jamais ailleurs.

## Pages

| Page | Contenu |
| --- | --- |
| **Accueil** | projets récents avec Éditeur, Terminal et agent ; avis (kit déplacé, autre copie du kit ouverte, installations en cours) ; emplacement du kit |
| **Projets** | nouveau projet, clonage depuis GitHub ou une adresse Git ; pour chaque projet son type, ses langages (avec leur logo) et les actions reconnues (`npm run dev`, `php -S`, `cargo run`, site statique…), avec port, sortie et ouverture dans le navigateur |
| **Agents** | Claude Code, Codex et Antigravity installés : version, connexion, limites de l'abonnement et consommation du jour et des 7 derniers jours ; instructions, skills et serveurs MCP partagés dans `data\agents` |
| **Langages et outils** | les composants avec leur logo, leurs dépendances et l'espace disque ; installer, mettre à jour, supprimer |
| **Paramètres** | langue, réseau (proxy, certificats, réseau local), clé Gemini, versions de Python, PHP et Rust, mises à jour de l'application |
| **Retirer le lecteur** | ferme tout ce que le kit a ouvert, ramène la connexion d'Antigravity dans le kit et permet d'éjecter la clé |

La console du kit est le Windows Terminal portable du kit (ou, dans les Paramètres, la console classique de Windows) avec le PowerShell 7 du kit et seulement l'environnement du kit : les programmes installés sur le PC ne sont pas visibles.

## Ce qui reste du Dev-Kit

Le menu en console de [nabre/dev-kit](https://github.com/nabre/dev-kit) disparaît : chacune de ses entrées a une page dans l'application (« Proxy et paramètres » est la page Paramètres) et la logique des scripts est en Rust dans l'exe. Les composants sont les mêmes (plus PowerShell 7 et l'éditeur intégré, moins VSCodium desktop et Windows Terminal), tout comme l'alignement des instructions, skills et MCP entre les trois agents et la relocalisation quand le kit change de dossier ou de lettre de lecteur. Les kits du Dev-Kit ne sont pas importés.

## Limites connues

- Windows 10 et 11 uniquement, x64 et ARM64 (sur ARM64, l'éditeur, PHP, Apache et TinyTeX tournent en émulation x64 ; R n'est pas disponible).
- Hors du kit, il ne reste que les traces de Windows, la connexion de GitHub CLI dans le Gestionnaire d'identification et celle d'Antigravity pendant qu'`agy` est ouvert (pour la session seulement).
- Les dossiers de `projects\` sont considérés comme fiables par Claude Code et Codex : n'ouvrez que des projets de confiance. L'éditeur, lui, demande à la première ouverture de chaque projet et ouvre en « mode restreint » ceux auxquels vous ne faites pas confiance.
- Les limites d'utilisation des agents viennent d'interfaces non documentées des fournisseurs et peuvent cesser de fonctionner après une de leurs mises à jour.
- « Retirer le lecteur » ne voit pas les programmes du PC qui gardent seulement un fichier du kit ouvert.
- L'exe n'est pas signé (voir plus bas).

## Structure

```
Dev-Kit\
  Dev-Kit.exe     toute la logique
  projects\       vos projets
  data\           programmes téléchargés, paramètres, connexions, cache, fichiers temporaires
                  (les paramètres du kit de chaque projet : data\app\projects\<projet>\dev-settings.json)
```

Pour une sauvegarde, copiez `projects\` et `data\` (sans `data\apps\` ni `data\cache\`, qui se téléchargent à nouveau).

## Ligne de commande

```
Dev-Kit.exe                          ouvre la fenêtre
Dev-Kit.exe env                      affiche l'environnement que reçoivent les programmes du kit
Dev-Kit.exe components               liste les langages et outils, avec ceux installés
Dev-Kit.exe check                    vérifie que les programmes installés répondent
Dev-Kit.exe prepare [options]        installe, met à jour et supprime les composants choisis
    --only a,b                       seulement ceux-ci (et ce dont ils ont besoin)
    --skip a,b                       pas ceux-ci
    --update-all                     met aussi à jour ceux déjà installés
    --force                          télécharge à nouveau
Dev-Kit.exe sync [--project <dossier>] [--verbose]
                                     aligne instructions, skills et MCP des trois agents
Dev-Kit.exe keyring restore|save     connexion d'Antigravity (utilisée par agy.cmd du kit)
Dev-Kit.exe serve <dossier> <port>   serveur de fichiers pour un site statique
Dev-Kit.exe version                  affiche la version
Dev-Kit.exe help                     affiche l'aide
```

## Compiler

Prérequis : Rust stable (au moins 1.93) avec la chaîne MSVC (Visual Studio Build Tools, charge de travail C++), Node.js LTS.

```
npm ci
npm run build
```

Les dépendances JavaScript sont seulement à la racine (`node_modules` et `package-lock.json`) : `ui` est un workspace npm.

L'exe se trouve dans `target\release\Dev-Kit.exe`. Aucun installateur n'est créé (`--no-bundle`).

Contrôles avant une modification :

```
npm run lint
npm test
npm -w ui run build
```

`npm run lint` exécute `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint et Prettier ; `npm test` exécute `cargo test` et les tests de l'interface ; `npm -w ui run build` vérifie les types et compile l'interface. Détails (en italien) dans [app/TESTING.md](app/TESTING.md).

Développement avec rechargement automatique : `npm run dev`.

## Si Windows bloque l'exe

L'exe n'est pas encore signé. Si SmartScreen affiche « Windows a protégé votre ordinateur », choisissez **Informations complémentaires** › **Exécuter quand même**. Si l'exe a été téléchargé depuis Internet : clic droit › **Propriétés** › **Débloquer**. Sur les PC d'entreprise avec AppLocker, il faut l'autorisation du service informatique.

## Licence

MIT.
