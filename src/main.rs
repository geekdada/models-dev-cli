mod app;
mod data;
mod ui;

use std::time::Duration;

use crossterm::event;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Fetching data from models.dev...");
    let api_data = match data::fetch_data() {
        Ok(data) => {
            eprintln!(
                "Loaded {} providers.",
                data.len()
            );
            data
        }
        Err(e) => {
            eprintln!("Error fetching data: {}", e);
            std::process::exit(1);
        }
    };

    let mut terminal = ratatui::init();
    terminal.clear()?;

    let mut app = app::App::new(api_data);

    while !app.should_quit {
        terminal.draw(|frame| ui::render(&mut app, frame))?;

        if event::poll(Duration::from_millis(100))? {
            let evt = event::read()?;
            app.handle_event(&evt);
        }
    }

    ratatui::restore();
    Ok(())
}
