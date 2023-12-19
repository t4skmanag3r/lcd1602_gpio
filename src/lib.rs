use rppal::gpio::{Gpio, OutputPin};
use rppal::system::DeviceInfo;
use std::error::Error;
use std::fmt;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

// Pinout of the LCD:
// 1 : GND
// 2 : 5V power
// 3 : Display contrast - Connect to middle pin of the potentiometer
// 4 : RS (Register Select)
// 5 : R/W (Read Write) - Ground this pin (important)
// 6 : Enable or Strobe
// 7 : Data Bit 0 - data pin 0, 1, 2, 3 are not used
// 8 : Data Bit 1 -
// 9 : Data Bit 2 -
// 10: Data Bit 3 -
// 11: Data Bit 4
// 12: Data Bit 5
// 13: Data Bit 6
// 14: Data Bit 7
// 15: LCD Backlight +5V
// 16: LCD Backlight GND

const LCD_RS: u8 = 7;
const LCD_E: u8 = 8;
const LCD_D4: u8 = 25;
const LCD_D5: u8 = 24;
const LCD_D6: u8 = 23;
const LCD_D7: u8 = 18;

const LCD_CHARS: usize = 16; // Characters per line (16 max)

pub enum LcdLine {
    Line1 = 0x80, // LCD memory location for 1st line
    Line2 = 0xC0, // LCD memory location 2nd line
}

pub enum LcdMode {
    Character, // used for sending characters
    Command,   // used for sending commands
}

pub enum LcdCommand {
    Initialize = 0x33,              // Initializes the display
    ClearScreen = 0x01,             // Clears the display
    SetCursorMoveDirrection = 0x06, // Sets the cursor move direction
    Set4BitMode = 0x32,             // Sets the display to 4 bit mode
    SetCursorOff = 0x0C,            // Turns the cursor off
    Set2LineDisplay = 0x28,         // Sets the display to 2 lines
}

pub struct LCDController {
    device_info: DeviceInfo,
    lcd_rs: OutputPin,
    lcd_e: OutputPin,
    lcd_d4: OutputPin,
    lcd_d5: OutputPin,
    lcd_d6: OutputPin,
    lcd_d7: OutputPin,
}

impl LCDController {
    pub fn new(
        lcd_rs: u8,
        lcd_e: u8,
        lcd_d4: u8,
        lcd_d5: u8,
        lcd_d6: u8,
        lcd_d7: u8,
    ) -> Result<LCDController, Box<dyn Error>> {
        let gpio = Gpio::new()?;
        let device_info = DeviceInfo::new()?;
        let mut controller = LCDController {
            device_info: device_info,
            lcd_rs: gpio.get(lcd_rs)?.into_output(),
            lcd_e: gpio.get(lcd_e)?.into_output(),
            lcd_d4: gpio.get(lcd_d4)?.into_output(),
            lcd_d5: gpio.get(lcd_d5)?.into_output(),
            lcd_d6: gpio.get(lcd_d6)?.into_output(),
            lcd_d7: gpio.get(lcd_d7)?.into_output(),
        };
        controller.init();
        controller.reset_data_pins();
        Ok(controller)
    }

    /// IMPORTANT!!! READ BEFORE USING DEFAULT!!!
    ///
    /// the default pin configuration may not work for your case and may damage the device if inproperly wired,
    /// make sure your read the guide on connecting the raspberry pi in the README,
    /// this will work with that specific wiring configuration
    pub fn default() -> Result<LCDController, Box<dyn Error>> {
        let gpio = Gpio::new()?;
        let device_info: DeviceInfo = DeviceInfo::new()?;
        let mut controller = LCDController {
            device_info: device_info,
            lcd_rs: gpio.get(LCD_RS)?.into_output(),
            lcd_e: gpio.get(LCD_E)?.into_output(),
            lcd_d4: gpio.get(LCD_D4)?.into_output(),
            lcd_d5: gpio.get(LCD_D5)?.into_output(),
            lcd_d6: gpio.get(LCD_D6)?.into_output(),
            lcd_d7: gpio.get(LCD_D7)?.into_output(),
        };
        controller.init();
        controller.reset_data_pins();
        Ok(controller)
    }

    /// Initializes the LCD display
    fn init(&mut self) {
        self.send(LcdCommand::Initialize as u8, LcdMode::Command); // Initialize
        self.send(LcdCommand::Set4BitMode as u8, LcdMode::Command); // Set to 4-bit mode
        self.send(LcdCommand::SetCursorMoveDirrection as u8, LcdMode::Command); // Cursor move direction
        self.send(LcdCommand::SetCursorOff as u8, LcdMode::Command); // Turn cursor off
        self.send(LcdCommand::Set2LineDisplay as u8, LcdMode::Command); // 2 line display
        self.send(LcdCommand::ClearScreen as u8, LcdMode::Command); // Clear display
        let duration = Duration::from_millis(1);
        thread::sleep(duration); // Delay to allow commands to process
    }

