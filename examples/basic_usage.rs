// Basic usage of the package
use lcd1602_gpio::{LCDController, LcdLine};
use std::error::Error;
use std::thread;
use std::time::Duration;

fn sleep(secs: u64) {
    let delay = Duration::new(secs, 0);
    thread::sleep(delay);
}

fn main() -> Result<(), Box<dyn Error>> {
    // Create an LCD controller instance
    // You can use the default pins specified in the README or specify your own pin configuration with LCDController::new()
    let mut controller = LCDController::default().unwrap();

    // Print LCD controller information
    println!("{controller}");

    for _ in 0..5 {
        // Display text on the LCD
        controller.display_text("Hello World!", LcdLine::Line1);
        sleep(3);

        controller.display_text("Hello Rustaceans", LcdLine::Line2);
        sleep(5);

        controller.clear_screen();
        sleep(1);
    }

    Ok(())
}
