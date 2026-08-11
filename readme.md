J'aimerai développer une application en Rust et Slint. Je récupérer une vidéo au travers d'une interface USB (vue comme une caméra). Cette caméra filme un cible placée à une certaine distance que je pourrais renseigner dans l'application. Sur cette video de cible, je dois pouvoir également pointer le centre de la cible, ainsi que la distance entre certain points remarquable, aussi bien en vertical qu'horizontal.
A chaque tir, je peux marquer l'impact sur la cible, le numéroter, en déduire la distance par rapport au point visé.
Je veux également pouvoir calculer le groupement des différents impacts

Oui. Et je te conseille de concevoir l'application comme un outil de mesure sur image, plutôt que comme une simple interface vidéo. Cela permettra ensuite d'ajouter facilement détection automatique des impacts, statistiques, sauvegarde des séances, etc.

Pour ton besoin, Rust + Slint + OpenCV est une combinaison très adaptée : Slint gère l'interface native, tandis qu'une couche caméra/vision traite les images. Slint sait notamment recevoir des images sous forme de SharedPixelBuffer depuis un thread de capture.

Architecture que je te propose
                 ┌──────────────────────────┐
                 │          Slint           │
                 │      Interface GUI       │
                 └────────────┬─────────────┘
                              │
                    commandes / événements
                              │
                 ┌────────────▼─────────────┐
                 │      Application Rust    │
                 │                          │
                 │  - configuration         │
                 │  - cible                 │
                 │  - impacts               │
                 │  - mesures                │
                 │  - statistiques          │
                 └───────┬─────────┬────────┘
                         │         │
             ┌───────────▼───┐ ┌──▼─────────────┐
             │ Camera        │ │ Vision /       │
             │               │ │ géométrie      │
             │ USB / UVC     │ │                │
             └───────────────┘ │ OpenCV         │
                               │ calibration    │
                               │ détection      │
                               └────────────────┘

Pour la caméra USB, Nokhwa est une bonne première candidate : elle fournit une abstraction Rust multiplateforme pour les webcams et expose notamment Camera, les résolutions, formats et acquisition des frames.

Documentation Slint Rust
Nokhwa – documentation Rust

1. Écran principal

Je verrais quelque chose comme :

┌──────────────────────────────────────────────────────────────┐
│ Session : 2026-08-11       Distance : [ 25.0 m ]            │
├──────────────────────────────────────┬───────────────────────┤
│                                      │ Configuration          │
│                                      │                       │
│          IMAGE DE LA CIBLE           │ Distance cible        │
│                                      │ [ 25.0 ] m            │
│          ┌─────────────┐             │                       │
│          │      •      │             │ Centre :              │
│          │   1    2    │             │ X : ... mm            │
│          │       3     │             │ Y : ... mm            │
│          │             │             │                       │
│          └─────────────┘             │ [Nouvel impact]       │
│                                      │ [Annuler]             │
│                                      │ [Calibration]          │
│                                      │                       │
├──────────────────────────────────────┴───────────────────────┤
│ Impacts : 8     Groupement : 32.4 mm     Moyenne : 14.2 mm │
└──────────────────────────────────────────────────────────────┘

L'image de la caméra serait affichée dans Slint, et les éléments graphiques — centre, repères, impacts, distances — seraient dessinés par-dessus.

Slint fournit justement Image et SharedPixelBuffer pour intégrer efficacement des images produites par du code Rust externe.

2. Il faut distinguer deux coordonnées

C'est un point très important pour ton projet.

La caméra donne des coordonnées en pixels :

(0,0) ─────────────────────────► X pixels
  │
  │        impact
  │          • (853,421)
  │
  │
  ▼
Y pixels

Mais l'utilisateur veut des distances physiques :

             +Y
              ↑
              │
       impact │
          •   │
              │
              ● centre
              │
              └──────────────► +X

          distance en mm

Il faut donc établir une transformation :

