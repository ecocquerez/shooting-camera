# Shooting Camera

`shooting-camera` est une application de bureau écrite en Rust et Slint permettant d’analyser des impacts de tir sur une cible à partir d’un flux vidéo provenant d’une caméra USB.

L’application affiche l’image en direct de la caméra, permet de calibrer manuellement la cible, d’ajouter et de modifier les impacts, et de calculer plusieurs métriques de tir, notamment :

- la taille du groupement / dispersion
- l’écart par rapport au point visé
- le point moyen d’impact (MPI)
- les valeurs angulaires en **mrad** et en **MOA**

## Fonctionnalités

### Caméra
- Détection des caméras disponibles via `nokhwa`
- Sélection de la caméra active depuis l’interface
- Affichage du flux vidéo en direct dans l’interface Slint

### Calibration
- Calibration manuelle de la cible à partir de l’image en direct
- Procédure par clics pour définir :
  - le centre de la cible
  - les repères horizontaux
  - les repères verticaux
- Distances de calibration horizontale et verticale distinctes
- Superposition visuelle de calibration avec étiquettes des points

### Impacts
- Ajout des impacts par clic direct sur l’image
- Sélection d’un impact :
  - sur l’image de la cible
  - dans la liste des impacts
- Déplacement d’un impact sélectionné
- Suppression d’un impact sélectionné
- Effacement de tous les impacts
- Renumérotation automatique après suppression

### Mesures
- **Point moyen d’impact (MPI)** affiché par un **marqueur bleu**
- **Centre du groupement** calculé à partir du **centre du plus petit cercle englobant**
- Dispersion / taille du groupement
- Écart par rapport au point visé
- Valeurs affichées en :
  - **mrad**
  - **MOA**

## Pile technique

- **Rust**
- **Slint** pour l’interface graphique
- **Nokhwa** pour l’accès à la caméra
- **image** pour la gestion des images et des frames

## Structure du projet

```text
shooting-camera/
├── Cargo.toml
├── build.rs
├── readme.md
└── src/
    ├── camera/
    │   ├── capture.rs
    │   ├── device.rs
    │   ├── format.rs
    │   └── mod.rs
    ├── cible/
    │   ├── calibration.rs
    │   ├── calibration_session.rs
    │   ├── geometry.rs
    │   ├── groupement.rs
    │   └── mod.rs
    ├── model/
    │   ├── impact.rs
    │   ├── point.rs
    │   └── mod.rs
    ├── ui/
    │   ├── app.slint
    │   ├── target_view.slint
    │   └── types.slint
    └── main.rs
```

## Prérequis

- Une toolchain Rust installée
- Une caméra USB compatible
- Un environnement de bureau capable d’exécuter une application Slint

## Compilation

```bash
cargo build
```

## Exécution

```bash
cargo run
```

## Tests

```bash
cargo test
```

Au moment de la rédaction, la suite de tests du projet passe correctement.

## Utilisation

### 1. Sélectionner une caméra
- Déplier la section `Caméras`
- Choisir le périphérique souhaité
- La section se replie automatiquement après la sélection

### 2. Configurer le tir et la calibration
- Déplier `Configuration de tir`
- Définir la distance de tir
- Saisir les distances de calibration :
  - distance de référence horizontale
  - distance de référence verticale

### 3. Calibrer la cible
Cliquer sur `Calibrer la cible`, puis cliquer sur l’image dans cet ordre :
1. centre de la cible
2. premier point de référence horizontal
3. second point de référence horizontal
4. premier point de référence vertical
5. second point de référence vertical

### 4. Capturer la vue calibrée
- Cliquer sur `Capturer la cible`
- La fenêtre verte de fin de calibration disparaît, mais la calibration reste active

### 5. Ajouter des impacts
- Cliquer sur l’image pour placer chaque impact
- L’application enregistre les coordonnées image et les coordonnées calibrées sur la cible

### 6. Modifier les impacts
- Sélectionner un impact depuis l’image ou la liste
- Utiliser :
  - `Déplacer` pour le repositionner
  - `Supprimer` pour le retirer
- Toutes les mesures sont recalculées automatiquement

## Définitions actuelles des mesures

### Point moyen d’impact (MPI)
Le MPI correspond à la moyenne arithmétique de tous les impacts calibrés.

### Centre du groupement
Le centre du groupement est défini comme le **centre du plus petit cercle englobant**.

### Dispersion
Le diamètre du groupement est calculé comme la **distance maximale entre deux impacts**.

## Limitations actuelles

Ce projet reste un prototype / outil de mesure et présente encore plusieurs limitations importantes :

- La calibration est manuelle
- Il n’y a pas de détection automatique des impacts
- Il n’y a pas de persistance des sessions
- Il n’y a pas encore de correction de perspective / homographie
- Il n’y a pas encore de fonction d’export de cible ou d’image
- La liste des impacts est compacte mais ne constitue pas encore un widget avancé totalement défilable

## Notes

- Les calculs d’impacts nécessitent une calibration valide
- Certains avertissements internes peuvent encore exister pour des fonctions auxiliaires ou des exports actuellement non utilisés
- Ce README décrit le comportement actuellement implémenté, et non les anciennes notes de conception initialement présentes dans ce fichier

## Licence

Ce projet est distribué sous licence **GNU General Public License v3.0 (GPLv3)**.

Voir le fichier [`LICENSE`](LICENSE) pour le texte complet de la licence.
