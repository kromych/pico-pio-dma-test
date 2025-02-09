#![no_std]
#![no_main]

//! Logs to the UART. The cable that I have should be connected to
//! the UART0 pins on the Pico. The pins are GPIO0 and GPIO1, and
//! the wires are BLUE -> GPIO0, GREEN -> GPIO1, and BLACK -> GND.
//!
//! ```sh
//! picocom -b 115200 -f n -d 8 -s 1 /dev/tty.usbmodem84102  # macOS
//! ```

use core::cell::RefCell;
use core::fmt::Write;
use fugit::RateExtU32;
use rp2040_hal::gpio::FunctionUart;
use rp2040_hal::rom_data;
use rp2040_hal::uart::DataBits;
use rp2040_hal::uart::StopBits;
use rp2040_hal::uart::UartConfig;
use rp2040_hal::uart::UartPeripheral;
use rp2040_hal::Clock;

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
/// Note: This boot block is not necessary when using a rp-hal based BSP
/// as the BSPs already perform this step.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

/// Program metadata for `picotool info`
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [rp2040_hal::binary_info::EntryAddr; 4] = [
    rp2040_hal::binary_info::rp_program_name!(c"Pico PIO DMA test"),
    rp2040_hal::binary_info::rp_program_description!(c"Pico PIO DMA experiments"),
    rp2040_hal::binary_info::rp_program_build_attribute!(),
    rp2040_hal::binary_info::rp_cargo_version!(),
];

mod time {
    pub fn time_us() -> u32 {
        unsafe { (*rp2040_pac::TIMER::PTR).timerawl().read().bits() }
    }

    pub fn time_us64() -> u64 {
        unsafe {
            (*rp2040_pac::TIMER::PTR).timelr().read().bits() as u64
                | (((*rp2040_pac::TIMER::PTR).timehr().read().bits() as u64) << 32)
        }
    }
}

type Uart = UartPeripheral<
    rp2040_hal::uart::Enabled,
    rp2040_pac::UART0,
    (
        rp2040_hal::gpio::Pin<
            rp2040_hal::gpio::bank0::Gpio0,
            FunctionUart,
            rp2040_hal::gpio::PullDown,
        >,
        rp2040_hal::gpio::Pin<
            rp2040_hal::gpio::bank0::Gpio1,
            FunctionUart,
            rp2040_hal::gpio::PullDown,
        >,
    ),
>;

pub struct UartLoggerInner {
    uart: RefCell<Uart>,
    log_source_path: bool,
}

pub struct UartLogger {
    uart: Option<UartLoggerInner>,
}

impl UartLogger {
    pub const fn null() -> Self {
        UartLogger { uart: None }
    }
}

unsafe impl Send for UartLogger {}
unsafe impl Sync for UartLogger {}

impl log::Log for UartLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.uart.is_none() {
            return;
        }
        let mut uart = self.uart.as_ref().unwrap().uart.borrow_mut();
        let log_source_path = self.uart.as_ref().unwrap().log_source_path;
        uart.write_fmt(format_args!(
            "{:08x}[{:7}][{}",
            time::time_us64(),
            record.level(),
            record.module_path().unwrap_or_default(),
        ))
        .ok();
        if log_source_path {
            uart.write_fmt(format_args!(
                "{}@{}",
                record.file().unwrap_or_default(),
                record.line().unwrap_or_default(),
            ))
            .ok();
        }
        uart.write_fmt(format_args!("] {}", record.args())).ok();
        uart.write_str("\r\n").ok();
    }

    fn flush(&self) {}
}

static mut UART_LOGGER: UartLogger = UartLogger::null();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}

#[rp2040_hal::entry]
fn main() -> ! {
    let mut pac = rp2040_pac::Peripherals::take().unwrap();
    let _core = rp2040_pac::CorePeripherals::take().unwrap();

    // Give more priority to the DMA peripheral
    pac.BUSCTRL.bus_priority().write(|w| {
        w.dma_r().set_bit();
        w.dma_w().set_bit()
    });

    let mut watchdog = rp2040_hal::watchdog::Watchdog::new(pac.WATCHDOG);
    let clocks = rp2040_hal::clocks::init_clocks_and_plls(
        crate::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = rp2040_hal::sio::Sio::new(pac.SIO);
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_pins = (
        pins.gpio0.into_function::<FunctionUart>(),
        pins.gpio1.into_function::<FunctionUart>(),
    );

    // Create a UART driver
    let log_uart: Uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let uart_logger = UartLogger {
        uart: Some(UartLoggerInner {
            uart: RefCell::new(log_uart),
            log_source_path: true,
        }),
    };

    #[allow(static_mut_refs)]
    // TODO: Make this look saner.
    unsafe {
        UART_LOGGER = uart_logger;
        log::set_logger_racy(&UART_LOGGER).unwrap();
        log::set_max_level_racy(log::LevelFilter::Trace);
    }

    log::info!(
        "Board {}, git revision {:x}, ROM verion {:x}, time {:x} us",
        rom_data::copyright_string(),
        rom_data::git_revision(),
        rom_data::rom_version_number(),
        time::time_us64()
    );

    loop {
        //cortex_m::asm::wfe();
        log::info!("WFE time: {:x}", time::time_us());
        cortex_m::asm::delay(100_000_000);
    }
}