pixels ─────────► millimètres
Calibration simple

Si la cible est parfaitement perpendiculaire à la caméra et qu'il n'y a pratiquement pas de distorsion :

distance réelle
──────────────────── = mm / pixel
distance pixels

Par exemple :

Deux repères sont distants de 100 mm.

Sur l'image :
     742 pixels

Échelle :
     100 / 742
     = 0,13477 mm/pixel

On peut alors transformer :

dx_mm = dx_pixels * scale_x;
dy_mm = dy_pixels * scale_y;

Je prévoirais cependant deux échelles, X et Y, plutôt qu'une seule.

3. Calibration que je recommande

Je ne me limiterais pas à demander uniquement :

distance caméra → cible

Cette distance est utile pour enregistrer les conditions de la séance, mais elle ne suffit pas à déterminer une distance en millimètres sur l'image.

Le mieux est que l'utilisateur puisse définir au départ plusieurs repères connus.

Par exemple :

             A
             ●
             │
             │ 100 mm
             │
             ● B

L'utilisateur clique :

sur A
sur B
indique 100 mm

L'application calcule automatiquement l'échelle.

Encore mieux : utiliser 4 points de calibration.

A ●────────────────────● B
  │                    │
  │                    │
  │                    │
D ●────────────────────● C

avec les dimensions réelles :

AB = 300 mm
AD = 300 mm

Cela permettrait de corriger une éventuelle perspective de la cible.

4. Correction de perspective

C'est probablement la fonctionnalité qui fera la différence entre un prototype et une application réellement fiable.

Si la caméra n'est pas exactement perpendiculaire à la cible :

Image caméra

    ╲──────────────────╲
     ╲      cible       ╲
      ╲──────────────────╲

alors une distance horizontale réelle ne correspondra pas nécessairement à la même distance en pixels partout dans l'image.

On peut alors calculer une homographie et transformer l'image en vue "à plat".

Conceptuellement :

IMAGE CAMÉRA                    IMAGE RECTIFIÉE

╱──────────────╲                ┌──────────────┐
│              │       →        │              │
│    CIBLE     │                │    CIBLE     │
│              │                │              │
╲──────────────╱                └──────────────┘

Pour les calculs de distance, je travaillerais ensuite dans l'image rectifiée, pas dans l'image brute.

C'est typiquement une tâche pour OpenCV.

5. Définition du centre de la cible

Je te conseille de ne pas coder le centre comme une simple constante.

L'utilisateur devrait avoir un mode :

Définir le centre

Il clique au centre :

             │
             │
        ─────●─────
             │
             │

L'application mémorise :

Target {
    center: Point2D,
    ...
}

Par exemple :

struct Point2D {
    x: f64,
    y: f64,
}

Les coordonnées pourraient être conservées dans le système réel :

centre = (0.0, 0.0)

Puis chaque impact devient :

impact #1 = (+12.4, -7.8) mm
impact #2 = (-4.2, +5.1) mm
impact #3 = (+8.3, +2.7) mm
6. Marquage des impacts

Pour commencer, je ferais un marquage manuel.

L'utilisateur clique sur l'impact :

               ╳ 1

                      ╳ 2

          ╳ 3

L'application attribue automatiquement :

1
2
3
4
...

Et affiche éventuellement :

        1
       ╳
          ↘ 13.7 mm
             ● centre

Une liste latérale pourrait afficher :

N°	X	Y	Distance centre
1	+12.4	-7.8	14.6 mm
2	-4.2	+5.1	6.6 mm
3	+8.3	+2.7	8.7 mm
7. Calcul de la distance au point visé

C'est très simple une fois le système de coordonnées établi.

Si :

Centre = (0, 0)

Impact = (x, y)

alors :

distance = sqrt(x² + y²)

Par exemple :

x = 12.4 mm
y = -7.8 mm

distance = sqrt(12.4² + 7.8²)
         ≈ 14.65 mm

