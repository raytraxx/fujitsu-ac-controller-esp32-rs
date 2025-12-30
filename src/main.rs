use esp_idf_svc::hal::delay::{Delay, FreeRtos};
use fuji_heat_pump::fuji_controller::{ACMode, ControllerType, FujiController, FujiUartDriver};

use esp_idf_svc::hal::gpio;
use esp_idf_svc::hal::gpio::{PinDriver, Pins};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::task::*;
use esp_idf_svc::hal::uart::*;
use esp_idf_svc::hal::units::*;
use esp_idf_svc::io::asynch::Read;
use esp_idf_svc::sys::EspError;

struct Driver<'a> {
    uart: UartDriver<'a>
}

impl<'a> FujiUartDriver<EspError> for Driver<'a> {
    fn send_frame(&mut self, frame: &[u8; 8]) -> Result<usize, EspError> {
        self.uart.write(frame)
    }
    fn read_frame(&mut self, buf: &mut [u8]) -> Result<usize, EspError> {
        self.uart.read(buf, 200)
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

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
    //controller.set_mode(ACMode::Cool);

    log::info!("Hello, world!");
}
