// =======================
//  pico-toy-elevator
//
//  by tkhshmsy@gmail.com
// =======================

#![no_std]
#![no_main]

use bsp::entry;
// use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin, PinState, ToggleableOutputPin};
use panic_probe as _;

use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    fugit::RateExtU32,
    pac,
    sio::Sio,
    uart::{DataBits, StopBits, UartConfig},
    watchdog::Watchdog,
};

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, Triangle},
    text::{Baseline, Text},
};
use font_7seg::Font7Seg;
use shared_bus::BusManagerSimple;
use ssd1306::{prelude::*, Ssd1306};

pub mod lift;
use lift::{Directions, FloorState, LiftState};
pub mod voice;

static OFFSET_X: i32 = 8;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();

    // GPIO
    let btn_a = pins.gpio2.into_pull_up_input();
    let btn_b = pins.gpio3.into_pull_up_input();
    let mut led_a = pins.gpio4.into_push_pull_output();
    let mut led_b = pins.gpio5.into_push_pull_output();
    let nplay = pins.gpio6.into_pull_up_input();
    //gpio7 is unused
    let btn_3 = pins.gpio8.into_pull_up_input();
    let btn_4 = pins.gpio9.into_pull_up_input();
    let mut led_3 = pins.gpio10.into_push_pull_output();
    let mut led_4 = pins.gpio11.into_push_pull_output();
    let mut led_2 = pins.gpio12.into_push_pull_output();
    let mut led_1 = pins.gpio13.into_push_pull_output();
    let btn_1 = pins.gpio14.into_pull_up_input();
    let btn_2 = pins.gpio15.into_pull_up_input();
    let mut led_6 = pins.gpio18.into_push_pull_output();
    let mut led_5 = pins.gpio19.into_push_pull_output();
    let btn_6 = pins.gpio20.into_pull_up_input();
    let btn_5 = pins.gpio21.into_pull_up_input();
    let mut led_8 = pins.gpio22.into_push_pull_output();
    let mut led_7 = pins.gpio26.into_push_pull_output();
    let btn_8 = pins.gpio27.into_pull_up_input();
    let btn_7 = pins.gpio28.into_pull_up_input();

    // wait for booting up i2c devices
    delay.delay_ms(1000);

    // UART0
    let uart_pins = (pins.gpio0.into_function(), pins.gpio1.into_function());
    let uart0 = bsp::hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    // I2C0
    // let i2c_pins = (pins.gpio16.into_function(), pins.gpio17.into_function());
    let sda_pin = pins.gpio16.into_function();
    let scl_pin = pins.gpio17.into_function();
    let i2c = bsp::hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );
    let i2c_bus = BusManagerSimple::new(i2c);
    let ssd1306_proxy = i2c_bus.acquire_i2c();

    delay.delay_ms(1000);
    let ssd1306_interface = ssd1306::I2CDisplayInterface::new(ssd1306_proxy);
    let mut display = Ssd1306::new(
        ssd1306_interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    display.init().unwrap();
    let line_style = PrimitiveStyle::with_stroke(BinaryColor::Off, 5);
    let fill_style = PrimitiveStyle::with_fill(BinaryColor::Off);
    let erase_style = PrimitiveStyle::with_fill(BinaryColor::On);
    let font7seg_style = Font7Seg::new(Size::new(30, 60), BinaryColor::Off);
    display.clear();
    display.flush().unwrap();

    // ===========
    //  main loop
    // ===========
    let mut lift = lift::Lift::new();
    let mut voice = voice::Voice::new(uart0).unwrap();

    let mut heartbeat_counter = 0;
    let mut animation_counter = 0;
    let mut prev_floor = FloorState::FloorMax;

    voice.talk("#K\r");
    delay.delay_ms(1000);
    voice.talk("#J\r");
    delay.delay_ms(1000);

    loop {
        delay.delay_ms(33);

        //heart beat
        heartbeat_counter += 1;
        if heartbeat_counter % 15 == 0 {
            led_pin.toggle().unwrap();
        }

        // key input
        {
            if btn_1.is_low().unwrap() {
                lift.set_floor_key(FloorState::Floor8th, true);
            }
            if btn_2.is_low().unwrap() {
                lift.set_floor_key(FloorState::Floor4th, true);
            }
            if btn_3.is_low().unwrap() {
                lift.set_floor_key(FloorState::Floor7th, true);
            }
            if btn_4.is_low().unwrap() {
                lift.set_floor_key(FloorState::Floor3rd, true);
            }
            if btn_5.is_low().unwrap() {
                lift.set_floor_key(FloorState::Floor6th, true);
            }
            if btn_6.is_low().unwrap() {
                lift.set_floor_key(FloorState::Floor2nd, true);
            }
            if btn_7.is_low().unwrap() {
                lift.set_floor_key(FloorState::Floor5th, true);
            }
            if btn_8.is_low().unwrap() {
                lift.set_floor_key(FloorState::Floor1st, true);
            }
            if btn_a.is_low().unwrap() {
                led_a.set_low().unwrap();
                lift.open();
            } else {
                led_a.set_high().unwrap();
            }
            if btn_b.is_low().unwrap() {
                led_b.set_low().unwrap();
                lift.close();
            } else {
                led_b.set_high().unwrap();
            }
        }

        // control floor led
        {
            let keys = lift.keys();
            led_1
                .set_state(PinState::from(!keys[FloorState::Floor8th as usize]))
                .unwrap();
            led_2
                .set_state(PinState::from(!keys[FloorState::Floor4th as usize]))
                .unwrap();
            led_3
                .set_state(PinState::from(!keys[FloorState::Floor7th as usize]))
                .unwrap();
            led_4
                .set_state(PinState::from(!keys[FloorState::Floor3rd as usize]))
                .unwrap();
            led_5
                .set_state(PinState::from(!keys[FloorState::Floor6th as usize]))
                .unwrap();
            led_6
                .set_state(PinState::from(!keys[FloorState::Floor2nd as usize]))
                .unwrap();
            led_7
                .set_state(PinState::from(!keys[FloorState::Floor5th as usize]))
                .unwrap();
            led_8
                .set_state(PinState::from(!keys[FloorState::Floor1st as usize]))
                .unwrap();
        }

        // display floor
        if prev_floor != *lift.floor_state() {
            Rectangle::new(
                Point::new(OFFSET_X + 64, 0),
                Size::new(128 - OFFSET_X as u32, 64),
            )
            .into_styled(erase_style)
            .draw(&mut display)
            .unwrap();
            let mut c = "0";
            match lift.floor_state() {
                FloorState::Floor1st => c = "1",
                FloorState::Floor2nd => c = "2",
                FloorState::Floor3rd => c = "3",
                FloorState::Floor4th => c = "4",
                FloorState::Floor5th => c = "5",
                FloorState::Floor6th => c = "6",
                FloorState::Floor7th => c = "7",
                FloorState::Floor8th => c = "8",
                FloorState::FloorMax => {}
            }
            Text::with_baseline(c, Point::new(64 + 16, 4), font7seg_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
            display.flush().unwrap();
        }

        // animation
        Rectangle::new(Point::new(0, 0), Size::new(64 + OFFSET_X as u32, 64))
            .into_styled(erase_style)
            .draw(&mut display)
            .unwrap();
        if animation_counter >= 0 {
            match lift.lift_state() {
                LiftState::Opening => {
                    let counter = animation_counter;
                    match counter {
                        65..=100 => {
                            // skip erase because already done
                            if (counter % 10) > 5 {
                                Triangle::new(
                                    Point::new(OFFSET_X + 32 - 10, 14),
                                    Point::new(OFFSET_X + 10, 32),
                                    Point::new(OFFSET_X + 32 - 10, 50),
                                )
                                .into_styled(fill_style)
                                .draw(&mut display)
                                .unwrap();
                                Triangle::new(
                                    Point::new(OFFSET_X + 32 + 10, 14),
                                    Point::new(OFFSET_X + 64 - 10, 32),
                                    Point::new(OFFSET_X + 32 + 10, 50),
                                )
                                .into_styled(fill_style)
                                .draw(&mut display)
                                .unwrap();
                            }
                        }
                        0..=64 => {
                            let dx = (counter / 2) as i32;
                            let width = (dx * 2) as u32;
                            Rectangle::new(Point::new(OFFSET_X + dx, 0), Size::new(64 - width, 64))
                                .into_styled(fill_style)
                                .draw(&mut display)
                                .unwrap();
                        }
                        _ => {}
                    }
                }
                LiftState::Waiting => {
                    Rectangle::new(Point::new(OFFSET_X + 0, 0), Size::new(64, 64))
                        .into_styled(fill_style)
                        .draw(&mut display)
                        .unwrap();
                    match lift.direction() {
                        Directions::Up => Triangle::new(
                            Point::new(OFFSET_X + 32, 16),
                            Point::new(OFFSET_X + 32 - 10, 48),
                            Point::new(OFFSET_X + 32 + 10, 48),
                        )
                        .into_styled(erase_style)
                        .draw(&mut display)
                        .unwrap(),
                        Directions::Down => Triangle::new(
                            Point::new(OFFSET_X + 32, 48),
                            Point::new(OFFSET_X + 32 - 10, 16),
                            Point::new(OFFSET_X + 32 + 10, 16),
                        )
                        .into_styled(erase_style)
                        .draw(&mut display)
                        .unwrap(),
                        Directions::None => {}
                    };
                }
                LiftState::Closing => {
                    let counter = animation_counter;
                    match counter {
                        65..=100 => {
                            Rectangle::new(Point::new(OFFSET_X + 0, 0), Size::new(64, 64))
                                .into_styled(fill_style)
                                .draw(&mut display)
                                .unwrap();
                            if (counter % 10) > 5 {
                                Triangle::new(
                                    Point::new(OFFSET_X + 10, 14),
                                    Point::new(OFFSET_X + 32 - 10, 32),
                                    Point::new(OFFSET_X + 10, 50),
                                )
                                .into_styled(erase_style)
                                .draw(&mut display)
                                .unwrap();
                                Triangle::new(
                                    Point::new(OFFSET_X + 64 - 10, 14),
                                    Point::new(OFFSET_X + 32 + 10, 32),
                                    Point::new(OFFSET_X + 64 - 10, 50),
                                )
                                .into_styled(erase_style)
                                .draw(&mut display)
                                .unwrap();
                            }
                        }
                        0..=64 => {
                            let dx = (counter / 2) as i32;
                            let width = (dx * 2) as u32;
                            Rectangle::new(Point::new(OFFSET_X + 32 - dx, 0), Size::new(width, 64))
                                .into_styled(fill_style)
                                .draw(&mut display)
                                .unwrap();
                        }
                        _ => {}
                    }
                }
                LiftState::Moving => {
                    let steps = [0, 5, 10, 15, 20, -20, -15, -10, -5, 0];
                    let diff = steps[(animation_counter % 10) as usize];
                    match lift.direction() {
                        Directions::Up => {
                            Triangle::new(
                                Point::new(OFFSET_X + 32, 8),
                                Point::new(OFFSET_X + 32 - diff, 56),
                                Point::new(OFFSET_X + 32 + diff, 56),
                            )
                            .into_styled(line_style)
                            .draw(&mut display)
                            .unwrap();
                        }
                        Directions::Down => {
                            Triangle::new(
                                Point::new(OFFSET_X + 32, 56),
                                Point::new(OFFSET_X + 32 - diff, 8),
                                Point::new(OFFSET_X + 32 + diff, 8),
                            )
                            .into_styled(line_style)
                            .draw(&mut display)
                            .unwrap();
                        }
                        Directions::None => {}
                    };
                }
                _ => {}
            }
            animation_counter -= 1;
        }

        display.flush().unwrap();
        prev_floor = lift.floor_state().clone();
        if nplay.is_low().unwrap() || animation_counter > 0 {
            continue;
        }

        // next state and floor
        let prev_lift_state = lift.lift_state().clone();
        lift.next();

        // talk
        if prev_lift_state != *lift.lift_state() || prev_floor != *lift.floor_state() {
            match lift.lift_state() {
                LiftState::Chime => {
                    match lift.direction() {
                        Directions::Up => voice.talk("#J\r"),
                        Directions::Down => voice.talk("#K\r"),
                        Directions::None => {}
                    };
                }
                LiftState::Arrived => {
                    match lift.floor_state() {
                        FloorState::Floor1st => {
                            voice.talk("<NUMK VAL=1 COUNTER=kai>de'_su.\r");
                        }
                        FloorState::Floor2nd => {
                            voice.talk("<NUMK VAL=2 COUNTER=kai>de'_su.\r");
                        }
                        FloorState::Floor3rd => {
                            voice.talk("<NUMK VAL=3 COUNTER=kai>de'_su.\r");
                        }
                        FloorState::Floor4th => {
                            voice.talk("<NUMK VAL=4 COUNTER=kai>de'_su.\r");
                        }
                        FloorState::Floor5th => {
                            voice.talk("<NUMK VAL=5 COUNTER=kai>de'_su.\r");
                        }
                        FloorState::Floor6th => {
                            voice.talk("<NUMK VAL=6 COUNTER=kai>de'_su.\r");
                        }
                        FloorState::Floor7th => {
                            voice.talk("<NUMK VAL=7 COUNTER=kai>de'_su.\r");
                        }
                        FloorState::Floor8th => {
                            voice.talk("<NUMK VAL=8 COUNTER=kai>de'_su.\r");
                        }
                        FloorState::FloorMax => {
                            voice.talk("kokowa/do'ko?\r");
                        }
                    };
                    animation_counter = 60;
                }
                LiftState::Opening => {
                    voice.talk("do'aga/hirakima'_su.\r");
                    animation_counter = 100;
                }
                LiftState::Waiting => {
                    match lift.direction() {
                        Directions::Up => voice.talk("ue'ni/mairima'_su.\r"),
                        Directions::Down => voice.talk("_shitani/mairima'_su.\r"),
                        Directions::None => {}
                    };
                    animation_counter = 60;
                }
                LiftState::Closing => {
                    voice.talk("do'aga/shimarima'_su.\r");
                    animation_counter = 100;
                }
                LiftState::Checking => voice.talk("i_kisakibo'tanno/o_shitekudasa'i.\r"),
                LiftState::Checked => {}
                LiftState::Moving => {
                    animation_counter = 60;
                }
            }
        }
    }
}

// End of file
