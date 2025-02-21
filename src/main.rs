#![no_std]
#![no_main]

//! Logs to the UART. The cable that I have should be connected to
//! the UART0 pins on the Pico. The pins are GPIO0 and GPIO1, and
//! the wires are BLUE -> GPIO0, GREEN -> GPIO1, and BLACK -> GND.
//!
//! ```sh
//! picocom -b 115200 -f n -d 8 -s 1 /dev/tty.usbmodem84102  # macOS
//! ```

use fugit::RateExtU32;
use rp2040_hal::gpio::FunctionUart;
use rp2040_hal::rom_data;
use rp2040_hal::uart::DataBits;
use rp2040_hal::uart::StopBits;
use rp2040_hal::uart::UartConfig;
use rp2040_hal::uart::UartPeripheral;
use rp2040_hal::Clock;
use uart_log::Uart;

mod time;
mod uart_log;

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

    // Initialize the UART logger
    {
        let uart_pins = (
            pins.gpio0.into_function::<FunctionUart>(),
            pins.gpio1.into_function::<FunctionUart>(),
        );

        let log_uart: Uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
            .enable(
                UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
                clocks.peripheral_clock.freq(),
            )
            .unwrap();

        uart_log::init_uart_log(log_uart, uart_log::LogSourcePath::Enabled);
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
