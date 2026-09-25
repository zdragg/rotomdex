use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay, SimulatorEvent, Window};
use mousefood::{
    EmbeddedBackend, EmbeddedBackendConfig,
    embedded_graphics::{geometry, pixelcolor::Bgr565},
};
use ratatui::{
    Frame, Terminal,
    widgets::{Block, Paragraph},
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    if let Err(err) = run().await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

const FRAMES_PER_SECOND: f64 = 33.3;
async fn run() -> Result<()> {
    let mut simulator_window = Window::new(
        "mousefood simulator",
        &OutputSettings {
            scale: 2,
            ..Default::default()
        },
    );
    simulator_window.set_max_fps(libm::round(FRAMES_PER_SECOND) as u32);

    let mut display = SimulatorDisplay::<Bgr565>::new(geometry::Size::new(480, 240));

    let config = EmbeddedBackendConfig {
        // Define how to display newly rendered widgets to the simulator window
        flush_callback: Box::new(move |display| {
            simulator_window.update(display);
            if simulator_window.events().any(|e| e == SimulatorEvent::Quit) {
                panic!("simulator window closed");
            }
        }),
        ..Default::default()
    };

    let backend = EmbeddedBackend::new(&mut display, config);
    let mut terminal = Terminal::new(backend)?;

    let ms_per_frame = libm::round(1000.0 / FRAMES_PER_SECOND) as u64;
    let mut ticker = Ticker::every(Duration::from_millis(ms_per_frame));
    loop {
        terminal.draw(draw)?;
        ticker.next().await;
    }
}

fn draw(frame: &mut Frame) {
    let block = Block::bordered().title("Mousefood");
    let paragraph = Paragraph::new("Hello from Mousefood!").block(block);
    frame.render_widget(paragraph, frame.area());
}
