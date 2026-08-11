use super::calibration::{Calibration, CalibrationError};
use crate::model::Point;

/// Étape courante de la procédure de calibration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationStep {
    /// Aucune calibration en cours.
    None,

    /// Attente du clic sur le centre.
    Center,

    /// Attente du premier point horizontal.
    HorizontalFirst,

    /// Attente du second point horizontal.
    HorizontalSecond,

    /// Attente du premier point vertical.
    VerticalFirst,

    /// Attente du second point vertical.
    VerticalSecond,

    /// Les points ont tous été sélectionnés.
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalibrationClickResult {
    Ignored,
    Accepted,
    Completed,
    Failed(CalibrationError),
}

/// État temporaire d'une procédure de calibration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalibrationSession {
    step: CalibrationStep,

    center: Option<Point>,

    horizontal_a: Option<Point>,
    horizontal_b: Option<Point>,

    vertical_a: Option<Point>,
    vertical_b: Option<Point>,

    horizontal_distance_mm: f32,
    vertical_distance_mm: f32,

    calibration: Option<Calibration>,
}

impl Default for CalibrationSession {
    fn default() -> Self {
        Self::new()
    }
}

impl CalibrationSession {
    pub fn new() -> Self {
        Self {
            step: CalibrationStep::None,

            center: None,
            calibration: None,

            horizontal_a: None,
            horizontal_b: None,

            vertical_a: None,
            vertical_b: None,

            horizontal_distance_mm: 100.0,
            vertical_distance_mm: 100.0,
        }
    }

    pub fn calibration(&self) -> Option<&Calibration> {
        self.calibration.as_ref()
    }
    /// Retourne l'étape courante.
    pub fn step(&self) -> CalibrationStep {
        self.step
    }
    pub fn is_calibrated(&self) -> bool {
        self.calibration.is_some()
    }

    pub fn pixel_to_mm(&self, point: Point) -> Option<Point> {
        self.calibration
            .as_ref()
            .map(|calibration| calibration.pixel_to_mm(point))
    }
    /// Démarre une nouvelle procédure de calibration.
    ///
    /// Toutes les anciennes mesures sont supprimées.
    pub fn start(&mut self) {
        self.reset();

        self.step = CalibrationStep::Center;
    }

    /// Annule la calibration en cours.
    pub fn cancel(&mut self) {
        self.reset();
    }

    /// Réinitialise complètement la session.
    pub fn reset(&mut self) {
        self.step = CalibrationStep::None;

        self.center = None;
        self.calibration = None;

        self.horizontal_a = None;
        self.horizontal_b = None;

        self.vertical_a = None;
        self.vertical_b = None;
    }

    /// Traite un clic dans l'image.
    ///
    pub fn click(&mut self, point: Point) -> CalibrationClickResult {
        match self.step {
            CalibrationStep::None | CalibrationStep::Complete => CalibrationClickResult::Ignored,

            CalibrationStep::Center => {
                self.center = Some(point);
                self.step = CalibrationStep::HorizontalFirst;

                CalibrationClickResult::Accepted
            }

            CalibrationStep::HorizontalFirst => {
                self.horizontal_a = Some(point);
                self.step = CalibrationStep::HorizontalSecond;

                CalibrationClickResult::Accepted
            }

            CalibrationStep::HorizontalSecond => {
                self.horizontal_b = Some(point);
                self.step = CalibrationStep::VerticalFirst;

                CalibrationClickResult::Accepted
            }

            CalibrationStep::VerticalFirst => {
                self.vertical_a = Some(point);
                self.step = CalibrationStep::VerticalSecond;

                CalibrationClickResult::Accepted
            }

            CalibrationStep::VerticalSecond => {
                self.vertical_b = Some(point);

                let points = match (
                    self.center,
                    self.horizontal_a,
                    self.horizontal_b,
                    self.vertical_a,
                    self.vertical_b,
                ) {
                    (
                        Some(center),
                        Some(horizontal_a),
                        Some(horizontal_b),
                        Some(vertical_a),
                        Some(vertical_b),
                    ) => (center, horizontal_a, horizontal_b, vertical_a, vertical_b),

                    _ => {
                        return CalibrationClickResult::Failed(
                            CalibrationError::InvalidHorizontalReference,
                        );
                    }
                };

                match Calibration::from_references(
                    points.0,
                    points.1,
                    points.2,
                    self.horizontal_distance_mm,
                    points.3,
                    points.4,
                    self.vertical_distance_mm,
                ) {
                    Ok(calibration) => {
                        self.calibration = Some(calibration);
                        self.step = CalibrationStep::Complete;

                        CalibrationClickResult::Completed
                    }

                    Err(error) => CalibrationClickResult::Failed(error),
                }
            }
        }
    }

