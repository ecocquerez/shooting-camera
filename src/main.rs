mod camera;
mod cible;
mod model;

slint::include_modules!();

use crate::cible::calibration_session::{CalibrationSession, CalibrationStep};
use crate::cible::geometry::calculate_center;
use crate::cible::groupement::calculate_groupement;
use crate::model::{Impact, Point};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};

use slint::{Image, Rgb8Pixel, SharedPixelBuffer, Timer, VecModel};

fn frame_to_slint_image(frame: camera::CameraFrame) -> Image {
    let buffer =
        SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(&frame.data, frame.width, frame.height);

    Image::from_rgb8(buffer)
}

fn load_camera_devices(
    window: &MainWindow,
) -> Result<Vec<camera::CameraDevice>, nokhwa::NokhwaError> {
    let devices = camera::enumerate()?;

    let model: Vec<CameraDeviceViewData> = devices
        .iter()
        .enumerate()
        .map(|(index, device)| CameraDeviceViewData {
            index: index as i32,
            name: device.name.clone().into(),
        })
        .collect();

    window.set_camera_devices(Rc::new(VecModel::from(model)).into());
    window.set_selected_camera(if devices.is_empty() { -1 } else { 0 });

    Ok(devices)
}

fn select_camera_format(
    camera_index: CameraIndex,
) -> Result<nokhwa::utils::CameraFormat, Box<dyn std::error::Error>> {
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
    let mut camera = Camera::new(camera_index, requested)?;
    let formats = camera.compatible_camera_formats()?;
    let formats = camera::unique_formats(formats);

    camera::select_best_format(&formats).ok_or_else(|| "Aucun format vidéo compatible".into())
}

