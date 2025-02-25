use crate::lax_dma;
use crate::lax_dma::Config;
use crate::lax_dma::Destination;
use crate::lax_dma::LaxDmaWrite;
use crate::lax_dma::Source;
use crate::lax_dma::TxReq;
use crate::lax_dma::TxSize;
use rp2040_hal::dma;
use rp2040_hal::pio::PIOExt;
use rp2040_pac::PIO0;
use rp2040_pac::RESETS;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum MonochromeColor {
    Bpp1 = 1,
    Bpp2 = 2,
    Bpp4 = 4,
}

struct TestConfig {
    src: &'static mut [u8; 4],
    dst: &'static mut [u8; 4],
    expected: [u8; 4],
    word_size: lax_dma::TxSize,
    byte_swap: bool,
    increment_src: bool,
    increment_dst: bool,
    test_name: &'static str,
}

fn run_dma_test<CHID: dma::ChannelIndex>(config: TestConfig) {
    let TestConfig {
        src,
        dst,
        expected,
        word_size,
        byte_swap,
        increment_src,
        increment_dst,
        test_name,
    } = config;

    log::info!("*** Running DMA test {}, channel {}", test_name, CHID::id());

    // Calculate the transaction count based on the word size
    let tx_count = match word_size {
        lax_dma::TxSize::_8bit => dst.len() as u32,
        lax_dma::TxSize::_16bit => dst.len() as u32 / core::mem::size_of::<u16>() as u32,
        lax_dma::TxSize::_32bit => dst.len() as u32 / core::mem::size_of::<u32>() as u32,
    };

    // Configure the DMA transfer
    let dma_config = lax_dma::Config {
        high_priority: false,
        word_size,
        source: lax_dma::Source {
            address: src.as_ptr(),
            increment: increment_src,
        },
        destination: lax_dma::Destination {
            address: dst.as_mut_ptr(),
            increment: increment_dst,
        },
        tx_count,
        tx_req: lax_dma::TxReq::Permanent,
        byte_swap,
        start: false,
    };

    let dma = lax_dma::LaxDmaWrite::new::<CHID>(dma_config);
    log::debug!("DMA source addr: {:x}", src.as_ptr() as usize);
    log::debug!("DMA dest addr: {:x}", dst.as_ptr() as usize);
    log::debug!("src: {:?}", src);
    log::debug!("dst: {:?}", dst);

    // Start the DMA transfer
    log::debug!("Starting DMA");
    dma.trigger();
    dma.wait();
    log::debug!("DMA done");

    // Log final state
    log::debug!("src: {:?}", src);
    log::debug!("dst: {:?}", dst);

    // Check for DMA errors
    log::debug!("DMA read error: {:?}", dma.read_error());
    log::debug!("DMA write error: {:?}", dma.write_error());
    log::debug!("DMA last read addr: {:x}", dma.last_read_addr() as usize);
    log::debug!("DMA last write addr: {:x}", dma.last_write_addr() as usize);
    log::debug!("DMA tx count remaining: {:?}", dma.tx_count_remaining());

    // Validate the result
    if dst != &expected {
        log::error!(
            "!!! {} failed! Expected: {:?}, got: {:?}",
            test_name,
            expected,
            dst
        );
    } else {
        log::info!(
            "*** {} passed. Expected: {:?}, got: {:?}",
            test_name,
            expected,
            dst
        );
    }
}