    pub fn center(&self) -> Option<Point> {
        self.center
    }

    pub fn horizontal_a(&self) -> Option<Point> {
        self.horizontal_a
    }

    pub fn horizontal_b(&self) -> Option<Point> {
        self.horizontal_b
    }

    pub fn vertical_a(&self) -> Option<Point> {
        self.vertical_a
    }

    pub fn vertical_b(&self) -> Option<Point> {
        self.vertical_b
    }

    /// Indique si tous les points nécessaires ont été sélectionnés.
    pub fn is_complete(&self) -> bool {
        self.step == CalibrationStep::Complete
    }

    /// Retourne tous les points sélectionnés.
    pub fn points(&self) -> Option<CalibrationPoints> {
        if !self.is_complete() {
            return None;
        }

        Some(CalibrationPoints {
            center: self.center?,
            horizontal_a: self.horizontal_a?,
            horizontal_b: self.horizontal_b?,
            vertical_a: self.vertical_a?,
            vertical_b: self.vertical_b?,
        })
    }
    pub fn build_calibration(
        &mut self,
        horizontal_distance_mm: f32,
        vertical_distance_mm: f32,
    ) -> Result<(), CalibrationError> {
        let points = self
            .points()
            .ok_or(CalibrationError::IncompleteCalibration)?;

        let calibration = Calibration::from_references(
            points.center,
            points.horizontal_a,
            points.horizontal_b,
            horizontal_distance_mm,
            points.vertical_a,
            points.vertical_b,
            vertical_distance_mm,
        )?;

        self.calibration = Some(calibration);

        Ok(())
    }
    pub fn set_reference_distances(
        &mut self,
        horizontal_mm: f32,
        vertical_mm: f32,
    ) -> Result<(), CalibrationError> {
        if horizontal_mm <= 0.0 || vertical_mm <= 0.0 {
            return Err(CalibrationError::InvalidRealDistance);
        }

        self.horizontal_distance_mm = horizontal_mm;
        self.vertical_distance_mm = vertical_mm;

        Ok(())
    }
}

