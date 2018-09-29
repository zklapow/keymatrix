#![no_std]
#![no_main]
#![feature(proc_macro_gen)]
#![recursion_limit="1024"]

extern crate atsamd21_hal as atsamd21;
extern crate samd21_mini as bsp;

extern crate cortex_m;
extern crate cortex_m_rt as rt;
#[macro_use]
extern crate cortex_m_rtfm as rtfm;
extern crate embedded_hal as hal;
extern crate generic_array;
#[macro_use]
extern crate keymatrix;
extern crate nb;
extern crate panic_abort;

use bsp::prelude::*;

use rtfm::Resource;
use rtfm::{app, Threshold};
use rt::entry;

use generic_array::functional::FunctionalSequence;
use generic_array::typenum::{U2, U3};
use generic_array::GenericArray;
use hal::prelude::*;

use bsp::atsamd21g18a::gclk::clkctrl::GENR;
use bsp::atsamd21g18a::gclk::genctrl::SRCR;
use bsp::clock::GenericClockController;
use bsp::gpio::*;
use bsp::prelude::*;
use bsp::sercom::{PadPin, Sercom0Pad2, Sercom0Pad3, UART0Pinout, UART0};

macro_rules! dbgprintln {
    ($uart:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        writeln!($uart, $($arg)*).ok();
    }};
}

macro_rules! dbgprint {
    ($uart:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        write!($uart, $($arg)*).ok();
    }};
}

key_columns!(
    KeyzCols,
    U3,
    [
        col0: (0, Pa14<Output<PushPull>>),
        col1: (1, Pa9<Output<PushPull>>),
        col2: (2, Pa8<Output<PushPull>>)
    ]
);

key_rows!(
    KeyzRows,
    U3,
    [
        row0: (0, Pa15<Input<PullDown>>),
        row1: (1, Pa20<Input<PullDown>>),
        row2: (2, Pa21<Input<PullDown>>)
    ]
);

type KeyzMatrix = keymatrix::KeyMatrix<U3, U3, KeyzCols, KeyzRows>;
type State = GenericArray<GenericArray<bool, U3>, U3>;

app! {
    device: atsamd21g18a,

    resources: {
        static UART: UART0;
        static KEYMATRIX: KeyzMatrix;
        static STATE: State;
        static TIMER: bsp::timer::TimerCounter3;
    },

    tasks: {
        TC3: {
            path: tc3_exec,
            resources: [TIMER, KEYMATRIX, STATE, UART],
        },
    }
}

fn tc3_exec(t: &mut Threshold, mut r: TC3::Resources) {
    // TODO: This is line is just to provide type hints for the IDE
    let matrix: &mut KeyzMatrix = &mut r.KEYMATRIX;
    let uart = r.UART.borrow_mut(t);
    if r.TIMER.wait().is_ok() {
        matrix.poll();

        let state: State = matrix.current_state();

        if !compare_state(&r.STATE, &state) {
            dbgprint!(uart, "{0: <10}", "row/col");
            for j in 0..matrix.col_size() {
                // Print col headers
                dbgprint!(uart, "|{0: <10}|", j);
            }
            dbgprint!(uart, "\n");

            for i in 0..matrix.row_size() {
                dbgprint!(uart, "{0: <10}", i);

                for j in 0..matrix.col_size() {
                    let bool = state[j][i];
                    dbgprint!(uart, "|{0: <10}|", bool)
                }

                dbgprint!(uart, "\n");
            }

            *r.STATE = state;
            dbgprint!(uart, "\n\n");
        }
    }
}

fn compare_state(a: &State, b: &State) -> bool {
    a.zip(b, |ea, eb| {
        ea.zip(eb, |ia, ib| ia == ib)
            .fold(true, |last, cur| last && cur)
    }).fold(true, |last, cur| last && cur)
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

fn init(mut p: init::Peripherals) -> init::LateResources {
    let mut clocks = GenericClockController::with_external_32kosc(
        p.device.GCLK,
        &mut p.device.PM,
        &mut p.device.SYSCTRL,
        &mut p.device.NVMCTRL,
    );
    clocks.configure_gclk_divider_and_source(GENR::GCLK2, 1, SRCR::DFLL48M, false);
    let glck0 = clocks.gclk0();
    let gclk2 = clocks.get_gclk(GENR::GCLK2).expect("Could not get clock 2");

    let mut pins = bsp::Pins::new(p.device.PORT);

    let mut tc3 = bsp::timer::TimerCounter::tc3_(
        &clocks.tcc2_tc3(&glck0).unwrap(),
        p.device.TC3,
        &mut p.device.PM,
    );

    let rx_pin: Sercom0Pad3 = pins
        .rx
        .into_pull_down_input(&mut pins.port)
        .into_pad(&mut pins.port);
    let tx_pin: Sercom0Pad2 = pins
        .tx
        .into_push_pull_output(&mut pins.port)
        .into_pad(&mut pins.port);
    let uart_clk = clocks
        .sercom0_core(&gclk2)
        .expect("Could not configure sercom0 core clock");

    let mut uart = UART0::new(
        &uart_clk,
        115200.hz(),
        p.device.SERCOM0,
        &mut p.core.NVIC,
        &mut p.device.PM,
        UART0Pinout::Rx3Tx2 {
            rx: rx_pin,
            tx: tx_pin,
        },
    );

    dbgprintln!(uart, "Init done\n");

    let col0 = pins.d2.into_push_pull_output(&mut pins.port);
    let col1 = pins.d3.into_push_pull_output(&mut pins.port);
    let col2 = pins.d4.into_push_pull_output(&mut pins.port);

    let row0 = pins.d5.into_pull_down_input(&mut pins.port);
    let row1 = pins.d6.into_pull_down_input(&mut pins.port);
    let row2 = pins.d7.into_pull_down_input(&mut pins.port);

    let cols = KeyzCols::new(col0, col1, col2);
    let rows = KeyzRows::new(row0, row1, row2);
    let matrix = keymatrix::KeyMatrix::new(&mut tc3, 1000.hz(), 5, cols, rows);
    let state = matrix.current_state();

    tc3.enable_interrupt();

    dbgprintln!(uart, "Matrix initialized\n");

    init::LateResources {
        UART: uart,
        KEYMATRIX: matrix,
        STATE: state,
        TIMER: tc3,
    }
}

#[entry]
fn run_app() -> ! {
    main();
    loop {}
}
