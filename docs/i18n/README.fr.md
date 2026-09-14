<p align="center">
  <img src="../../desktop/src/assets/app-icon.png" width="88" alt="CueTuck">
</p>

# CueTuck · 唤词

**Your prompts, a shortcut away.**

[简体中文](../../README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Español](README.es.md) · **Français** · [Deutsch](README.de.md)

Un espace de travail de bureau qui privilégie le stockage local de vos prompts. Rangez modèles, images de référence et Skills pour agents ; recherchez un prompt, renseignez ses variables, puis copiez le résultat dans votre outil d’IA.

[Télécharger la préversion](https://github.com/sutao2/CueTuck/releases) · [Documentation](../../docs/INDEX.md) · [Signaler un problème](https://github.com/sutao2/CueTuck/issues)

![Catalogue de prompts : catégories, images, recherche et filtres](../../docs/assets/readme/square.png)

## Fonctionnalités

| Fonction | Usage |
|---|---|
| Catalogue de prompts | Explorer par catégorie, modèle ou mot-clé ; consulter images, sources et auteurs ; télécharger ou ajouter aux favoris. |
| Bibliothèque locale | Classer, rechercher, gérer les favoris et modifier textes et références. Basculer entre cartes et liste. |
| Lanceur indépendant | Ouvrir avec un raccourci global configurable, rechercher, remplir les variables et copier. Créer un prompt, l’améliorer avec l’IA ou lancer une recherche dans le catalogue. |
| Traduction et IA | Afficher les versions chinoise, originale et anglaise du catalogue. Utiliser votre propre modèle pour la traduction et l’amélioration locales, en le choisissant dans la liste récupérée auprès du fournisseur. |
| Gestion des Skills | Parcourir les sources publiques, découvrir les Skills locaux, gérer agent et portée, installer, restaurer les sauvegardes et rechercher manuellement les mises à jour. |
| Intégration MCP | Permettre aux agents compatibles de rechercher, lire et compléter les prompts locaux. Les outils du catalogue public sont facultatifs. |

## Du rangement à l’utilisation

### Des modèles conservés localement

La bibliothèque de bureau utilise SQLite. Les modèles acceptent des `{{variables}}` et des références. Connectez-vous lorsque vous souhaitez synchroniser votre bibliothèque, publier ou enregistrer des favoris du catalogue.

![Bibliothèque locale avec des exemples de prompts](../../docs/assets/readme/library.png)

### Renseigner, vérifier, copier

Adaptez un modèle à un nouvel objectif, un public ou un contenu. Vérifiez le texte complet avant de le copier, puis réutilisez le modèle la fois suivante.

![Aperçu d’un prompt après saisie des variables](../../docs/assets/readme/variables.png)

### À portée de raccourci

Le lanceur dispose de sa propre fenêtre et se pilote au clavier. Renseignez les variables et appuyez sur Entrée pour copier. Une nouvelle idée peut aussi devenir un prompt ou faire l’objet d’une amélioration par IA.

![Saisie de variables dans le lanceur indépendant](../../docs/assets/readme/launcher.png)

> Les captures proviennent du code actuel, dans l’aperçu navigateur. Le catalogue utilise du contenu public ; la bibliothèque et le lanceur présentent des exemples. SQLite natif, les raccourcis système et l’installation de fichiers nécessitent l’application de bureau. Les paquets publiés peuvent être moins récents que le code source.

## Installation et mises à jour

Téléchargez le fichier adapté sur [GitHub Releases](https://github.com/sutao2/CueTuck/releases).

| Plateforme | Paquet | Prise en charge |
|---|---|---|
| macOS · Apple Silicon | `.dmg` | arm64, vérifié sur un Mac réel |
| Windows | `.exe` | x64, installation, lancement et désinstallation vérifiés en CI |
| Linux | — | Pas encore vérifié |

**Paramètres → Mises à jour** permet de rechercher une version, suivre le téléchargement et confirmer l’installation. Il s’agit d’une préversion. Les paquets macOS utilisent une signature ad hoc, sans notarisation Apple ; consultez les [notes d’installation](../../deploy/README.md).

## Premiers pas

1. Créez un prompt local ou téléchargez un modèle du catalogue.
2. Choisissez **Utiliser**, renseignez les variables et copiez le résultat dans votre outil d’IA.
3. Réglez le raccourci, la langue et l’apparence dans les paramètres.
4. Dans **IA et modèles**, saisissez l’URL du service et votre clé API, récupérez la liste des modèles et choisissez-en un.
5. Pour synchroniser, publier ou utiliser les favoris du catalogue, connectez-vous et définissez un pseudonyme public.

Une configuration IA locale ne signifie pas une exécution hors ligne. Les clés sont conservées dans le gestionnaire d’identifiants du système ; les traductions et améliorations transmettent le texte concerné au fournisseur choisi. Vérifiez texte et pièces jointes publiques avant de publier.

## Skills et MCP

Gérez les dossiers de Skills pour Codex, Claude Code, Cursor, Pi et OpenCode, avec une portée globale ou par projet. Installation et remplacement comportent des étapes de confirmation et de sauvegarde. Installer un Skill n’installe pas ses dépendances MCP et n’exécute pas ses scripts. Voir la [spécification Skills](../../docs/specs/skills/spec.md).

MCP est un **service stdio en Rust**, compilé séparément, qui n’a besoin ni de Node.js ni d’uv à l’exécution. Par défaut, seuls les outils locaux en lecture sont exposés. Générez la configuration dans **Paramètres → Réseau et proxy → Intégration MCP des agents**, puis suivez le [guide MCP](../../docs/how-to/mcp-clients.md).

## Développement

Prévoyez Node.js 22+. L’application native nécessite aussi Rust et les dépendances de compilation Tauri propres à votre plateforme.

```bash
git clone https://github.com/sutao2/CueTuck.git
cd CueTuck/desktop
npm ci
npm test
npm run dev
```

Ces commandes lancent l’aperçu navigateur. Arrêtez-le avant d’exécuter `npm run tauri dev` pour l’application native.

[Développement local](../../docs/how-to/local-dev.md) · [Déploiement et distribution](../../deploy/README.md) · [Contribuer](../../CONTRIBUTING.md) · [Critères de test](../../docs/reference/test-gates.md)

La documentation technique est principalement en chinois. Pour signaler un bug, indiquez le système, la version, les étapes et des captures sans données personnelles. Ne joignez jamais de clés ou de jetons de session.