On pourra également afficher :

Écart horizontal : +12.4 mm
Écart vertical   : -7.8 mm
Écart radial     : 14.6 mm
8. Calcul du groupement

Je prévoirais plusieurs statistiques, plutôt qu'une seule définition du "groupement".

Centre du groupement

Pour N impacts :

Xg = Σ Xi / N
Yg = Σ Yi / N

Cela donne le centre moyen des impacts.

Graphiquement :

           ×
       ×       ×


            ●
         centre
        groupement

Cela permet de distinguer deux choses :

Précision / centrage

Où se trouve le centre du groupement par rapport au centre visé ?

et

Dispersion

À quel point les impacts sont-ils regroupés ?

9. Taille du groupement

Je calculerais au minimum :

Extrême dispersion

La distance maximale entre deux impacts :

G = max(distance(impact_i, impact_j))

Graphiquement :

      ×
      │
      │  ← groupement
      │
      ×

Par exemple :

Groupement : 32.4 mm
Distance moyenne au centre du groupement

On peut également calculer :

Rmean = moyenne(distance(impact, centre_groupement))

et éventuellement :

Rmax = distance maximale au centre du groupement
10. Une fonctionnalité intéressante : ellipse du groupement

Plus tard, on pourrait afficher une ellipse représentant la dispersion :

             _________
         .-´           `-.
       .'      ×          `.
      /   ×        ×       \
     |                       |
      \        ×            /
       `.                 .'
         `-.___________.-'

                ●
           centre moyen

On pourrait alors donner :

Axe majeur : 28.3 mm
Axe mineur : 17.1 mm
Orientation : 23°

C'est beaucoup plus informatif qu'un seul diamètre.

11. Architecture Rust

Je partirais sur quelque chose de ce genre :

src/
├── main.rs
│
├── camera/
│   ├── mod.rs
│   └── capture.rs
│
├── vision/
│   ├── mod.rs
│   ├── calibration.rs
│   ├── perspective.rs
│   └── image.rs
│
├── target/
│   ├── mod.rs
│   ├── target.rs
│   ├── impact.rs
│   └── grouping.rs
│
├── session/
│   ├── mod.rs
│   └── session.rs
│
└── ui/
    ├── mod.rs
    └── app.slint

Avec des structures métier indépendantes de Slint :

struct Target {
    center: Point2D,
    calibration: Calibration,
}

struct Calibration {
    scale_x: f64,
    scale_y: f64,
    // éventuellement homographie
}

struct Impact {
    number: u32,
    position: Point2D,
}

struct ShotGroup {
    impacts: Vec<Impact>,
}

struct GroupStatistics {
    center: Point2D,
    extreme_spread: f64,
    mean_radius: f64,
    max_radius: f64,
}

C'est important : les calculs ne doivent pas être dans le code Slint.

Slint reste responsable de l'affichage et des interactions utilisateur ; Rust contient le modèle et les calculs. C'est justement le mode de fonctionnement prévu par Slint.

12. Capture caméra

Pour un premier prototype, je testerais Nokhwa.

Il permet notamment :

énumération des caméras ;
sélection de la caméra ;
choix de résolution ;
choix du format ;
acquisition des frames ;
fonctionnement avec les backends natifs selon la plateforme.

On pourrait avoir :

camera.open_stream()?;

loop {
    let frame = camera.frame()?;

    // conversion RGB
    // traitement éventuel
    // envoi vers Slint
}

La capture tournerait dans un thread dédié.

Le thread caméra ne doit surtout pas bloquer la boucle événementielle Slint.

Le principe serait :

Thread caméra
     │
     │ Frame
     ▼
 traitement
     │
     ▼
 SharedPixelBuffer
     │
     ▼
 Slint UI

Slint documente précisément le transfert d'un SharedPixelBuffer produit dans un thread vers le thread UI via invoke_from_event_loop.

13. Je séparerais aussi "vidéo" et "image de mesure"

