use nokhwa::utils::{CameraFormat, FrameFormat};

/// Supprime les doublons présents dans les formats exposés
/// par certains périphériques / backends.
pub fn unique_formats(formats: Vec<CameraFormat>) -> Vec<CameraFormat> {
    let mut unique = Vec::new();

    for format in formats {
        if !unique.contains(&format) {
            unique.push(format);
        }
    }

    unique
}

/// Retourne les formats MJPEG disponibles.
///
/// MJPEG est privilégié pour notre application car il permet
/// de conserver une bonne résolution tout en limitant le débit USB.
pub fn mjpeg_formats(formats: &[CameraFormat]) -> Vec<CameraFormat> {
    formats
        .iter()
        .filter(|format| format.format() == FrameFormat::MJPEG)
        .copied()
        .collect()
}

/// Sélectionne le meilleur format pour notre application.
///
/// Priorités :
/// 1. MJPEG
/// 2. résolution maximale
/// 3. FPS maximal
pub fn select_best_format(formats: &[CameraFormat]) -> Option<CameraFormat> {
    if formats.is_empty() {
        return None;
    }

    let mjpeg = mjpeg_formats(formats);

    let candidates = if mjpeg.is_empty() {
        formats.to_vec()
    } else {
        mjpeg
    };

    candidates
        .into_iter()
        .max_by_key(|format| (resolution_pixels(format), format.frame_rate()))
}

/// Nombre de pixels d'une résolution.
fn resolution_pixels(format: &CameraFormat) -> u32 {
    format.resolution().width() * format.resolution().height()
}
