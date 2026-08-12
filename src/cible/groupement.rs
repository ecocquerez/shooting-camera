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

        let center = smallest_enclosing_circle_center(&points)?;

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

    /// Diamètre du groupement (extreme spread).
    ///
    /// Il s'agit de la distance maximale entre deux impacts.
    pub fn diameter(&self) -> f32 {
        let points: Vec<Point> = self
            .impacts
            .iter()
            .filter_map(|impact| impact.position_cible)
            .collect();

        let mut max_distance = 0.0_f32;

        for (index, point_a) in points.iter().enumerate() {
            for point_b in points.iter().skip(index + 1) {
                max_distance = max_distance.max(point_a.distance_to(*point_b));
            }
        }

        max_distance
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
    pub fn offset_mrad(&self, target_distance: f32) -> Option<Point> {
        if target_distance <= 0.0 {
            return None;
        }

        Some(Point::new(
            self.center.x / target_distance,
            self.center.y / target_distance,
        ))
    }

    pub fn distance_from_aim_mrad(&self, target_distance: f32) -> Option<f32> {
        if target_distance <= 0.0 {
            return None;
        }

        Some(self.distance_from_aim() / target_distance)
    }

    pub fn diameter_mrad(&self, target_distance: f32) -> Option<f32> {
        if target_distance <= 0.0 {
            return None;
        }

        Some(self.diameter() / target_distance)
    }

    pub fn offset_moa(&self, target_distance: f32) -> Option<Point> {
        self.offset_mrad(target_distance)
            .map(|offset| Point::new(mrad_to_moa(offset.x), mrad_to_moa(offset.y)))
    }

    pub fn distance_from_aim_moa(&self, target_distance: f32) -> Option<f32> {
        self.distance_from_aim_mrad(target_distance)
            .map(mrad_to_moa)
    }

    pub fn diameter_moa(&self, target_distance: f32) -> Option<f32> {
        self.diameter_mrad(target_distance).map(mrad_to_moa)
    }
}

fn mrad_to_moa(value_mrad: f32) -> f32 {
    value_mrad * 3.4377468
}

fn smallest_enclosing_circle_center(points: &[Point]) -> Option<Point> {
    match points {
        [] => None,
        [point] => Some(*point),
        _ => {
            let mut best_center = None;
            let mut best_radius = f32::INFINITY;

            for (index, point_a) in points.iter().enumerate() {
                for point_b in points.iter().skip(index + 1) {
                    let center =
                        Point::new((point_a.x + point_b.x) / 2.0, (point_a.y + point_b.y) / 2.0);
                    let radius = point_a.distance_to(*point_b) / 2.0;

                    if let Some(radius) = enclosing_radius_if_valid(points, center, radius) {
                        if radius < best_radius {
                            best_radius = radius;
                            best_center = Some(center);
                        }
                    }
                }
            }

            for (index_a, point_a) in points.iter().enumerate() {
                for (index_b, point_b) in points.iter().enumerate().skip(index_a + 1) {
                    for point_c in points.iter().skip(index_b + 1) {
                        if let Some(center) = circumcenter(*point_a, *point_b, *point_c) {
                            let radius = center.distance_to(*point_a);

                            if let Some(radius) = enclosing_radius_if_valid(points, center, radius)
                            {
                                if radius < best_radius {
                                    best_radius = radius;
                                    best_center = Some(center);
                                }
                            }
                        }
                    }
                }
            }

            best_center
        }
    }
}

fn enclosing_radius_if_valid(points: &[Point], center: Point, radius: f32) -> Option<f32> {
    if !radius.is_finite() {
        return None;
    }

    const EPSILON: f32 = 1e-3;

    for point in points {
        let distance = point.distance_to(center);

        if !distance.is_finite() || distance > radius + EPSILON {
            return None;
        }
    }

    Some(radius)
}