pub fn run_dma_tests() {
    // Define the test configurations
    let tests = [
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [42, 43, 44, 45],
            word_size: lax_dma::TxSize::_8bit,
            byte_swap: false,
            increment_src: true,
            increment_dst: true,
            test_name: "dma_test_8bit",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [42, 43, 44, 45],
            word_size: lax_dma::TxSize::_16bit,
            byte_swap: false,
            increment_src: true,
            increment_dst: true,
            test_name: "dma_test_16bit",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [42, 43, 44, 45],
            word_size: lax_dma::TxSize::_32bit,
            byte_swap: false,
            increment_src: true,
            increment_dst: true,
            test_name: "dma_test_32bit",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [42, 43, 44, 45],
            word_size: lax_dma::TxSize::_8bit,
            byte_swap: true,
            increment_src: true,
            increment_dst: true,
            test_name: "dma_test_8bit_byte_swap",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [43, 42, 45, 44],
            word_size: lax_dma::TxSize::_16bit,
            byte_swap: true,
            increment_src: true,
            increment_dst: true,
            test_name: "dma_test_16bit_byte_swap",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [45, 44, 43, 42],
            word_size: lax_dma::TxSize::_32bit,
            byte_swap: true,
            increment_src: true,
            increment_dst: true,
            test_name: "dma_test_32bit_byte_swap",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [42, 42, 42, 42],
            word_size: lax_dma::TxSize::_8bit,
            byte_swap: false,
            increment_src: false,
            increment_dst: true,
            test_name: "dma_test_8bit_fill",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [42, 43, 42, 43],
            word_size: lax_dma::TxSize::_16bit,
            byte_swap: false,
            increment_src: false,
            increment_dst: true,
            test_name: "dma_test_16bit_fill",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [42, 43, 44, 45],
            word_size: lax_dma::TxSize::_32bit,
            byte_swap: false,
            increment_src: false,
            increment_dst: true,
            test_name: "dma_test_32bit_fill",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [45, 0, 0, 0],
            word_size: lax_dma::TxSize::_8bit,
            byte_swap: false,
            increment_src: true,
            increment_dst: false,
            test_name: "dma_test_8bit_dst_fixed",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [44, 45, 0, 0],
            word_size: lax_dma::TxSize::_16bit,
            byte_swap: false,
            increment_src: true,
            increment_dst: false,
            test_name: "dma_test_16bit_dst_fixed",
        },
        TestConfig {
            src: cortex_m::singleton!(: [u8; 4] = [42, 43, 44, 45]).unwrap(),
            dst: cortex_m::singleton!(: [u8; 4] = [0; 4]).unwrap(),
            expected: [42, 43, 44, 45],
            word_size: lax_dma::TxSize::_32bit,
            byte_swap: false,
            increment_src: true,
            increment_dst: false,
            test_name: "dma_test_32bit_dst_fixed",
        },
    ];

    for test in tests.into_iter() {
        run_dma_test::<dma::CH5>(test);
    }
}

