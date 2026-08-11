use nokhwa::{
    query,
    utils::{ApiBackend, CameraIndex},
};

#[derive(Debug, Clone)]
pub struct CameraDevice {
    pub index: CameraIndex,
    pub name: String,
    pub description: String,
    pub misc: String,
}

/// Énumère les caméras disponibles sur le système.
///
/// `ApiBackend::Auto` sélectionne automatiquement le backend
/// natif de la plateforme.
pub fn enumerate() -> Result<Vec<CameraDevice>, nokhwa::NokhwaError> {
    let cameras = query(ApiBackend::Auto)?;

    let devices = cameras
        .into_iter()
        .map(|camera| CameraDevice {
            index: camera.index().clone(),
            name: camera.human_name(),
            description: camera.description().to_string(),
            misc: camera.misc(),
        })
        .collect();

    Ok(devices)
}
