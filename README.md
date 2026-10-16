# rustake

`rustake` est un outil de build moderne inspiré de GNU `make`, entièrement développé en Rust. Contrairement à `make` qui s'appuie sur les timestamps (dates de modification) des fichiers pour déterminer si une cible doit être reconstruite, `rustake` utilise un système de **hachage** et prend en charge l'**exécution parallèle** des tâches indépendantes.

---

## Fonctionnalités

- **Configuration TOML** : Déclaration simple des cibles, dépendances et commandes à éxecuter dans un fichier `Rustakefile.toml`.
- **Détection par Hash (Blake3)** : Identification fiable des changements de fichiers sources grâce à la crate `blake3`, évitant les reconstructions inutiles.
- **Analyse de Dépendances Cycliques** : Détection automatique des cycles de dépendances via un tri topologique.
- **Builds Parallèles par Niveau** : Analyse du graphe de dépendances permettant de regrouper et d'exécuter simultanément (via des threads) les cibles de même niveau hiérarchique.
- **Gestion de Cache** : Sauvegarde locale des empreintes dans `.rustake/cache` au format TOML pour garder le même état entre chaque exécution.

---

## Architecture et Fonctionnement

Le projet est modulaire et découpé de la manière suivante :

1. **CLI (`src/cli.rs`)** : Gère l'interface en ligne de commande avec `clap` (commandes `init`, `clean` ou build par défaut).
2. **Configuration (`src/config.rs`)** : S'occupe du parsing du fichier `Rustakefile.toml` via `serde` et `toml`.
3. **Graph (`src/graph.rs`)** : Implémente l'algorithme de tri topologique par niveau pour calculer l'ordre d'exécution.
4. **Hachage (`src/update.rs`)** : Calcule l'empreinte de chaque fichier avec `blake3`.
5. **Cache (`src/cache.rs`)** : Assure la persistance et la lecture des empreintes sur le disque.
6. **Moteur principal (`src/main.rs`)** : Organise le build, lance les threads pour chaque niveau du graphe, et invoque les commandes système.


L'ordonnancement découpe le graphe en sous-listes de tâches indépendantes. Toutes les tâches d'un niveau $N$ sont exécutées en parallèle au sein de threads natifs ; le moteur attend la complétion de l'ensemble du niveau avant de passer au niveau $N+1$.

---

## Installation et Prérequis

Assurez-vous d'avoir installé **Rust** (et `cargo`).

1. Clonez ce dépôt dans le répertoir de votre choix.
2. Compilez le projet en mode release :
   ```bash
   cargo build --release
3. Le binaire généré se trouve dans `target/release/rustake`. Vous pouvez l'ajouter à votre `PATH` ou l'utiliser localement.

## Utilisation

`rustake` propose trois commandes principales gérées via l'interface en ligne de commande :

### 1. Initialiser un projet
Pour créer un fichier de configuration par défaut dans le répertoire courant, exécutez :
```bash
rustake init
```
#### Comportement : 
Cette commande vérifie si un fichier `Rustakefile.toml` existe déjà. Si ce n'est pas le cas, elle génère un fichier d'exemple contenant trois cibles standard (program, test, run). Si le fichier est déjà présent, elle affiche un message d'erreur et ne l'écrase pas.

### 2. Exécuter le build
Pour lancer l'analyse du graphe et l'exécution des tâches, lancez simplement l'outil sans argument
```bash
rustake
```

#### Comportement :
Comportement : `rustake` charge la configuration, calcule l'ordre d'exécution à l'aide du tri topologique, puis lance les commandes associées aux cibles obsolètes ou modifiées en parallélisant les tâches d'un même niveau. Les empreintes des fichiers sources et cibles réussies sont ensuite sauvegardées dans le cache.

### 3. Nettoyer les fichiers compilés
Pour réinitialiser l'état de votre projet, utilisez
```bash
rustake clean
```

#### Comportement : 
Comportement : Cette commande parcourt les cibles définies dans votre configuration et supprime les fichiers correspondants présents sur le disque. Elle supprime également l'intégralité du dossier `.rustake/` contenant le cache des empreintes, forçant une reconstruction totale lors du prochain build.

## Tests : 
Le projet inclut une suite de tests unitaires pour valider les composants clés (parsing TOML, intégrité du cache et cohérence du hachage). Pour exécuter ces tests, utilisez l'outil standard de Rust :
```bash
cargo test
```

## Comparaison en temps entre make et rustake
Le projet inclut également la possibilité de comparer le temps que prennent make et rustake à build le projet, ici le mini-projet minigrep du TP6. Deux scénarios sont étudiés, un premier en partant d’un clean, simulant un projet entièrement recompilé, un second après l’exécution de la commande touch main.rs, afin de simuler une modification partielle du projet. Les résultats des mesures sont regroupés dans le fichier results.txt, situé à la racine du projet. Afin de lancer l'exécution, il faut lancer la commande :
```bash
./timeCompare.sh
```