fn circumcenter(a: Point, b: Point, c: Point) -> Option<Point> {
    let d = 2.0 * (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y));

    let scale =
        a.x.abs()
            .max(a.y.abs())
            .max(b.x.abs())
            .max(b.y.abs())
            .max(c.x.abs())
            .max(c.y.abs())
            .max(1.0);

    if d.abs() <= 1e-6 * scale * scale {
        return None;
    }

    let a_sq = a.x * a.x + a.y * a.y;
    let b_sq = b.x * b.x + b.y * b.y;
    let c_sq = c.x * c.x + c.y * c.y;

    let ux = (a_sq * (b.y - c.y) + b_sq * (c.y - a.y) + c_sq * (a.y - b.y)) / d;
    let uy = (a_sq * (c.x - b.x) + b_sq * (a.x - c.x) + c_sq * (b.x - a.x)) / d;

    Some(Point::new(ux, uy))
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
    fn centre_du_groupement_est_le_centre_du_plus_petit_cercle() {
        let impacts = [
            impact(1, 0.0, 0.0),
            impact(2, 10.0, 0.0),
            impact(3, 4.0, 2.0),
        ];

        let groupement = calculate_groupement(&impacts).unwrap();

        assert!((groupement.center().x - 5.0).abs() < 0.001);
        assert!((groupement.center().y - 0.0).abs() < 0.001);
    }

    #[test]
    fn centre_du_groupement_reste_stable_pour_triangle_presque_colineaire() {
        let impacts = [
            impact(1, 20.512823, -12.853471),
            impact(2, 67.52136, -12.853471),
            impact(3, 39.10256, -27.763498),
        ];

        let groupement = calculate_groupement(&impacts).unwrap();

        assert!(groupement.center().x.is_finite());
        assert!(groupement.center().y.is_finite());
        assert!(groupement.center().x > 40.0);
        assert!(groupement.center().x < 50.0);
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
    #[test]
    fn offset_en_mrad() {
        let impacts = [impact(1, 10.0, -20.0), impact(2, 30.0, 20.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        // Centre : (20, 0) mm
        // À 100 m : 20 mm = 0.20 mrad
        let offset = groupement.offset_mrad(100.0).unwrap();

        assert!((offset.x - 0.20).abs() < 0.001);
        assert!(offset.y.abs() < 0.001);
    }

    #[test]
    fn distance_au_point_vise_en_mrad() {
        let impacts = [impact(1, 30.0, 40.0), impact(2, 10.0, 0.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        // Centre : (20, 20) mm
        // Distance = sqrt(20² + 20²) = 28.284 mm
        // À 100 m = 0.28284 mrad
        let distance = groupement.distance_from_aim_mrad(100.0).unwrap();

        assert!((distance - 0.2828427).abs() < 0.001);
    }

    #[test]
    fn diametre_en_mrad() {
        let impacts = [impact(1, -30.0, 0.0), impact(2, 30.0, 0.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        // Diamètre = 60 mm
        // À 100 m = 0.60 mrad
        let diameter = groupement.diameter_mrad(100.0).unwrap();

        assert!((diameter - 0.60).abs() < 0.001);
    }

    #[test]
    fn conversion_mrad_vers_moa() {
        let impacts = [impact(1, -30.0, 0.0), impact(2, 30.0, 0.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        let diameter_moa = groupement.diameter_moa(100.0).unwrap();
        let distance_moa = groupement.distance_from_aim_moa(100.0).unwrap();
        let offset_moa = groupement.offset_moa(100.0).unwrap();

        assert!((diameter_moa - 2.062648).abs() < 0.001);
        assert!(distance_moa.abs() < 0.001);
        assert!(offset_moa.x.abs() < 0.001);
        assert!(offset_moa.y.abs() < 0.001);
    }
    #[test]
    fn distance_de_tir_nulle_invalide() {
        let impacts = [impact(1, -30.0, 0.0), impact(2, 30.0, 0.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        assert!(groupement.offset_mrad(0.0).is_none());
        assert!(groupement.distance_from_aim_mrad(0.0).is_none());
        assert!(groupement.diameter_mrad(0.0).is_none());
    }

    #[test]
    fn distance_de_tir_negative_invalide() {
        let impacts = [impact(1, -30.0, 0.0), impact(2, 30.0, 0.0)];

        let groupement = calculate_groupement(&impacts).unwrap();

        assert!(groupement.offset_mrad(-10.0).is_none());
        assert!(groupement.distance_from_aim_mrad(-10.0).is_none());
        assert!(groupement.diameter_mrad(-10.0).is_none());
    }
}
