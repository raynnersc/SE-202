#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::{DMA1_CH5, PB14, PB6, PB7, USART1};
use embassy_stm32::usart::Uart;
use embassy_stm32::Config;
use embassy_stm32::{bind_interrupts, rcc::*, usart};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Ticker, Timer};
use heapless::box_pool;
use heapless::pool::boxed::{Box, BoxBlock};
use panic_probe as _;
use tp_led_matrix::{Color, Image, Matrix};
use futures::FutureExt;

static IMAGE: Mutex<ThreadModeRawMutex, Image> = Mutex::new(Image::new_solid(Color::GREEN));
static NEXT_IMAGE: Signal<ThreadModeRawMutex, Box<POOL>> = Signal::new();
box_pool!(POOL: Image);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Setup the clocks at 80MHz using HSI (by default since HSE/MSI
    // are not configured): HSI(16MHz)×10/2=80MHz. The flash wait
    // states will be configured accordingly.
    let mut config = Config::default();
    config.rcc.mux = ClockSrc::PLL1_R;
    config.rcc.hsi = true;
    config.rcc.pll = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL10,
        divp: None,
        divq: None,
        divr: Some(PllRDiv::DIV2), // 16 * 10 / 2 = 80MHz
    });
    let p = embassy_stm32::init(config);

    let matrix = Matrix::new(
        p.PA2, p.PA3, p.PA4, p.PA5, p.PA6, p.PA7, p.PA15, p.PB0, p.PB1, p.PB2, p.PC3, p.PC4, p.PC5,
    )
    .await;

    unsafe {
        const BLOCK: BoxBlock<Image> = BoxBlock::new();
        static mut MEMORY: [BoxBlock<Image>; 3] = [BLOCK; 3];
        for block in &mut MEMORY {
          POOL.manage(block);
        }
    }

    // spawner.spawn(change_image()).unwrap();
    spawner.spawn(serial_receiver(p.USART1, p.PB6, p.PB7, p.DMA1_CH5)).unwrap();
    spawner.spawn(blinker(p.PB14)).unwrap();
    spawner.spawn(display(matrix)).unwrap();
}

#[embassy_executor::task]
async fn blinker(pb14: PB14) {
    let mut green_led = Output::new(pb14, Level::Low, Speed::VeryHigh);
    loop {
        for _ in 0..3 {
            green_led.set_high();
            Timer::after_millis(100).await;
            green_led.set_low();
            Timer::after_millis(100).await;
        }
        Timer::after_millis(3000).await;
    }
}

#[embassy_executor::task]
async fn display(mut matrix: Matrix<'static>) {
    let mut ticker = Ticker::every(Duration::from_hz(640));
    loop {
        let mut image = NEXT_IMAGE.wait().now_or_never();
        if image.is_none() {
            image = NEXT_IMAGE.await;
        }
        matrix.display_image(&image, &mut ticker).await;
        ticker.next().await;
        row = (row + 1) % 8;
    }
}

#[embassy_executor::task]
async fn change_image() {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let colors = [Color::RED, Color::BLUE, Color::GREEN];
    let mut color_cycle = colors.iter().cycle();
    loop {
        let color = color_cycle.next().unwrap();
        let mut image = IMAGE.lock().await;
        *image = Image::new_solid(*color);
        drop(image);
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn serial_receiver(usart1: USART1, pb6: PB6, pb7: PB7, dma1_ch5: DMA1_CH5) {
    let mut config = usart::Config::default();
    config.baudrate = 38400;
    let mut serial = Uart::new(usart1, pb7, pb6, Irqs, NoDma, dma1_ch5, config).unwrap();
    let mut buffer = [0 as u8; 192];
    loop {
        let mut c = 0;
        serial.read(core::slice::from_mut(&mut c)).await.unwrap();
        if c != 0xff {
            continue;
        }
        let mut start = 0;
        'receive: loop {
            serial.read(&mut buffer).await.unwrap();
            for pos in (start..192).rev() {
                if buffer[pos] == 0xff {
                    buffer.rotate_right(pos);
                    start = 192 - (pos + 1);
                    continue 'receive;
                }
            }
            break;
        }
        let mut image = IMAGE.lock().await;
        *image.as_mut() = buffer;
        drop(image);
    }
}

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<USART1>;
});