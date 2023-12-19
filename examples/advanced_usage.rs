use std::error::Error;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use lcd1602_gpio::{LCDController, LcdLine};

enum LCDCommand {
    DisplayText(String, LcdLine),
    ClearScreen,
    Quit,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Create an LCD controller instance and wrap it in a mutex
    let shared_controller = Arc::new(Mutex::new(LCDController::default()?));

    // Create a channel for communication between main thread and LCD thread
    let (lcd_tx, lcd_rx) = mpsc::channel();

    // Clone the Arc to send to the LCD thread
    let shared_controller_clone = shared_controller.clone();

    // Spawn a new thread for handling the LCD
    let lcd_thread = thread::spawn(move || {
        for command in lcd_rx {
            match command {
                LCDCommand::DisplayText(text, line) => {
                    let mut controller = shared_controller_clone.lock().unwrap();
                    controller.display_text(&text, line);
                }
                LCDCommand::ClearScreen => {
                    let mut controller = shared_controller_clone.lock().unwrap();
                    controller.clear_screen();
                }
                LCDCommand::Quit => {
                    // Cleanup and exit the thread
                    break;
                }
            }
        }
    });

    // Continue with the main thread logic here
    // ...

    // Example: Send commands to the LCD thread
    lcd_tx.send(LCDCommand::DisplayText(
        "Hello World!".to_string(),
        LcdLine::Line1,
    ))?;
    thread::sleep(Duration::new(3, 0));
    lcd_tx.send(LCDCommand::DisplayText(
        "Hello Rustaceans".to_string(),
        LcdLine::Line2,
    ))?;
    thread::sleep(Duration::new(5, 0));

    // Send a command to clear the screen
    lcd_tx.send(LCDCommand::ClearScreen)?;

    // Signal the LCD thread to quit
    lcd_tx.send(LCDCommand::Quit)?;

    // Wait for the LCD thread to finish before exiting
    lcd_thread.join().unwrap();

    Ok(())
}