    /// Sends a pulse to the LCD_E pin to indicate the end of data stream
    fn enable(&mut self) {
        let duration = Duration::from_millis(1);
        thread::sleep(duration);
        self.lcd_e.set_high();
        thread::sleep(duration);
        self.lcd_e.set_low();
        thread::sleep(duration);
    }

    /// Resets the data pin values to low
    pub fn reset_data_pins(&mut self) {
        self.lcd_d4.set_low();
        self.lcd_d5.set_low();
        self.lcd_d6.set_low();
        self.lcd_d7.set_low();
    }

    /// Resets all the pins values to low and clear the display
    pub fn reset(&mut self) {
        self.send(LcdCommand::ClearScreen as u8, LcdMode::Command); // Clear display
        self.lcd_rs.set_low();
        self.lcd_e.set_low();
        self.lcd_d4.set_low();
        self.lcd_d5.set_low();
        self.lcd_d6.set_low();
        self.lcd_d7.set_low();
        thread::sleep(Duration::from_millis(1));
    }

    /// Send a command or character to the display
    pub fn send(&mut self, bits: u8, mode: LcdMode) {
        // Changing pin value for RS changes the LCD mode
        match mode {
            LcdMode::Character => self.lcd_rs.set_high(),
            LcdMode::Command => self.lcd_rs.set_low(),
        }

        self.reset_data_pins();
        // High bits
        if bits & 0x10 == 0x10 {
            self.lcd_d4.set_high()
        }
        if bits & 0x20 == 0x20 {
            self.lcd_d5.set_high()
        }
        if bits & 0x40 == 0x40 {
            self.lcd_d6.set_high()
        }
        if bits & 0x80 == 0x80 {
            self.lcd_d7.set_high()
        }
        self.enable();

        self.reset_data_pins();

        // Low bits
        if bits & 0x01 == 0x01 {
            self.lcd_d4.set_high()
        }
        if bits & 0x02 == 0x02 {
            self.lcd_d5.set_high() // This is the problem pin
        }
        if bits & 0x04 == 0x04 {
            self.lcd_d6.set_high()
        }
        if bits & 0x08 == 0x08 {
            self.lcd_d7.set_high()
        }

        self.enable();
    }

    /// Display text on the LCD display
    ///
    /// Panics: if text length is > 16 (LCD_CHARS)
    /// Make sure to handle the incorect text length yourself,
    /// this is done to prevent unexpected behaviour.
    pub fn display_text(&mut self, text: &str, line: LcdLine) {
        if text.len() > LCD_CHARS {
            thread::sleep(Duration::from_millis(1));
            panic!("Text can't be longer than {LCD_CHARS} characters")
        }
        let text = pad_text(text, LCD_CHARS, " ");
        self.send(line as u8, LcdMode::Command);

        for i in 0..LCD_CHARS {
            if let Some(c) = text.chars().nth(i) {
                self.send(c as u8, LcdMode::Character)
            }
        }
    }

    /// Clear the screen of the LCD display
    pub fn clear_screen(&mut self) {
        self.send(LcdCommand::ClearScreen as u8, LcdMode::Command)
    }
}

impl Drop for LCDController {
    fn drop(&mut self) {
        // Reset the pin state to low and clears the screen when the controller goes out of scope.
        self.reset();
    }
}

impl std::fmt::Display for LCDController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LCD Controller\n")?;
        write!(f, "Device: {}\n", self.device_info.model())?;
        write!(f, "-------------------------------\n")?;

        write!(f, "PIN CONFIGURATION:\n")?;
        write!(f, "lcd_rs: GPIO {}\n", self.lcd_rs.pin())?;
        write!(f, "lcd_e : GPIO {}\n", self.lcd_e.pin())?;
        write!(f, "lcd_d4: GPIO {}\n", self.lcd_d4.pin())?;
        write!(f, "lcd_d5: GPIO {}\n", self.lcd_d5.pin())?;
        write!(f, "lcd_d6: GPIO {}\n", self.lcd_d6.pin())?;
        write!(f, "lcd_d7: GPIO {}\n", self.lcd_d7.pin())?;

        write!(f, "-------------------------------\n")?;
        write!(f, "PIN STATUS:\n")?;
        write!(f, "lcd_rs: {}\n", high_or_low(&self.lcd_rs))?;
        write!(f, "lcd_e : {}\n", high_or_low(&self.lcd_e))?;
        write!(f, "lcd_d4: {}\n", high_or_low(&self.lcd_d4))?;
        write!(f, "lcd_d5: {}\n", high_or_low(&self.lcd_d5))?;
        write!(f, "lcd_d6: {}\n", high_or_low(&self.lcd_d6))?;
        write!(f, "lcd_d7: {}\n", high_or_low(&self.lcd_d7))?;
        Ok(())
    }
}

fn high_or_low(pin: &OutputPin) -> String {
    match pin.is_set_low() {
        false => "high".to_owned(),
        true => "low".to_owned(),
    }
}

fn pad_text(text: &str, width: usize, pad_with: &str) -> String {
    let padding = width.saturating_sub(text.len());
    let padded_text = format!("{}{}", text, pad_with.repeat(padding));
    padded_text
}
