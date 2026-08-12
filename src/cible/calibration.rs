use crate::model::Point;

/// Calibration permettant de convertir les coordonnées
/// de l'image en coordonnées relatives au centre de la cible.
///
/// Le repère physique est centré sur le point visé :
///
///             Y
///             ^
///             |
///             |
///       ------+------> X
///             |
///             |
///
/// X positif : droite
/// Y positif : bas
///
/// Les coordonnées sont exprimées en millimètres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calibration {
    center: Point,
    pixels_per_mm_x: f32,
    pixels_per_mm_y: f32,
}

impl Calibration {
    /// Crée une calibration.
    ///
    /// `center` est le centre de la cible dans l'image.
    pub fn new(
        center: Point,
        pixel_distance_x: f32,
        real_distance_x: f32,
        pixel_distance_y: f32,
        real_distance_y: f32,
    ) -> Result<Self, CalibrationError> {
        if pixel_distance_x <= 0.0 || pixel_distance_y <= 0.0 {
            return Err(CalibrationError::InvalidPixelDistance);
        }

        if real_distance_x <= 0.0 || real_distance_y <= 0.0 {
            return Err(CalibrationError::InvalidRealDistance);
        }

        Ok(Self {
            center,
            pixels_per_mm_x: pixel_distance_x / real_distance_x,
            pixels_per_mm_y: pixel_distance_y / real_distance_y,
        })
    }
    /// Construit une calibration à partir de deux références :
    ///
    /// - une référence horizontale ;
    /// - une référence verticale.
    ///
    /// Les distances réelles sont exprimées en millimètres.
    pub fn from_references(
        center: Point,
        horizontal_a: Point,
        horizontal_b: Point,
        horizontal_distance_mm: f32,
        vertical_a: Point,
        vertical_b: Point,
        vertical_distance_mm: f32,
    ) -> Result<Self, CalibrationError> {
        let pixel_distance_x = (horizontal_b.x - horizontal_a.x).abs();

        let pixel_distance_y = (vertical_b.y - vertical_a.y).abs();

        if pixel_distance_x <= f32::EPSILON {
            return Err(CalibrationError::InvalidHorizontalReference);
        }

        if pixel_distance_y <= f32::EPSILON {
            return Err(CalibrationError::InvalidVerticalReference);
        }

        Self::new(
            center,
            pixel_distance_x,
            horizontal_distance_mm,
            pixel_distance_y,
            vertical_distance_mm,
        )
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn pixels_per_mm_x(&self) -> f32 {
        self.pixels_per_mm_x
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn pixels_per_mm_y(&self) -> f32 {
        self.pixels_per_mm_y
    }

    /// Convertit une position image en coordonnées
    /// relatives au centre de la cible, exprimées en mm.
    pub fn pixel_to_mm(&self, point: Point) -> Point {
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;

        Point {
            x: dx / self.pixels_per_mm_x,
            y: dy / self.pixels_per_mm_y,
        }
    }

    /// Convertit une position exprimée en mm par rapport
    /// au centre de la cible vers les coordonnées image.
    pub fn mm_to_pixel(&self, point: Point) -> Point {
        Point {
            x: self.center.x + point.x * self.pixels_per_mm_x,

            y: self.center.y + point.y * self.pixels_per_mm_y,
        }
    }

    /// Distance horizontale en pixels → mm.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn pixel_distance_x_to_mm(&self, distance: f32) -> f32 {
        distance / self.pixels_per_mm_x
    }

    /// Distance verticale en pixels → mm.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn pixel_distance_y_to_mm(&self, distance: f32) -> f32 {
        distance / self.pixels_per_mm_y
    }

    /// Calcule directement la distance d'un point image
    /// au centre de la cible.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn distance_to_center(&self, point: Point) -> f32 {
        let relative = self.pixel_to_mm(point);