/// Ensemble des points nécessaires pour créer une calibration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalibrationPoints {
    pub center: Point,

    pub horizontal_a: Point,
    pub horizontal_b: Point,

    pub vertical_a: Point,
    pub vertical_b: Point,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f32, y: f32) -> Point {
        Point::new(x, y)
    }

    #[test]
    fn nouvelle_session_est_inactive() {
        let session = CalibrationSession::new();

        assert_eq!(session.step(), CalibrationStep::None);

        assert!(!session.is_complete());
        assert_eq!(session.center(), None);
    }

    #[test]
    fn start_demarre_au_centre() {
        let mut session = CalibrationSession::new();

        session.start();

        assert_eq!(session.step(), CalibrationStep::Center);
    }

    #[test]
    fn clic_centre() {
        let mut session = CalibrationSession::new();

        session.start();

        assert_eq!(
            session.click(point(360.0, 240.0)),
            CalibrationClickResult::Accepted
        );

        assert_eq!(session.center(), Some(point(360.0, 240.0)));

        assert_eq!(session.step(), CalibrationStep::HorizontalFirst);
    }
    #[test]
    fn calibration_echoue_si_reference_horizontale_invalide() {
        let mut session = CalibrationSession::new();

        session.start();

        session.click(point(360.0, 240.0));

        // Même X : distance horizontale nulle.
        session.click(point(360.0, 240.0));
        session.click(point(360.0, 240.0));

        session.click(point(360.0, 120.0));
        let result = session.click(point(360.0, 360.0));

        assert_eq!(
            result,
            CalibrationClickResult::Failed(CalibrationError::InvalidHorizontalReference)
        );

        assert!(!session.is_complete());
        assert!(session.calibration().is_none());
    }
    #[test]
    fn sequence_complete() {
        let mut session = CalibrationSession::new();

        session.start();

        assert_eq!(
            session.click(point(360.0, 240.0)),
            CalibrationClickResult::Accepted
        );
        assert_eq!(
            session.click(point(180.0, 240.0)),
            CalibrationClickResult::Accepted
        );
        assert_eq!(
            session.click(point(540.0, 240.0)),
            CalibrationClickResult::Accepted
        );
        assert_eq!(
            session.click(point(360.0, 120.0)),
            CalibrationClickResult::Accepted
        );
        assert_eq!(
            session.click(point(360.0, 360.0)),
            CalibrationClickResult::Completed
        );

        assert!(session.is_complete());

        assert_eq!(session.step(), CalibrationStep::Complete);
    }

    #[test]
    fn points_complets() {
        let mut session = CalibrationSession::new();

        session.start();

        let center = point(360.0, 240.0);
        let horizontal_a = point(180.0, 240.0);
        let horizontal_b = point(540.0, 240.0);
        let vertical_a = point(360.0, 120.0);
        let vertical_b = point(360.0, 360.0);

        session.click(center);
        session.click(horizontal_a);
        session.click(horizontal_b);
        session.click(vertical_a);
        session.click(vertical_b);

        let points = session.points().unwrap();

        assert_eq!(points.center, center);
        assert_eq!(points.horizontal_a, horizontal_a);
        assert_eq!(points.horizontal_b, horizontal_b);
        assert_eq!(points.vertical_a, vertical_a);
        assert_eq!(points.vertical_b, vertical_b);
    }

    #[test]
    fn clic_sans_session_n_est_pas_consomme() {
        let mut session = CalibrationSession::new();

        assert_eq!(
            session.click(point(100.0, 100.0)),
            CalibrationClickResult::Ignored
        );

        assert_eq!(session.step(), CalibrationStep::None);
    }

    #[test]
    fn annulation() {
        let mut session = CalibrationSession::new();

        session.start();
        session.click(point(360.0, 240.0));

        session.cancel();

        assert_eq!(session.step(), CalibrationStep::None);

        assert_eq!(session.center(), None);
        assert!(!session.is_complete());
    }

    #[test]
    fn clic_apres_completion_n_est_pas_consomme() {
        let mut session = CalibrationSession::new();

        session.start();

        session.click(point(360.0, 240.0));
        session.click(point(180.0, 240.0));
        session.click(point(540.0, 240.0));
        session.click(point(360.0, 120.0));
        session.click(point(360.0, 360.0));

        assert_eq!(
            session.click(point(100.0, 100.0)),
            CalibrationClickResult::Ignored
        );
    }
    #[test]
    fn calibration_est_creee_apres_completion() {
        let mut session = CalibrationSession::new();

        session.start();

        session.click(point(360.0, 240.0));
        session.click(point(180.0, 240.0));
        session.click(point(540.0, 240.0));
        session.click(point(360.0, 120.0));
        session.click(point(360.0, 360.0));

        let result = session.build_calibration(100.0, 100.0);

        assert!(result.is_ok());
        assert!(session.calibration().is_some());
    }
    #[test]
    fn calibration_session_produit_les_bonnes_echelles() {
        let mut session = CalibrationSession::new();

        session.start();

        session.click(point(360.0, 240.0));
        session.click(point(180.0, 240.0));
        session.click(point(540.0, 240.0));
        session.click(point(360.0, 120.0));
        session.click(point(360.0, 360.0));

        session.build_calibration(100.0, 100.0).unwrap();

        let calibration = session.calibration().unwrap();

        assert!((calibration.pixels_per_mm_x() - 3.6).abs() < 0.001);

        assert!((calibration.pixels_per_mm_y() - 2.4).abs() < 0.001);
    }
    #[test]
    fn calibration_impossible_avant_completion() {
        let mut session = CalibrationSession::new();

        session.start();

        let result = session.build_calibration(100.0, 100.0);

        assert_eq!(result, Err(CalibrationError::IncompleteCalibration));

        assert!(session.calibration().is_none());
    }
    #[test]
    fn distances_de_reference_par_defaut() {
        let session = CalibrationSession::new();

        assert_eq!(session.horizontal_distance_mm, 100.0);
        assert_eq!(session.vertical_distance_mm, 100.0);
    }
    #[test]
    fn distances_de_reference_peuvent_etre_modifiees() {
        let mut session = CalibrationSession::new();

        session.set_reference_distances(200.0, 150.0).unwrap();

        assert_eq!(session.horizontal_distance_mm, 200.0);
        assert_eq!(session.vertical_distance_mm, 150.0);
    }
    #[test]
    fn distance_horizontale_invalide() {
        let mut session = CalibrationSession::new();

        let result = session.set_reference_distances(0.0, 100.0);

        assert_eq!(result, Err(CalibrationError::InvalidRealDistance));
    }

    #[test]
    fn distance_verticale_invalide() {
        let mut session = CalibrationSession::new();

        let result = session.set_reference_distances(100.0, 0.0);

        assert_eq!(result, Err(CalibrationError::InvalidRealDistance));
    }
    #[test]
    fn calibration_utilise_les_distances_de_reference() {
        let mut session = CalibrationSession::new();

        session.set_reference_distances(200.0, 100.0).unwrap();

        session.start();

        session.click(point(360.0, 240.0));
        session.click(point(180.0, 240.0));
        session.click(point(540.0, 240.0));
        session.click(point(360.0, 120.0));
        session.click(point(360.0, 360.0));

        let calibration = session.calibration().unwrap();

        // 360 pixels correspondent maintenant à 200 mm.
        assert!((calibration.pixels_per_mm_x() - 1.8).abs() < 0.001);

        // 240 pixels correspondent à 100 mm.
        assert!((calibration.pixels_per_mm_y() - 2.4).abs() < 0.001);
    }
    #[test]
    fn restart_efface_ancienne_calibration() {
        let mut session = CalibrationSession::new();

        session.start();

        session.click(point(360.0, 240.0));
        session.click(point(180.0, 240.0));
        session.click(point(540.0, 240.0));
        session.click(point(360.0, 120.0));
        session.click(point(360.0, 360.0));

        assert!(session.calibration().is_some());

        session.start();

        assert_eq!(session.calibration(), None);
        assert_eq!(session.step(), CalibrationStep::Center);
        assert_eq!(session.center(), None);
    }
    #[test]
    fn calibration_est_inactive_au_depart() {
        let session = CalibrationSession::new();

        assert!(!session.is_calibrated());
    }

    #[test]
    fn pixel_to_mm_utilise_la_calibration() {
        let mut session = CalibrationSession::new();

        session.start();

        session.click(point(360.0, 240.0));
        session.click(point(180.0, 240.0));
        session.click(point(540.0, 240.0));
        session.click(point(360.0, 120.0));
        session.click(point(360.0, 360.0));

        assert!(session.is_calibrated());

        let result = session.pixel_to_mm(point(396.0, 288.0)).unwrap();

        assert!((result.x - 10.0).abs() < 0.001);
        assert!((result.y - 20.0).abs() < 0.001);
    }
}
