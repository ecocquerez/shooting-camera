use crate::model::Point;

/// Calcule le centre géométrique d'une liste de points.
///
/// Le résultat est exprimé dans le même repère que les points
/// d'entrée.
pub fn calculate_center(points: &[Point]) -> Option<Point> {
    if points.is_empty() {
        return None;
    }

    let (x, y) = points.iter().fold((0.0_f32, 0.0_f32), |(sx, sy), point| {
        (sx + point.x, sy + point.y)
    });

    let n = points.len() as f32;

    Some(Point { x: x / n, y: y / n })
}

/// Calcule la distance maximale entre une série de points
/// et leur centre.
///
/// Le résultat est exprimé dans l'unité des points.
#[cfg_attr(not(test), allow(dead_code))]
pub fn calculate_max_distance(points: &[Point], center: Point) -> Option<f32> {
    if points.is_empty() {
        return None;
    }

    let max_distance = points
        .iter()
        .map(|point| point.distance_to(center))
        .fold(0.0_f32, f32::max);

    Some(max_distance)
}

/// Calcule la distance moyenne entre une série de points
/// et leur centre.
#[cfg_attr(not(test), allow(dead_code))]
pub fn calculate_average_distance(points: &[Point], center: Point) -> Option<f32> {
    if points.is_empty() {
        return None;
    }

    let sum: f32 = points.iter().map(|point| point.distance_to(center)).sum();

    Some(sum / points.len() as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centre_de_points() {
        let points = [
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 0.0, y: 10.0 },
            Point { x: 10.0, y: 10.0 },
        ];

        let center = calculate_center(&points).unwrap();

        assert!((center.x - 5.0).abs() < f32::EPSILON);
        assert!((center.y - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn centre_liste_vide() {
        let points: [Point; 0] = [];

        assert_eq!(calculate_center(&points), None);
    }

    #[test]
    fn distance_maximale() {
        let center = Point { x: 0.0, y: 0.0 };

        let points = [Point { x: 3.0, y: 4.0 }, Point { x: 6.0, y: 8.0 }];

        let distance = calculate_max_distance(&points, center).unwrap();

        assert!((distance - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn distance_moyenne() {
        let center = Point { x: 0.0, y: 0.0 };

        let points = [Point { x: 3.0, y: 4.0 }, Point { x: 6.0, y: 8.0 }];

        let distance = calculate_average_distance(&points, center).unwrap();

        assert!((distance - 7.5).abs() < f32::EPSILON);
    }
}