pub fn test_with_pio_invert_twice(pio: PIO0, resets: &mut RESETS) {
    // | DMA Channel | Source (Read Address)      | Destination (Write Address) | FIFO Connection           | Shift Register              |
    // |-------------|----------------------------|-----------------------------|---------------------------|-----------------------------|
    // | DMA 1 (TX)  | RAM Buffer                 | PIO TX FIFO (PIO0_TXF_SM0)  | TX FIFO feeds OSR         | OSR (Output Shift Register) |
    // | DMA 2 (RX)  | PIO RX FIFO (PIO0_RXF_SM0) | RAM Buffer                  | RX FIFO receives from ISR | ISR (Input Shift Register)  |

    const SIZE: usize = 32;
    let input_buffer = [0x55u8; SIZE];
    let mut output_buffer = [0u8; SIZE];
    let input_buffer_addr = [input_buffer.as_ptr() as u32];

    let (mut pio, sm0, sm1, _, _) = pio.split(resets);

    let invert_pio = pio_proc::pio_asm!(
        "more:",
        "       pull",           // PIO TX FIFO -> OSR (no need if `autopull` is true)
        "       mov     x, osr", // OSR -> x (same as `out x, 32` for shifting 32 bits from OSR)
        "       mov     x, ~x",  // ~x -> x (bitwise invert)
        "       mov     isr, x", // x -> ISR (same as `in x, 32` as shifting 32 bits into ISR)
        "       push",           // ISR -> PIO TX FIFO (no need if `autopush` is true)
        "       irq     wait 4",
        "       jmp     !osre, more",
    );

    let invert_pio_again = pio_proc::pio_asm!(
        "more:",
        "       wait    1 irq 4",
        "       pull",           // PIO TX FIFO -> OSR (no need if `autopull` is true)
        "       mov     x, osr", // OSR -> x (same as `out x, 32` for shifting 32 bits from OSR)
        "       mov     x, ~x",  // ~x -> x (bitwise invert)
        "       mov     isr, x", // x -> ISR (same as `in x, 32` as shifting 32 bits into ISR)
        "       push",           // ISR -> PIO TX FIFO (no need if `autopush` is true)
        "       jmp     !osre, more",
    );

    let (sm0, rx0, tx0) = rp2040_hal::pio::PIOBuilder::from_installed_program(
        pio.install(&invert_pio.program).unwrap(),
    )
    .autopull(false)
    .autopush(false)
    .build(sm0);
    sm0.start();

    let (sm1, rx1, tx1) = rp2040_hal::pio::PIOBuilder::from_installed_program(
        pio.install(&invert_pio_again.program).unwrap(),
    )
    .autopull(false)
    .autopush(false)
    .build(sm1);
    sm1.start();

    let txf0 = tx0.fifo_address();
    let rxf0 = rx0.fifo_address();

    let txf1 = tx1.fifo_address();
    let rxf1 = rx1.fifo_address();

    log::info!("input_buffer: {:02x?}", input_buffer);
    log::info!("output_buffer: {:02x?}", output_buffer);

    // This DMA channel transfers data from the PIO state machine's
    // RX FIFO to the output buffer. It will be stalled until the
    // next DMA channel is started and feeds the PIO TX FIFO.
    let dma3 = LaxDmaWrite::new::<dma::CH3>(Config {
        high_priority: false,
        word_size: TxSize::_32bit,
        source: Source {
            address: rxf1.cast(),
            increment: false,
        },
        destination: Destination {
            address: output_buffer.as_mut_ptr(),
            increment: true,
        },
        tx_count: SIZE as u32 / 4,
        tx_req: TxReq::Pio0Rx1,
        byte_swap: false,
        start: true,
    });

    // This DMA channel transfers data from the PIO state machine's
    // RX FIFO to the output buffer. It will be stalled until the
    // next DMA channel is started and feeds the PIO TX FIFO.
    let dma2 = LaxDmaWrite::new::<dma::CH2>(Config {
        high_priority: false,
        word_size: TxSize::_32bit,
        source: Source {
            address: rxf0.cast(),
            increment: false,
        },
        destination: Destination {
            address: txf1.cast_mut().cast(),
            increment: false,
        },
        tx_count: SIZE as u32 / 4,
        tx_req: TxReq::Pio0Tx1,
        byte_swap: false,
        start: true,
    });

    // This DMA channel transfers data from the input buffer to the PIO state machine's TX FIFO.
    // If this one is chained to dma0 (that writes to this channel's read trigger address),
    // the two will be res-starting together, running in the ping-pong mode.
    let dma1 = LaxDmaWrite::new::<dma::CH1>(Config {
        high_priority: false,
        word_size: TxSize::_32bit,
        source: Source {
            address: core::ptr::null(),
            increment: true,
        },
        destination: Destination {
            address: txf0.cast_mut().cast(),
            increment: false,
        },
        tx_count: SIZE as u32 / 4,
        tx_req: TxReq::Pio0Tx0,
        byte_swap: false,
        start: false,
    });

    // This DMA channel is used to configure the next one by writing to
    // the channel read address trigger register.
    let dma0 = LaxDmaWrite::new::<dma::CH0>(Config {
        high_priority: false,
        word_size: TxSize::_32bit,
        source: Source {
            address: input_buffer_addr.as_ptr().cast(),
            increment: false,
        },
        destination: Destination {
            address: dma1.read_trig_addr().cast_mut().cast(),
            increment: false,
        },
        tx_count: 1,
        tx_req: TxReq::Permanent,
        byte_swap: false,
        start: false,
    });

    // Start the DMA transfers
    dma0.trigger();

    // Wait for the DMA transfers to complete
    dma0.wait();
    dma2.wait();
    dma3.wait();

    log::info!("input_buffer: {:02x?}", input_buffer);
    log::info!("output_buffer: {:02x?}", output_buffer);
}

