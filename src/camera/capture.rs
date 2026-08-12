use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::thread;

use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{CameraFormat, CameraIndex, RequestedFormat, RequestedFormatType},
};

#[derive(Debug)]
pub struct CameraFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum CameraCommand {
    Start,
    Stop,
}
struct FpsCounter {
    frames: u64,
}
impl FpsCounter {
    fn new() -> Self {
        Self { frames: 0 }
    }

    fn tick(&mut self) {
        self.frames += 1;
    }
}

pub struct CameraCapture {
    command_tx: Sender<CameraCommand>,
    frame_rx: Receiver<CameraFrame>,
}

impl CameraCapture {
    pub fn new(camera_index: CameraIndex, format: CameraFormat) -> Self {
        let (command_tx, command_rx) = mpsc::channel();

        let (frame_tx, frame_rx) = mpsc::sync_channel(2);

        thread::spawn(move || {
            camera_thread(camera_index, format, command_rx, frame_tx);
        });

        Self {
            command_tx,
            frame_rx,
        }
    }

    pub fn start(&self) {
        let _ = self.command_tx.send(CameraCommand::Start);
    }

    pub fn stop(&self) {
        let _ = self.command_tx.send(CameraCommand::Stop);
    }

    pub fn try_receive_frame(&self) -> Option<CameraFrame> {
        let mut latest = None;

        while let Ok(frame) = self.frame_rx.try_recv() {
            latest = Some(frame);
        }

        latest
    }
}

fn camera_thread(
    camera_index: CameraIndex,
    format: CameraFormat,
    command_rx: Receiver<CameraCommand>,
    frame_tx: SyncSender<CameraFrame>,
) {
    println!("Thread caméra démarré - {:?}", camera_index);

    loop {
        match command_rx.recv() {
            Ok(CameraCommand::Start) => {
                println!("Démarrage avec le format : {}", format);

                if let Err(error) =
                    capture_loop(camera_index.clone(), format, &command_rx, &frame_tx)
                {
                    eprintln!("Erreur caméra : {}", error);
                }
            }

            Ok(CameraCommand::Stop) => {
                println!("Arrêt caméra.");
                break;
            }

            Err(_) => {
                println!("Canal caméra fermé.");
                break;
            }
        }
    }
}

fn capture_loop(
    camera_index: CameraIndex,
    format: CameraFormat,
    command_rx: &Receiver<CameraCommand>,
    frame_tx: &SyncSender<CameraFrame>,
) -> Result<(), Box<dyn std::error::Error>> {
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(format));
    let mut fps = FpsCounter::new();
    let mut camera = Camera::new(camera_index, requested)?;

    println!("Caméra : {}", camera.info().human_name());

    println!("Format demandé : {}", format);

    println!("Format caméra : {}", camera.camera_format());

    camera.open_stream()?;
    let mut frame_count = 0u64;
    let mut last_report = std::time::Instant::now();
    println!("Flux caméra démarré.");

    loop {
        match command_rx.try_recv() {
            Ok(CameraCommand::Stop) => {
                camera.stop_stream()?;
                break;
            }

            Ok(CameraCommand::Start) => {
                // Déjà démarrée.
            }

            Err(mpsc::TryRecvError::Empty) => {}

            Err(mpsc::TryRecvError::Disconnected) => {
                camera.stop_stream()?;
                break;
            }
        }

        let frame = camera.frame()?;
        frame_count += 1;

        if last_report.elapsed() >= std::time::Duration::from_secs(1) {
            println!("[Camera thread] {} FPS", frame_count);

            frame_count = 0;
            last_report = std::time::Instant::now();
        }
        let image = frame.decode_image::<RgbFormat>()?;

        let camera_frame = CameraFrame {
            width: image.width(),
            height: image.height(),
            data: image.into_raw(),
        };
        fps.tick();
        let _ = frame_tx.try_send(camera_frame);
    }

    Ok(())
}