fn connect_selected_camera(
    window: &MainWindow,
    devices: &[camera::CameraDevice],
    selected_device: usize,
    capture_slot: &Rc<RefCell<Option<camera::CameraCapture>>>,
) {
    if let Some(previous_capture) = capture_slot.borrow_mut().take() {
        previous_capture.stop();
    }

    let Some(device) = devices.get(selected_device) else {
        window.set_camera_status("Caméra sélectionnée invalide".into());
        return;
    };

    window.set_camera_status(format!("Initialisation de {}...", device.name).into());
    window.set_selected_camera(selected_device as i32);

    match select_camera_format(device.index.clone()) {
        Ok(format) => {
            println!("Format sélectionné : {}", format);
            let capture = camera::CameraCapture::new(device.index.clone(), format);
            capture.start();
            *capture_slot.borrow_mut() = Some(capture);
            window.set_camera_status(format!("Caméra connectée : {}", device.name).into());
        }
        Err(error) => {
            window.set_camera_status(format!("Erreur caméra : {}", error).into());
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = MainWindow::new()?;
    ui.set_camera_status("Recherche caméra...".into());

    let devices = Rc::new(RefCell::new(load_camera_devices(&ui)?));
    let capture = Rc::new(RefCell::new(None::<camera::CameraCapture>));

    if devices.borrow().is_empty() {
        ui.set_camera_status("Aucune caméra trouvée".into());
        ui.run()?;
        return Ok(());
    }

    connect_selected_camera(&ui, &devices.borrow(), 0, &capture);

    let calibration = Rc::new(RefCell::new(CalibrationSession::new()));
    let impacts = Rc::new(RefCell::new(Vec::<Impact>::new()));
    let selected_impact = Rc::new(RefCell::new(None::<usize>));
    let moving_impact = Rc::new(RefCell::new(false));

    let devices_for_refresh = devices.clone();
    let capture_for_refresh = capture.clone();
    let ui_weak = ui.as_weak();
    ui.on_connect_camera(move || {
        if let Some(ui) = ui_weak.upgrade() {
            match load_camera_devices(&ui) {
                Ok(new_devices) => {
                    *devices_for_refresh.borrow_mut() = new_devices;

                    if devices_for_refresh.borrow().is_empty() {
                        capture_for_refresh.borrow_mut().take();
                        ui.set_camera_status("Aucune caméra trouvée".into());
                    }
                }
                Err(error) => {
                    ui.set_camera_status(format!("Erreur caméra : {}", error).into());
                }
            }
        }
    });

    let devices_for_selection = devices.clone();
    let capture_for_selection = capture.clone();
    let ui_weak = ui.as_weak();
    ui.on_camera_selected(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            connect_selected_camera(
                &ui,
                &devices_for_selection.borrow(),
                index as usize,
                &capture_for_selection,
            );
            ui.set_camera_section_open(false);
        }
    });

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

    ui.on_calibration_distances_validated({
        let calibration = calibration.clone();
        let ui_weak = ui.as_weak();

        move || {
            let mut session = calibration.borrow_mut();

            if let Some(ui) = ui_weak.upgrade() {
                let horizontal = ui.get_calibration_horizontal_distance();
                let vertical = ui.get_calibration_vertical_distance();

                if let Err(error) = session.set_reference_distances(horizontal, vertical) {
                    println!("Distances de calibration invalides : {:?}", error);
                }

                if session.is_complete() {
                    if let Err(error) = session.build_calibration(horizontal, vertical) {
                        println!("Impossible de reconstruire la calibration : {:?}", error);
                    }
                }

                update_calibration_ui(&ui, &session);
            }
        }
    });

    ui.on_cancel_calibration({
        let calibration = calibration.clone();
        let ui_weak = ui.as_weak();

        move || {
            let mut session = calibration.borrow_mut();
            session.cancel();

            if let Some(ui) = ui_weak.upgrade() {
                update_calibration_ui(&ui, &session);
            }
        }
    });

    ui.on_capture_target({
        let calibration = calibration.clone();
        let ui_weak = ui.as_weak();

        move || {
            let session = calibration.borrow();

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_calibration_visible(false);
                ui.set_calibration_complete(false);
                ui.set_calibration_step("Aucune".into());
                ui.set_calibration_active_point("".into());
                ui.set_calibration_instruction("".into());

                ui.set_horizontal_a_visible(false);
                ui.set_horizontal_b_visible(false);
                ui.set_vertical_a_visible(false);
                ui.set_vertical_b_visible(false);

                if session.calibration().is_some() {
                    ui.set_shooting_configuration_open(false);
                }
            }
        }
    });

    ui.on_impact_selected({
        let selected_impact = selected_impact.clone();
        let ui_weak = ui.as_weak();

        move |index| {
            let selected_index = if index > 0 {
                (index - 1) as usize
            } else {
                index as usize
            };

            *selected_impact.borrow_mut() = Some(selected_index);

            println!("Impact sélectionné : #{}", selected_index + 1);

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_selected_impact((selected_index + 1) as i32);
            }
        }
    });

    ui.on_clear_impacts({
        let impacts = impacts.clone();
        let selected_impact = selected_impact.clone();
        let moving_impact = moving_impact.clone();
        let ui_weak = ui.as_weak();

        move || {
            impacts.borrow_mut().clear();
            *selected_impact.borrow_mut() = None;
            *moving_impact.borrow_mut() = false;

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_selected_impact(-1);
                ui.set_moving_impact(false);
                update_impacts_ui(&ui, &[]);
                clear_groupement_ui(&ui);
            }
        }
    });

    ui.on_move_selected_impact({
        let moving_impact = moving_impact.clone();
        let selected_impact = selected_impact.clone();
        let ui_weak = ui.as_weak();

        move || {
            if selected_impact.borrow().is_none() {
                return;
            }

            let new_state = !*moving_impact.borrow();
            *moving_impact.borrow_mut() = new_state;

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_moving_impact(new_state);
            }
        }
    });

    ui.on_delete_selected_impact({
        let impacts = impacts.clone();
        let selected_impact = selected_impact.clone();
        let moving_impact = moving_impact.clone();
        let calibration = calibration.clone();
        let ui_weak = ui.as_weak();

        move || {
            let Some(selected_index) = *selected_impact.borrow() else {
                return;
            };

            let mut impacts = impacts.borrow_mut();

            if selected_index >= impacts.len() {
                return;
            }

            impacts.remove(selected_index);

            for (index, impact) in impacts.iter_mut().enumerate() {
                impact.number = index as u32 + 1;
            }

            let new_selection = if impacts.is_empty() {
                None
            } else if selected_index >= impacts.len() {
                Some(impacts.len() - 1)
            } else {
                Some(selected_index)
            };

            *selected_impact.borrow_mut() = new_selection;
            *moving_impact.borrow_mut() = false;

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_moving_impact(false);
                ui.set_selected_impact(new_selection.map(|index| (index + 1) as i32).unwrap_or(-1));
                update_impacts_ui(&ui, &impacts);

                let session = calibration.borrow();
                if let Some(calibration) = session.calibration() {
                    let target_distance = ui.get_target_distance();
                    update_groupement_ui(&ui, &impacts, target_distance, calibration);
                } else {
                    clear_groupement_ui(&ui);
                }
            }
        }
    });

    ui.on_target_distance_validated({
        let impacts = impacts.clone();
        let calibration = calibration.clone();
        let ui_weak = ui.as_weak();

        move || {
            if let Some(ui) = ui_weak.upgrade() {
                let distance = ui.get_target_distance();
                let impacts = impacts.borrow();
                let session = calibration.borrow();

                ui.set_shooting_configuration_open(false);

                let Some(calibration) = session.calibration() else {
                    println!("Impossible de recalculer le groupement : cible non calibrée");
                    return;
                };

                update_groupement_ui(&ui, &impacts, distance, calibration);
            }
        }
    });

    ui.on_target_clicked({
        let calibration = calibration.clone();
        let impacts = impacts.clone();
        let selected_impact = selected_impact.clone();
        let moving_impact = moving_impact.clone();
        let ui_weak = ui.as_weak();

        move |x, y| {
            let point = Point::new(x, y);
            let mut session = calibration.borrow_mut();

            if session.step() != CalibrationStep::None
                && session.step() != CalibrationStep::Complete
            {
                let result = session.click(point);
                println!("Calibration : clic ({:.1}, {:.1}) -> {:?}", x, y, result);

                if let Some(ui) = ui_weak.upgrade() {
                    update_calibration_ui(&ui, &session);

                    if session.is_complete() {
                        ui.set_shooting_configuration_open(false);
                    }
                }

                return;
            }

            let calibration = match session.calibration() {
                Some(calibration) => calibration,
                None => {
                    println!("Impact ignoré : cible non calibrée");
                    return;
                }
            };

            let target_position = calibration.pixel_to_mm(point);
            let mut impacts = impacts.borrow_mut();

            if *moving_impact.borrow() {
                let Some(selected_index) = *selected_impact.borrow() else {
                    return;
                };

                if let Some(impact) = impacts.get_mut(selected_index) {
                    impact.position_image = point;
                    impact.set_target_position(target_position);

                    println!(
                        "Impact #{} déplacé : image=({:.1}, {:.1}) -> cible=({:.2}, {:.2}) mm",
                        impact.number, x, y, target_position.x, target_position.y
                    );
                }

                *moving_impact.borrow_mut() = false;

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_moving_impact(false);
                    update_impacts_ui(&ui, &impacts);
                    let target_distance = ui.get_target_distance();
                    update_groupement_ui(&ui, &impacts, target_distance, calibration);
                }

                return;
            }

            let number = impacts.len() as u32 + 1;
            let impact = Impact::with_target_position(number, point, target_position);

            println!(
                "Impact #{} : image=({:.1}, {:.1}) -> cible=({:.2}, {:.2}) mm",
                number, x, y, target_position.x, target_position.y
            );

            impacts.push(impact);
            let selected_index = impacts.len() - 1;
            *selected_impact.borrow_mut() = Some(selected_index);

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_selected_impact((selected_index + 1) as i32);
                update_impacts_ui(&ui, &impacts);
                let target_distance = ui.get_target_distance();
                update_groupement_ui(&ui, &impacts, target_distance, calibration);
            }
        }
    });

    let timer = Timer::default();
    let ui_weak = ui.as_weak();
    let capture_for_timer = capture.clone();

    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(33),
        move || {
            let frame = {
                let capture = capture_for_timer.borrow();
                capture
                    .as_ref()
                    .and_then(|capture| capture.try_receive_frame())
            };

            if let Some(frame) = frame {
                let image = frame_to_slint_image(frame);

                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_camera_image(image);
                }
            }
        },
    );

    ui.run()?;
    Ok(())
}

