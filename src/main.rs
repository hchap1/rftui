mod application;
mod filesystem;

use std::env;
use arboard::Clipboard;
use std::process::Command;
use application::Application;
use ratatui::DefaultTerminal;

fn main() {
    let mut application: Application = Application::new(env::current_dir().unwrap());
    let mut terminal: DefaultTerminal = ratatui::init();
    application.run(&mut terminal);

    ratatui::restore();

    if application.clipboard == None { return; }

    match env::var("WAYLAND_DISPLAY") {
        Ok(_) =>  {
            let _ = Command::new("wl-copy").arg(application.clipboard.unwrap()).spawn();
        }
        Err(_) => {
            let mut clipboard: Clipboard = Clipboard::new().unwrap();
            let _ = clipboard.set_text(application.clipboard.unwrap());
        }
    }
}
