use std::{
    env,
    path::PathBuf,
    sync::{
        mpsc::{self, SyncSender},
        Arc, RwLock,
    },
    thread::{self},
};

mod cpu;

use cpu::{Cpu, Memory};
use eframe::{
    egui::{self, Sense},
    epaint::{pos2, Pos2, Rect, Rounding, Stroke},
};
use egui::{Color32, Frame, Vec2};

#[derive(Debug)]
struct Game {
    screen: Arc<RwLock<Screen>>,
    sender: Option<SyncSender<Event>>,
}

impl Game {
    fn new(x_size: usize, y_size: usize) -> Self {
        let mut screen = Screen::new(x_size, y_size);
        screen.fill(Color32::LIGHT_BLUE);
        Self {
            screen: Arc::new(RwLock::new(screen)),
            sender: None,
        }
    }

    fn start(&mut self, tx: &SyncSender<Event>, path: PathBuf) {
        let (sender, rc) = mpsc::sync_channel(0);
        let _tx = tx.clone();
        let screen = Arc::clone(&self.screen);
        let _ = thread::spawn(move || {
            let mut memory = Memory::new();

            memory.load_file(path.as_path()).unwrap();
            let mut cpu = Cpu::new(memory, screen);
            loop {
                if let Ok(event) = rc.try_recv() {
                    if event == Event::Quit {
                        break;
                    }
                }
                cpu.tick();
            }
        });
        self.sender = Some(sender);
    }
}

#[derive(Debug)]
struct Screen {
    pixels: Vec<Color32>,
}

impl Screen {
    fn new(x: usize, y: usize) -> Self {
        let _s_size = Vec2::new(x as f32, y as f32);
        let v = &[Color32::BLACK; 0xFFFF];
        Self { pixels: v.to_vec() }
    }

    fn fill(&mut self, color: Color32) {
        self.pixels = self.pixels.iter().map(|_| color).collect();
    }

    fn update(&mut self, data: &[u8]) {
        let mut s = Vec::new();
        for value in data {
            if *value < 216 {
                let red = (*value as u32 / 36 * 0x33) as u8;
                let green = (*value as u32 / 6 % 6 * 0x33) as u8;
                let blue = (*value as u32 % 6 * 0x33) as u8;
                let color = Color32::from_rgb(red, green, blue);
                s.push(color);
            } else {
                s.push(Color32::BLACK);
            }
        }
        self.pixels = s;
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Draw(u32, u32, Color32),
    Noop,
    Quit,
}

#[derive(Debug)]
struct MyApp {
    game: Game,
}

impl MyApp {
    fn new(x_size: usize, y_size: usize, path: PathBuf) -> Self {
        let (tx, _) = mpsc::sync_channel(0);
        let mut game = Game::new(x_size, y_size);
        game.start(&tx, path);
        Self { game }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    // Size of the canvas
                    let size = Vec2::splat(1024.0);

                    // Size of each "pixel"
                    let resolution = size / Vec2::splat(255.0);

                    // allocate a painter, and create a rectangle at 0.0 with resolution width
                    let (_, painter) = ui.allocate_painter(size, Sense::focusable_noninteractive());
                    let mut r = Rect::from_min_max(Pos2::ZERO, Pos2::ZERO + resolution);
                    let mut i = 0;
                    for _ in 0..256 {
                        for _ in 0..256 {
                            let screen = self.game.screen.read().unwrap();
                            let color = screen.pixels.get(i).unwrap();
                            painter.rect(r, Rounding::ZERO, *color, Stroke::NONE);

                            // move rectangles top left and bottom right point
                            r.min.x += resolution.x;
                            r.max.x += resolution.x;
                            i += 1;
                        }

                        // move the rectangle to first column and next row offset from previous row
                        r.min = pos2(0.0, r.min.y + resolution.y);

                        // move the rectangle bottom right corner to offset from top left
                        r.max = r.min + resolution;
                    }
                    ctx.request_repaint();
                });
            });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("RustByther")
            .with_inner_size([1024.0, 1024.0]),
        ..Default::default()
    };

    let args: Vec<_> = env::args().collect();
    let mut path = PathBuf::new();
    path.push(args[1].clone());

    eframe::run_native(
        "ByteRusther",
        options,
        Box::new(|_cc| Box::new(MyApp::new(128, 128, path))),
    )
}
