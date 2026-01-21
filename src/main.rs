use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
use fuji_heat_pump::fuji_controller::{ACMode, ControllerType, FujiController, FujiUartDriver};

use esp_idf_svc::hal::gpio;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::uart::*;
use esp_idf_svc::hal::units::*;
use esp_idf_svc::sys::{xTaskCreatePinnedToCore, EspError};

struct Driver<'a> {
    uart: UartDriver<'a>
}

impl<'a> FujiUartDriver<EspError> for Driver<'a> {
    fn send_frame(&self, frame: &[u8; 8]) -> Result<usize, EspError> {
        self.uart.write(frame)
    }
    fn read_frame(&self, buf: &mut [u8]) -> Result<usize, EspError> {
        self.uart.read(buf, 200)
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();


    //controller.set_mode(ACMode::Cool);
    //uart.read()

    let peripherals = Peripherals::take().expect("Failed to take Peripherals");
    let tx = peripherals.pins.gpio12;
    let rx = peripherals.pins.gpio13;

    let config = config::Config::new()
        .baudrate(Hertz(500))
        .data_bits(config::DataBits::DataBits8)
        .parity_even()
        .stop_bits(config::StopBits::STOP1);
    let uart = UartDriver::new(
        peripherals.uart1,
        tx,
        rx,
        Option::<gpio::Gpio0>::None,
        Option::<gpio::Gpio1>::None,
        &config,
    ).expect("Failed to initialize UART");

    let fuji_driver = Box::new(Driver { uart });
    let mut controller = FujiController::new(ControllerType::Primary, fuji_driver);
    let mutex = controller.spawn_thread();

    let mut c = mutex.lock().unwrap();
    c.set_mode(ACMode::Auto);
    drop(c);

    log::info!("Hello, world!");
}