pub fn test_with_pio_expand_12times(pio: PIO0, resets: &mut RESETS) {
    // | DMA Channel | Source (Read Address)      | Destination (Write Address) | FIFO Connection           | Shift Register              |
    // |-------------|----------------------------|-----------------------------|---------------------------|-----------------------------|
    // | DMA 1 (TX)  | RAM Buffer                 | PIO TX FIFO (PIO0_TXF_SM0)  | TX FIFO feeds OSR         | OSR (Output Shift Register) |
    // | DMA 2 (RX)  | PIO RX FIFO (PIO0_RXF_SM0) | RAM Buffer                  | RX FIFO receives from ISR | ISR (Input Shift Register)  |

    const SIZE: usize = 4;
    let input_buffer = [0x5au8; SIZE];
    let mut output_buffer = [0u8; 12 * SIZE]; // bpp = 1; 12 /bpp

    let (mut pio, sm0, _, _, _) = pio.split(resets);

    // bpp = 1, greyscale (effectively BW) so R == G == B, each
    // repeating 12 times within RGB444.
    // If a pixel == 1, produce twelve 1's,
    // if a pixel == 0, produce twelve 0's.
    let expand_times12_pio = pio_proc::pio_asm!(
        ".wrap_target",
        "           out     x, 1",  // bpp
        "           set     y, 11", // 12/bpp - 1
        "repeat:",
        "           in      x, 1", // bpp
        "           jmp     y--, repeat",
        ".wrap"
    );

    let installed_pio = pio.install(&expand_times12_pio.program).unwrap();
    let (sm, rx, tx) = rp2040_hal::pio::PIOBuilder::from_installed_program(installed_pio)
        .autopull(true)
        .autopush(true)
        .build(sm0);
    sm.start();

    let txf = tx.fifo_address();
    let rxf = rx.fifo_address();

    log::info!("input_buffer: {:02x?}", input_buffer);
    log::info!("output_buffer: {:02x?}", output_buffer);

    // This DMA channel transfers data from the input buffer to the PIO state machine's TX FIFO.
    let dma1 = LaxDmaWrite::new::<dma::CH1>(Config {
        high_priority: false,
        word_size: TxSize::_32bit,
        source: Source {
            address: input_buffer.as_ptr(),
            increment: true,
        },
        destination: Destination {
            address: txf.cast_mut().cast(),
            increment: false,
        },
        tx_count: SIZE as u32 / 4,
        tx_req: TxReq::Pio0Tx0,
        byte_swap: false,
        start: false,
    });

    // This DMA channel transfers data from the PIO state machine's
    // RX FIFO to the output buffer
    let dma2 = LaxDmaWrite::new::<dma::CH2>(Config {
        high_priority: false,
        word_size: TxSize::_32bit,
        source: Source {
            address: rxf.cast(),
            increment: false,
        },
        destination: Destination {
            address: output_buffer.as_mut_ptr(),
            increment: true,
        },
        tx_count: 12 * SIZE as u32 / 4,
        tx_req: TxReq::Pio0Rx0,
        byte_swap: false,
        start: false,
    });

    // Start the DMA transfers
    dma1.trigger();
    dma2.trigger();

    // Wait for the DMA transfers to complete
    dma1.wait();
    dma2.wait();

    log::info!("input_buffer: {:02x?}", input_buffer);
    log::info!("output_buffer: {:02x?}", output_buffer);
}