fn clear_groupement_ui(window: &MainWindow) {
    window.set_impact_count(0);
    window.set_average_impact_visible(false);
    window.set_grouping_diameter("—".into());
    window.set_grouping_center("—".into());
    window.set_grouping_offset("—".into());
    window.set_average_impact_offset("—".into());
}

fn update_groupement_ui(
    window: &MainWindow,
    impacts: &[Impact],
    target_distance: f32,
    calibration: &crate::cible::calibration::Calibration,
) {
    window.set_impact_count(impacts.len() as i32);

    let Some(groupement) = calculate_groupement(impacts) else {
        return;
    };

    let calibrated_points: Vec<Point> = impacts
        .iter()
        .filter_map(|impact| impact.position_cible)
        .collect();

    let Some(average_impact) = calculate_center(&calibrated_points) else {
        return;
    };

    let average_impact_image = calibration.mm_to_pixel(average_impact);
    window.set_average_impact_image_x(average_impact_image.x);
    window.set_average_impact_image_y(average_impact_image.y);
    window.set_average_impact_visible(calibrated_points.len() >= 2);

    let average_distance_mrad = average_impact.distance_to(Point::new(0.0, 0.0)) / target_distance;
    let average_offset_x_mrad = average_impact.x / target_distance;
    let average_offset_y_mrad = average_impact.y / target_distance;
    let average_distance_moa = average_distance_mrad * 3.4377468;
    let average_offset_x_moa = average_offset_x_mrad * 3.4377468;
    let average_offset_y_moa = average_offset_y_mrad * 3.4377468;

    let center = groupement.center();
    let impact_count = groupement.count();

    if impact_count < 2 {
        window.set_grouping_diameter("—".into());
        window.set_grouping_center("—".into());
        window.set_grouping_offset("—".into());
        window.set_average_impact_offset("—".into());
        return;
    }

    let Some(diameter_mrad) = groupement.diameter_mrad(target_distance) else {
        window.set_grouping_diameter("—".into());
        window.set_grouping_center("—".into());
        window.set_grouping_offset("—".into());
        return;
    };

    let Some(distance_mrad) = groupement.distance_from_aim_mrad(target_distance) else {
        return;
    };

    let Some(offset_mrad) = groupement.offset_mrad(target_distance) else {
        return;
    };

    let Some(diameter_moa) = groupement.diameter_moa(target_distance) else {
        return;
    };

    let Some(distance_moa) = groupement.distance_from_aim_moa(target_distance) else {
        return;
    };

    let Some(offset_moa) = groupement.offset_moa(target_distance) else {
        return;
    };

    println!("Groupement : {} impacts", impact_count);
    println!("Centre : ({:.2}, {:.2}) mm", center.x, center.y);
    println!(
        "Diamètre : {:.2} mrad / {:.2} MOA",
        diameter_mrad, diameter_moa
    );
    println!(
        "Écart au point visé : {:.2} mrad / {:.2} MOA",
        distance_mrad, distance_moa
    );
    println!(
        "Décalage X/Y : ({:.2}, {:.2}) mrad / ({:.2}, {:.2}) MOA",
        offset_mrad.x, offset_mrad.y, offset_moa.x, offset_moa.y
    );

    window.set_grouping_diameter(
        format!("{:.2} mrad / {:.2} MOA", diameter_mrad, diameter_moa).into(),
    );
    window
        .set_grouping_center(format!("{:.2} mrad / {:.2} MOA", distance_mrad, distance_moa).into());
    window.set_grouping_offset(
        format!(
            "X {:.2} / Y {:.2} mrad · X {:.2} / Y {:.2} MOA",
            offset_mrad.x, offset_mrad.y, offset_moa.x, offset_moa.y
        )
        .into(),
    );
    window.set_average_impact_offset(
        format!(
            "{:.2} mrad / {:.2} MOA · X {:.2} / Y {:.2} mrad · X {:.2} / Y {:.2} MOA",
            average_distance_mrad,
            average_distance_moa,
            average_offset_x_mrad,
            average_offset_y_mrad,
            average_offset_x_moa,
            average_offset_y_moa
        )
        .into(),
    );
}

