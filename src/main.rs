mod camera;
mod cible;
mod model;

slint::include_modules!();

use crate::cible::calibration_session::{CalibrationSession, CalibrationStep};
use crate::cible::groupement::calculate_groupement;
use crate::model::{Impact, Point};

use std::cell::RefCell;
use std::rc::Rc;

use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    query,
    utils::{ApiBackend, RequestedFormat, RequestedFormatType},
};

use std::time::Duration;

use slint::{Image, Rgb8Pixel, SharedPixelBuffer, Timer, VecModel};

fn frame_to_slint_image(frame: camera::CameraFrame) -> Image {
    let buffer =
        SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(&frame.data, frame.width, frame.height);

    Image::from_rgb8(buffer)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // -----------------------------------------------------
    // UI
    // -----------------------------------------------------

    let ui = MainWindow::new()?;

    ui.set_camera_status("Recherche caméra...".into());

    // -----------------------------------------------------
    // Recherche caméra
    // -----------------------------------------------------

    let cameras = query(ApiBackend::Auto)?;

    if cameras.is_empty() {
        ui.set_camera_status("Aucune caméra trouvée".into());

        ui.run()?;

        return Ok(());
    }

    let camera_info = cameras.first().ok_or("Aucune caméra disponible")?;

    ui.set_camera_status("Initialisation...".into());

    // -----------------------------------------------------
    // Recherche des formats
    // -----------------------------------------------------

    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);

    let mut camera = Camera::new(camera_info.index().clone(), requested)?;

    let formats = camera.compatible_camera_formats()?;

    let formats = camera::unique_formats(formats);

    let format = camera::select_best_format(&formats).ok_or("Aucun format vidéo compatible")?;

    println!("Format sélectionné : {}", format);

    // -----------------------------------------------------
    // Calibration
    // -----------------------------------------------------

    let calibration = Rc::new(RefCell::new(CalibrationSession::new()));
    let impacts = Rc::new(RefCell::new(Vec::<Impact>::new()));
    // -----------------------------------------------------
    // Démarrage capture
    // -----------------------------------------------------

    let capture = camera::CameraCapture::new(camera_info.index().clone(), format);

    capture.start();

    ui.set_camera_status("Caméra connectée".into());

    // -----------------------------------------------------
    // Démarrage calibration
    // -----------------------------------------------------

    let calibration_for_start = calibration.clone();
    let ui_weak = ui.as_weak();

    ui.on_request_calibration(move || {
        let mut session = calibration_for_start.borrow_mut();

        session.start();

        println!("Calibration démarrée : {:?}", session.step());

        if let Some(ui) = ui_weak.upgrade() {
            update_calibration_ui(&ui, &session);
        }
    });
    ui.on_impact_selected({
        let ui_weak = ui.as_weak();

        move |number| {
            println!("Impact sélectionné : #{}", number);

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_selected_impact(number);
            }
        }
    });
    // -----------------------------------------------------
    // Clic dans la cible
    // -----------------------------------------------------

    ui.on_target_clicked({
        let calibration = calibration.clone();
        let impacts = impacts.clone();
        let ui_weak = ui.as_weak();

        move |x, y| {
            let point = Point::new(x, y);

            let mut session = calibration.borrow_mut();

            // -------------------------------------------------
            // Calibration en cours
            // -------------------------------------------------

            if session.step() != CalibrationStep::None
                && session.step() != CalibrationStep::Complete
            {
                let result = session.click(point);

                println!("Calibration : clic ({:.1}, {:.1}) -> {:?}", x, y, result);

                if let Some(ui) = ui_weak.upgrade() {
                    update_calibration_ui(&ui, &session);
                }

                return;
            }

            // -------------------------------------------------
            // Tir normal
            // -------------------------------------------------

            let calibration = match session.calibration() {
                Some(calibration) => calibration,
                None => {
                    println!("Impact ignoré : cible non calibrée");
                    return;
                }
            };

            let target_position = calibration.pixel_to_mm(point);

            let mut impacts = impacts.borrow_mut();

            let number = impacts.len() as u32 + 1;

            let impact = Impact::with_target_position(number, point, target_position);

            println!(
                "Impact #{} : image=({:.1}, {:.1}) -> cible=({:.2}, {:.2}) mm",
                number, x, y, target_position.x, target_position.y
            );

            impacts.push(impact);
            if let Some(ui) = ui_weak.upgrade() {
                update_impacts_ui(&ui, &impacts);
            }
            if let Some(groupement) = calculate_groupement(&impacts) {
                let center = groupement.center();
                let offset = groupement.offset_from_aim();

                let diameter = groupement.diameter();
                let distance_from_aim = groupement.distance_from_aim();

                println!("Groupement : {} impacts", groupement.count());

                println!("Centre : ({:.2}, {:.2}) mm", center.x, center.y);

                println!("Diamètre : {:.2} mm", diameter);

                println!("Écart au point visé : {:.2} mm", distance_from_aim);

                println!("Décalage X/Y : ({:.2}, {:.2}) mm", offset.x, offset.y);

                // Mise à jour de l'interface
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_impact_count(impacts.len() as i32);

                    ui.set_grouping_diameter(format!("{:.2}", diameter).into());

                    ui.set_grouping_center(format!("{:.2}", distance_from_aim).into());

                    ui.set_grouping_offset(
                        format!("X {:.2} / Y {:.2} mm", offset.x, offset.y).into(),
                    );

                    ui.set_grouping_center_x(center.x);
                    ui.set_grouping_center_y(center.y);
                }
            }
        }
    });
    // -----------------------------------------------------
    // Timer de rafraîchissement UI
    // -----------------------------------------------------

    let timer = Timer::default();

    let ui_weak = ui.as_weak();

    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(33),
        move || {
            if let Some(frame) = capture.try_receive_frame() {
                let image = frame_to_slint_image(frame);

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_camera_image(image);
                }
            }
        },
    );

    // -----------------------------------------------------
    // Application
    // -----------------------------------------------------

    ui.run()?;

    Ok(())
}

fn update_calibration_ui(window: &MainWindow, session: &CalibrationSession) {
    window.set_calibration_visible(session.step() != CalibrationStep::None);

    if let Some(point) = session.center() {
        window.set_center_x(point.x);
        window.set_center_y(point.y);
    }

    if let Some(point) = session.horizontal_a() {
        window.set_horizontal_a_x(point.x);
        window.set_horizontal_a_y(point.y);
    }

    if let Some(point) = session.horizontal_b() {
        window.set_horizontal_b_x(point.x);
        window.set_horizontal_b_y(point.y);
    }

    if let Some(point) = session.vertical_a() {
        window.set_vertical_a_x(point.x);
        window.set_vertical_a_y(point.y);
    }

    if let Some(point) = session.vertical_b() {
        window.set_vertical_b_x(point.x);
        window.set_vertical_b_y(point.y);
    }
}

fn update_impacts_ui(window: &MainWindow, impacts: &[Impact]) {
    let model: Vec<ImpactViewData> = impacts
        .iter()
        .map(|impact| ImpactViewData {
            number: impact.number as i32,
            image_x: impact.position_image.x,
            image_y: impact.position_image.y,
            target_x: impact.position_cible.map(|p| p.x).unwrap_or(0.0),
            target_y: impact.position_cible.map(|p| p.y).unwrap_or(0.0),
        })
        .collect();

    window.set_impacts(Rc::new(VecModel::from(model)).into());
}
