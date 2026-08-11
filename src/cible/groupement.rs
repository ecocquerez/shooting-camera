use crate::cible::geometry::calculate_center;
use crate::model::{Impact, Point};

/// Représente le groupement d'une série d'impacts.
#[derive(Debug, Clone)]
pub struct Groupement {
    impacts: Vec<Impact>,
    center: Point,
}

impl Groupement {
    /// Crée un groupement à partir d'une série d'impacts.
    ///
    /// Retourne `None` si aucun impact n'est fourni.
    pub fn new(impacts: &[Impact]) -> Option<Self> {
        let points: Vec<Point> = impacts
            .iter()
            .filter_map(|impact| impact.position_cible)
            .collect();

        let center = calculate_center(&points)?;

        Some(Self {
            impacts: impacts.to_vec(),
            center,
        })
    }

    /// Retourne les impacts du groupement.
    pub fn impacts(&self) -> &[Impact] {
        &self.impacts
    }

    /// Retourne le centre géométrique du groupement.
    pub fn center(&self) -> Point {
        self.center
    }

    /// Nombre d'impacts.
    pub fn count(&self) -> usize {
        self.impacts
            .iter()
            .filter(|impact| impact.is_calibrated())
            .count()
    }

    /// Distance entre un impact et le centre du groupement.
    pub fn distance_from_center(&self, impact: &Impact) -> Option<f32> {
        let position = impact.position_cible?;

        Some(position.distance_to(self.center))
    }

    /// Distance maximale entre un impact et le centre
    /// du groupement.
    ///
    /// C'est le rayon maximal du groupement.
    pub fn max_distance(&self) -> f32 {
        self.impacts
            .iter()
            .filter_map(|impact| self.distance_from_center(impact))
            .fold(0.0_f32, f32::max)
    }

    /// Diamètre du groupement.
    pub fn diameter(&self) -> f32 {
        self.max_distance() * 2.0
    }

    /// Distance moyenne des impacts au centre du groupement.
    pub fn average_distance(&self) -> f32 {
        let distances: Vec<f32> = self
            .impacts
            .iter()
            .filter_map(|impact| self.distance_from_center(impact))
            .collect();

        if distances.is_empty() {
            return 0.0;
        }

        distances.iter().sum::<f32>() / distances.len() as f32
    }

    /// Écart du centre du groupement par rapport au point visé.
    ///
    /// Dans notre modèle, le point visé est l'origine `(0, 0)`.
    pub fn offset_from_aim(&self) -> Point {
        self.center
    }

    /// Distance du centre du groupement au point visé.
    pub fn distance_from_aim(&self) -> f32 {
        self.center.distance_to(Point::new(0.0, 0.0))
    }
}

/// Crée un groupement à partir d'impacts.
pub fn calculate_groupement(impacts: &[Impact]) -> Option<Groupement> {
    Groupement::new(impacts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn impact(number: u32, x: f32, y: f32) -> Impact {
        Impact::with_target_position(number, Point::new(x, y), Point::new(x, y))
    }

    #[test]
    fn groupement_vide() {
        let impacts: Vec<Impact> = Vec::new();

        assert!(calculate_groupement(&impacts).is_none());
    }

    #[test]
    fn impact_non_calibre() {
        let impact = Impact::new(1, Point::new(350.0, 230.0));

        assert!(!impact.is_calibrated());
        assert_eq!(impact.target_position(), None);
    }
    #[test]
    fn groupement_ne_compte_pas_les_impacts_non_calibres() {
        let impacts = [
            impact(1, 10.0, 10.0),
            Impact::new(2, Point::new(20.0, 20.0)),
        ];

        let groupement = calculate_groupement(&impacts).unwrap();

        assert_eq!(groupement.count(), 1);
    }
    #[test]
    fn impact_calibre() {
        let mut impact = Impact::new(1, Point::new(350.0, 230.0));

        impact.set_target_position(Point::new(-10.0, -5.0));

        assert!(impact.is_calibrated());

        assert_eq!(impact.target_position(), Some(Point::new(-10.0, -5.0)));
    }

    #[test]
    fn groupement_un_impact() {
        let impacts = [impact(1, 10.0, 20.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        assert_eq!(groupement.count(), 1);
        assert_eq!(groupement.center(), Point::new(10.0, 20.0));
    }

    #[test]
    fn centre_du_groupement() {
        let impacts = [
            impact(1, 0.0, 0.0),
            impact(2, 10.0, 0.0),
            impact(3, 0.0, 10.0),
            impact(4, 10.0, 10.0),
        ];

        let groupement = calculate_groupement(&impacts).unwrap();

        assert!((groupement.center().x - 5.0).abs() < 0.001);

        assert!((groupement.center().y - 5.0).abs() < 0.001);
    }

    #[test]
    fn distance_maximale() {
        let impacts = [impact(1, -3.0, -4.0), impact(2, 3.0, 4.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        assert!((groupement.max_distance() - 5.0).abs() < 0.001);
    }

    #[test]
    fn diametre() {
        let impacts = [impact(1, -3.0, 0.0), impact(2, 3.0, 0.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        assert!((groupement.diameter() - 6.0).abs() < 0.001);
    }

    #[test]
    fn distance_moyenne() {
        let impacts = [impact(1, -3.0, 0.0), impact(2, 3.0, 0.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        assert!((groupement.average_distance() - 3.0).abs() < 0.001);
    }

    #[test]
    fn ecart_au_point_vise() {
        let impacts = [impact(1, 10.0, 0.0), impact(2, 20.0, 0.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        let offset = groupement.offset_from_aim();

        assert!((offset.x - 15.0).abs() < 0.001);

        assert!(offset.y.abs() < 0.001);

        assert!((groupement.distance_from_aim() - 15.0).abs() < 0.001);
    }
}