fn update_calibration_ui(window: &MainWindow, session: &CalibrationSession) {
    let step = session.step();
    let calibration_complete = session.is_complete();

    window.set_calibration_visible(step != CalibrationStep::None);
    window.set_calibration_complete(calibration_complete);
    window.set_calibration_step(calibration_step_label(step).into());
    window.set_calibration_active_point(calibration_active_point(step).into());
    window.set_calibration_instruction(calibration_instruction(step).into());

    if let Some(point) = session.center() {
        window.set_center_x(point.x);
        window.set_center_y(point.y);
    }

    if let Some(point) = session.horizontal_a() {
        window.set_horizontal_a_x(point.x);
        window.set_horizontal_a_y(point.y);
    } else {
        window.set_horizontal_a_x(-1.0);
        window.set_horizontal_a_y(-1.0);
    }
    window.set_horizontal_a_visible(session.horizontal_a().is_some());

    if let Some(point) = session.horizontal_b() {
        window.set_horizontal_b_x(point.x);
        window.set_horizontal_b_y(point.y);
    } else {
        window.set_horizontal_b_x(-1.0);
        window.set_horizontal_b_y(-1.0);
    }
    window.set_horizontal_b_visible(session.horizontal_b().is_some());

    if let Some(point) = session.vertical_a() {
        window.set_vertical_a_x(point.x);
        window.set_vertical_a_y(point.y);
    } else {
        window.set_vertical_a_x(-1.0);
        window.set_vertical_a_y(-1.0);
    }
    window.set_vertical_a_visible(session.vertical_a().is_some());

    if let Some(point) = session.vertical_b() {
        window.set_vertical_b_x(point.x);
        window.set_vertical_b_y(point.y);
    } else {
        window.set_vertical_b_x(-1.0);
        window.set_vertical_b_y(-1.0);
    }
    window.set_vertical_b_visible(session.vertical_b().is_some());
}

