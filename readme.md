# Shooting Camera

`shooting-camera` is a desktop application written in Rust and Slint for analyzing shot impacts on a target from a USB camera feed.

The application displays a live camera image, lets you calibrate the target manually, place and edit shot impacts, and compute shooting metrics such as:

- group size / dispersion
- offset from point of aim
- average point of impact (MPI)
- angular values in **mrad** and **MOA**

## Features

### Camera
- Detects available cameras through `nokhwa`
- Lets you select the active camera from the UI
- Streams the live video feed into the Slint interface

### Calibration
- Manual target calibration from the live image
- Click-based workflow for:
  - target center
  - horizontal reference
  - vertical reference
- Separate horizontal and vertical calibration distances
- Visual calibration overlay and point labels

### Impacts
- Add impacts by clicking directly on the image
- Select impacts either:
  - on the target image
  - in the impacts list
- Move a selected impact
- Delete a selected impact
- Clear all impacts
- Automatic renumbering after deletion

### Metrics
- **Average point of impact (MPI)** shown as a **blue marker**
- **Groupement center** computed from the **smallest enclosing circle center**
- Dispersion / group size
- Offset from point of aim
- Values displayed in:
  - **mrad**
  - **MOA**

## Tech stack

- **Rust**
- **Slint** for the GUI
- **Nokhwa** for camera access
- **image** for frame/image handling

## Project structure

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

## Requirements

- Rust toolchain
- A supported USB camera
- A desktop environment capable of running Slint

## Build

```bash
cargo build
```

## Run

```bash
cargo run
```

## Test

```bash
cargo test
```

At the time of writing, the project test suite passes.

## How to use

### 1. Select a camera
- Expand the `Caméras` section
- Choose the desired device
- The section auto-collapses after selection

### 2. Configure shooting and calibration
- Expand `Configuration de tir`
- Set the shooting distance
- Enter calibration distances:
  - horizontal reference distance
  - vertical reference distance

### 3. Calibrate the target
Click `Calibrer la cible`, then click the image in this order:
1. target center
2. first horizontal reference point
3. second horizontal reference point
4. first vertical reference point
5. second vertical reference point

### 4. Capture the calibrated target view
- Click `Capturer la cible`
- The calibration popup disappears, but the calibration remains active

### 5. Add impacts
- Click on the image to place each impact
- The application stores image coordinates and calibrated target coordinates

### 6. Edit impacts
- Select an impact from the image or list
- Use:
  - `Déplacer` to reposition it
  - `Supprimer` to delete it
- All metrics are recalculated automatically

## Current metric definitions

### Average point of impact (MPI)
The MPI is the arithmetic mean of all calibrated impacts.

### Groupement center
The groupement center is defined as the **center of the smallest enclosing circle**.

### Dispersion
The group diameter is computed as the **maximum distance between two impacts**.

## Current limitations

This project is still a prototype / measurement tool and has some important limitations:

- Calibration is manual
- No automatic impact detection
- No session persistence
- No perspective correction / homography yet
- No target/image export workflow yet
- The impacts list is compact but not yet a fully scrollable advanced list widget

## Notes

- Impact calculations require a valid calibration
- Some internal code warnings may still exist for currently unused helper functions or exports
- The README describes the current implemented behavior, not the earlier design notes originally stored in this file

## License

This project is licensed under the **GNU General Public License v3.0 (GPLv3)**.

See the [`LICENSE`](LICENSE) file for the full license text.
