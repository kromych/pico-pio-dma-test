use core::cell::RefCell;
use core::fmt::Write;
use rp2040_hal::gpio::FunctionUart;
use rp2040_hal::uart::UartPeripheral;

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LogSourcePath {
    Enabled,
    Disabled,
}

pub type Uart = UartPeripheral<
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
    log_source_path: LogSourcePath,
    color: bool,
}

pub struct UartLogger {
    uart: Option<UartLoggerInner>,
}

impl UartLogger {
    pub const fn null() -> Self {
        UartLogger { uart: None }
    }

    pub fn set(&mut self, uart: Uart, log_source_path: LogSourcePath, color: bool) {
        self.uart = Some(UartLoggerInner {
            uart: RefCell::new(uart),
            log_source_path,
            color,
        });
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
        let color = self.uart.as_ref().unwrap().color;
        let vte_color = match record.level() {
            log::Level::Trace => "\x1b[37m",
            log::Level::Debug => "\x1b[36m",
            log::Level::Info => "\x1b[32m",
            log::Level::Warn => "\x1b[33m",
            log::Level::Error => "\x1b[31m",
        };
        if color {
            uart.write_str(vte_color).ok();
        }

        uart.write_fmt(format_args!(
            "{:08x}:[{:7}][{}",
            crate::time::time_us64(),
            record.level(),
            record.module_path().unwrap_or_default(),
        ))
        .ok();
        match log_source_path {
            LogSourcePath::Enabled => {
                uart.write_fmt(format_args!(
                    "{}@{}",
                    record.file().unwrap_or_default(),
                    record.line().unwrap_or_default(),
                ))
                .ok();
            }
            LogSourcePath::Disabled => {}
        }
        uart.write_fmt(format_args!("] {}", record.args())).ok();
        uart.write_str("\r\n").ok();

        // Reset color
        if color {
            uart.write_str("\x1b[0m").ok();
        }
    }

    fn flush(&self) {}
}

static mut UART_LOGGER: UartLogger = UartLogger::null();

pub fn init_uart_log(uart: Uart, log_source_path: LogSourcePath, color: bool) {
    #[allow(static_mut_refs)]
    unsafe {
        UART_LOGGER.set(uart, log_source_path, color);
        log::set_logger_racy(&UART_LOGGER).unwrap();
        log::set_max_level_racy(log::LevelFilter::Trace);
    }
}