fn calibration_step_label(step: CalibrationStep) -> &'static str {
    match step {
        CalibrationStep::None => "Aucune",
        CalibrationStep::Center => "Centre",
        CalibrationStep::HorizontalFirst => "Repère horizontal A",
        CalibrationStep::HorizontalSecond => "Repère horizontal B",
        CalibrationStep::VerticalFirst => "Repère vertical A",
        CalibrationStep::VerticalSecond => "Repère vertical B",
        CalibrationStep::Complete => "Terminée",
    }
}

fn calibration_active_point(step: CalibrationStep) -> &'static str {
    match step {
        CalibrationStep::None => "",
        CalibrationStep::Center => "C",
        CalibrationStep::HorizontalFirst => "H1",
        CalibrationStep::HorizontalSecond => "H2",
        CalibrationStep::VerticalFirst => "V1",
        CalibrationStep::VerticalSecond => "V2",
        CalibrationStep::Complete => "",
    }
}

fn calibration_instruction(step: CalibrationStep) -> &'static str {
    match step {
        CalibrationStep::None => "Cliquez sur \"Calibrer la cible\" pour démarrer.",
        CalibrationStep::Center => "Cliquez sur le centre de la cible.",
        CalibrationStep::HorizontalFirst => {
            "Cliquez sur le premier point de la référence horizontale."
        }
        CalibrationStep::HorizontalSecond => {
            "Cliquez sur le second point de la référence horizontale."
        }
        CalibrationStep::VerticalFirst => "Cliquez sur le premier point de la référence verticale.",
        CalibrationStep::VerticalSecond => "Cliquez sur le second point de la référence verticale.",
        CalibrationStep::Complete => {
            "Calibration terminée. Vous pouvez maintenant marquer les impacts."
        }
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