        (relative.x * relative.x + relative.y * relative.y).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalibrationError {
    InvalidPixelDistance,
    InvalidRealDistance,
    InvalidHorizontalReference,
    InvalidVerticalReference,
    IncompleteCalibration,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calibration() -> Calibration {
        Calibration::new(Point::new(360.0, 240.0), 360.0, 100.0, 240.0, 100.0).unwrap()
    }

    #[test]
    fn centre_est_origine() {
        let calibration = calibration();

        let result = calibration.pixel_to_mm(Point::new(360.0, 240.0));

        assert!((result.x).abs() < 0.001);
        assert!((result.y).abs() < 0.001);
    }

    #[test]
    fn conversion_pixel_vers_mm() {
        let calibration = calibration();

        let result = calibration.pixel_to_mm(Point::new(540.0, 360.0));

        assert!((result.x - 50.0).abs() < 0.001);
        assert!((result.y - 50.0).abs() < 0.001);
    }

    #[test]
    fn conversion_mm_vers_pixel() {
        let calibration = calibration();

        let result = calibration.mm_to_pixel(Point::new(50.0, 50.0));

        assert!((result.x - 540.0).abs() < 0.001);
        assert!((result.y - 360.0).abs() < 0.001);
    }

    #[test]
    fn conversion_est_reversible() {
        let calibration = calibration();

        let original = Point::new(420.0, 300.0);

        let mm = calibration.pixel_to_mm(original);
        let result = calibration.mm_to_pixel(mm);

        assert!((result.x - original.x).abs() < 0.001);
        assert!((result.y - original.y).abs() < 0.001);
    }

    #[test]
    fn distance_au_centre() {
        let calibration = calibration();

        let impact = Point::new(396.0, 288.0);

        let distance = calibration.distance_to_center(impact);

        // 36 px / 3.6 = 10 mm
        // 48 px / 2.4 = 20 mm
        // distance = sqrt(10² + 20²)
        assert!((distance - 22.36068).abs() < 0.001);
    }

    #[test]
    fn distance_x() {
        let calibration = calibration();

        let distance = calibration.pixel_distance_x_to_mm(180.0);

        assert!((distance - 50.0).abs() < 0.001);
    }

    #[test]
    fn distance_y() {
        let calibration = calibration();

        let distance = calibration.pixel_distance_y_to_mm(120.0);

        assert!((distance - 50.0).abs() < 0.001);
    }

    #[test]
    fn distance_pixel_invalide() {
        let result = Calibration::new(Point::new(360.0, 240.0), 0.0, 100.0, 240.0, 100.0);

        assert_eq!(result, Err(CalibrationError::InvalidPixelDistance));
    }

    #[test]
    fn distance_reelle_invalide() {
        let result = Calibration::new(Point::new(360.0, 240.0), 360.0, 0.0, 240.0, 100.0);

        assert_eq!(result, Err(CalibrationError::InvalidRealDistance));
    }
    #[test]
    fn calibration_depuis_references() {
        let calibration = Calibration::from_references(
            Point::new(360.0, 240.0),
            // Référence horizontale :
            Point::new(180.0, 240.0),
            Point::new(540.0, 240.0),
            100.0,
            // Référence verticale :
            Point::new(360.0, 120.0),
            Point::new(360.0, 360.0),
            100.0,
        )
        .unwrap();

        assert!((calibration.pixels_per_mm_x() - 3.6).abs() < 0.001);

        assert!((calibration.pixels_per_mm_y() - 2.4).abs() < 0.001);
    }
    #[test]
    fn calibration_references_convertit_correctement() {
        let calibration = Calibration::from_references(
            Point::new(360.0, 240.0),
            Point::new(180.0, 240.0),
            Point::new(540.0, 240.0),
            100.0,
            Point::new(360.0, 120.0),
            Point::new(360.0, 360.0),
            100.0,
        )
        .unwrap();

        let position = calibration.pixel_to_mm(Point::new(396.0, 288.0));

        assert!((position.x - 10.0).abs() < 0.001);

        assert!((position.y - 20.0).abs() < 0.001);
    }
}
