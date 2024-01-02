use std::{
    env,
    path::PathBuf,
    sync::{
        mpsc::{self, SyncSender},
        Arc, RwLock,
    },
    thread::{self},
    time::{Duration, Instant},
};

mod cpu;

use cpu::{Cpu, Memory};
use eframe::{
    egui::{self, Key, Sense},
    epaint::{pos2, Pos2, Rect, Rounding, Stroke},
};
use egui::{Color32, Frame, Vec2};

#[derive(Debug)]
struct Game {
    screen: Arc<RwLock<Screen>>,
    sender: Option<SyncSender<Event>>,
}

impl Game {
    fn new() -> Self {
        let mut screen = Screen::new();
        screen.fill(Color32::LIGHT_BLUE);
        Self {
            screen: Arc::new(RwLock::new(screen)),
            sender: None,
        }
    }

    fn start(&mut self, path: PathBuf) {
        let (sender, rc) = mpsc::sync_channel(0);
        let screen = Arc::clone(&self.screen);
        thread::spawn(move || {
            let mut memory = Memory::new();
            memory.load_file(path.as_path()).unwrap();

            let mut cpu = Cpu::new(memory, screen);

            let tick = Duration::new(1, 0) / 60;
            loop {
                let instant = Instant::now();
                if let Ok(Event::KeyEvent(value)) = rc.try_recv() {
                    let value = value.to_be_bytes();
                    cpu.process_input(&value)
                }

                cpu.tick();
                let elapsed = instant.elapsed();
                let sleep = tick - elapsed;
                thread::sleep(sleep);
            }
        });
        self.sender = Some(sender);
    }

    fn send(&self, event: Event) -> anyhow::Result<()> {
        if let Some(sender) = &self.sender {
            sender.send(event)?;
        };
        Ok(())
    }
}

#[derive(Debug)]
struct Screen {
    pixels: Vec<Color32>,
}

impl Screen {
    fn new() -> Self {
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
    KeyEvent(u16),
}

#[derive(Debug)]
struct MainApp {
    game: Game,
}

impl MainApp {
    fn new(path: PathBuf) -> Self {
        let mut game = Game::new();
        game.start(path);
        Self { game }
    }
}

impl eframe::App for MainApp {
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
                            let color = screen.pixels.get(i).unwrap_or(&Color32::TEMPORARY_COLOR);
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
                ctx.input(|i| {
                    let value = i.events.iter().map(get_key).next().unwrap_or(0);
                    self.game.send(Event::KeyEvent(value)).unwrap_or_else(|_| {
                        println!("Could not send key event to game loop");
                    });
                })
            });
    }
}

fn get_key(event: &egui::Event) -> u16 {
    if let egui::Event::Key { key, .. } = event {
        return match key {
            Key::Num0 => 0x01,
            Key::Num1 => 0x02,
            Key::Num2 => 0x04,
            Key::Num3 => 0x08,
            Key::Num4 => 0x10,
            Key::Num5 => 0x20,
            Key::Num6 => 0x40,
            Key::Num7 => 0x80,
            Key::Num8 => 0x100,
            Key::Num9 => 0x200,
            Key::A => 0x400,
            Key::B => 0x800,
            Key::C => 0x1000,
            Key::D => 0x2000,
            Key::E => 0x4000,
            Key::F => 0x8000,
            _ => 0x00,
        };
    }
    0x00
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
        Box::new(|_cc| Box::new(MainApp::new(path))),
    )
}