C'est important pour les performances.

Pendant que la caméra fonctionne :

30 FPS
 │
 ├──► affichage vidéo
 │
 └──► éventuellement traitement vision

Mais lorsqu'on veut mesurer :

           bouton
             │
             ▼
        ┌───────────┐
        │ Capturer  │
        └─────┬─────┘
              │
              ▼
        image figée
              │
              ▼
      calibration / impacts

Ainsi l'utilisateur peut arrêter une image de cible et travailler dessus sans que la caméra ne bouge les coordonnées sous ses yeux.

14. Détection automatique des impacts

Je ne commencerais pas par ça.

Version 1 :

Caméra
   ↓
Image
   ↓
Utilisateur clique
   ↓
Impact

Une fois le système géométrique fiable, on pourra ajouter :

Caméra
   ↓
Image avant tir
   ↓
Image après tir
   ↓
Différence d'images
   ↓
Détection d'une nouvelle marque
   ↓
Proposition d'impact
   ↓
Utilisateur valide

Cela permettrait à l'application de dire :

Nouvel impact détecté à 12.7 mm du centre.

L'utilisateur pourrait simplement cliquer sur Valider.

C'est une évolution beaucoup plus réaliste que d'essayer immédiatement de reconnaître tous les impacts avec de l'IA.

15. Persistance des séances

Je prévoirais dès le départ un modèle de session :

Session
│
├── date
├── cible
├── distance caméra/cible
├── calibration
├── centre
│
└── impacts
    ├── #1
    ├── #2
    ├── #3
    └── ...

Et sauvegarde :

session.json

ou une petite base SQLite si tu souhaites ensuite avoir :

Historique
─────────────────────────────
11/08/2026   25 m   10 impacts
10/08/2026   25 m   15 impacts
08/08/2026   50 m   10 impacts
16. MVP que je développerais en premier

Je découperais le développement en 6 étapes.

Étape 1 — Caméra
détecter les caméras USB ;
choisir une caméra ;
afficher le flux dans Slint ;
choisir résolution/FPS.
Étape 2 — Image interactive
afficher une image ;
zoom ;
déplacement ;
coordonnées souris ;
superposition de points et lignes.
Étape 3 — Calibration
définir deux ou quatre points ;
entrer les distances réelles ;
calculer mm/pixel ;
afficher une règle graduée.
Étape 4 — Cible
définir le centre ;
définir éventuellement les axes X/Y ;
enregistrer la cible.
Étape 5 — Impacts
clic pour créer un impact ;
numérotation automatique ;
coordonnées X/Y ;
distance au centre ;
suppression/déplacement d'un impact.
Étape 6 — Groupement
centre moyen ;
écart au centre visé ;
dispersion maximale ;
rayon moyen ;
ellipse ;
statistiques.

Et seulement ensuite, détection automatique des impacts.

Stack que je choisirais
Fonction	Technologie
Langage	Rust
Interface	Slint
Caméra USB	Nokhwa
Traitement image	OpenCV
Manipulation images	image / OpenCV
Persistance V1	JSON
Persistance V2	SQLite
Calcul géométrique	Rust pur
Thread caméra	std::thread / canaux
Tests	cargo test

Slint est actuellement en version 1.x et fournit une intégration Rust native ; la documentation officielle propose également un template Rust pour démarrer un projet.

Template Rust Slint
Dépôt officiel Slint

Et surtout

Je pense qu'il y a un très bon projet à construire progressivement ici. On peut commencer sans OpenCV complexe : caméra USB → image Slint → clics → calibration → impacts → groupement. Une fois cette base fonctionnelle, on ajoute la vision automatique.

Si tu veux, je peux directement te proposer le squelette complet du projet Cargo Rust + Slint, avec Cargo.toml, main.rs, app.slint, un module caméra et une première fenêtre affichant le flux USB avec possibilité de cliquer sur la cible.