/// Generates a PIO program to produce greyscale color encoded as RGB444
/// physically. Each pixel may have 2, 4, or 16 greyscale levels (1, 2, or 4 bpp).
fn greyscale_pio(color: MonochromeColor) -> pio::Program<{ pio::RP2040_MAX_PROGRAM_SIZE }> {
    let mut a = pio::Assembler::<{ pio::RP2040_MAX_PROGRAM_SIZE }>::new();

    const RGB_BPP: u8 = 12;
    let bpp = color as u8;

    let mut repeat = a.label();

    // Pull `bpp` bits (1, 2, or 4) from the TX FIFO into OSR
    a.out(pio::OutDestination::X, bpp);

    // Loop counter in `Y` to repeat `bpp` as many times as need
    // to fill RGB444 for the greyscale color.
    a.set(pio::SetDestination::Y, RGB_BPP / bpp - 1);
    a.bind(&mut repeat);
    // Push the bits into ISR which goes into RX FIFO.
    a.r#in(pio::InSource::X, bpp);
    // Repeat
    a.jmp(pio::JmpCondition::YDecNonZero, &mut repeat);

    a.assemble_program()
}

pub fn test_with_pio_expand_dynamic(pio: PIO0, resets: &mut RESETS, color: MonochromeColor) {
    // | DMA Channel | Source (Read Address)      | Destination (Write Address) | FIFO Connection           | Shift Register              |
    // |-------------|----------------------------|-----------------------------|---------------------------|-----------------------------|
    // | DMA 1 (TX)  | RAM Buffer                 | PIO TX FIFO (PIO0_TXF_SM0)  | TX FIFO feeds OSR         | OSR (Output Shift Register) |
    // | DMA 2 (RX)  | PIO RX FIFO (PIO0_RXF_SM0) | RAM Buffer                  | RX FIFO receives from ISR | ISR (Input Shift Register)  |

    const RGB_BPP: u8 = 12;
    let bpp = RGB_BPP / color as u8;

    const SIZE: usize = 8;
    let input_buffer: [u8; SIZE] = [0xaa; SIZE];
    let mut output_buffer: [u8; 12 * SIZE] = [0u8; 12 * SIZE]; // Max output size, each input bit repeated 12 times (greyscale RGB444)

    let (mut pio, sm0, _, _, _) = pio.split(resets);

    let greyscale_pio = greyscale_pio(color);
    let installed_pio = pio.install(&greyscale_pio).unwrap();
    let (sm, rx, tx) = rp2040_hal::pio::PIOBuilder::from_installed_program(installed_pio)
        .autopull(true)
        .autopush(true)
        .build(sm0);
    sm.start();

    let txf = tx.fifo_address();
    let rxf = rx.fifo_address();

    log::info!("input_buffer: {:02x?}", input_buffer);
    log::info!("output_buffer: {:02x?}", output_buffer);

    // This DMA channel transfers data from the input buffer to the PIO state machine's TX FIFO.
    let dma1 = LaxDmaWrite::new::<dma::CH1>(Config {
        high_priority: false,
        word_size: TxSize::_32bit,
        source: Source {
            address: input_buffer.as_ptr(),
            increment: true,
        },
        destination: Destination {
            address: txf.cast_mut().cast(),
            increment: false,
        },
        tx_count: SIZE as u32 / 4,
        tx_req: TxReq::Pio0Tx0,
        byte_swap: false,
        start: false,
    });

    // This DMA channel transfers data from the PIO state machine's
    // RX FIFO to the output buffer.
    let dma2 = LaxDmaWrite::new::<dma::CH2>(Config {
        high_priority: false,
        word_size: TxSize::_32bit,
        source: Source {
            address: rxf.cast(),
            increment: false,
        },
        destination: Destination {
            address: output_buffer.as_mut_ptr(),
            increment: true,
        },
        tx_count: bpp as u32 * SIZE as u32 / 4,
        tx_req: TxReq::Pio0Rx0,
        byte_swap: false,
        start: false,
    });

    // Start the DMA transfers
    dma1.trigger();
    dma2.trigger();

    // Wait for the DMA transfers to complete
    dma1.wait();
    dma2.wait();

    log::info!("input_buffer: {:02x?}", input_buffer);
    log::info!("output_buffer: {:02x?}", output_buffer);
}
