use super::Point;

/// Représente un impact sur la cible.
///
/// `position_image` est toujours disponible et correspond
/// aux coordonnées en pixels de l'image caméra.
///
/// `position_cible` correspond aux coordonnées en mm par
/// rapport au point visé. Elle est calculée après calibration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Impact {
    pub number: u32,
    pub position_image: Point,
    pub position_cible: Option<Point>,
}

impl Impact {
    /// Crée un impact à partir de sa position dans l'image.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new(number: u32, position_image: Point) -> Self {
        Self {
            number,
            position_image,
            position_cible: None,
        }
    }

    /// Crée directement un impact avec ses coordonnées
    /// image et cible.
    pub fn with_target_position(number: u32, position_image: Point, position_cible: Point) -> Self {
        Self {
            number,
            position_image,
            position_cible: Some(position_cible),
        }
    }

    /// Définit la position de l'impact dans le repère
    /// de la cible, en millimètres.
    pub fn set_target_position(&mut self, position: Point) {
        self.position_cible = Some(position);
    }

    /// Retourne la position calibrée si elle existe.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn target_position(&self) -> Option<Point> {
        self.position_cible
    }

    /// Indique si l'impact possède une position calibrée.
    pub fn is_calibrated(&self) -> bool {
        self.position_cible.is_some()
    }
}
