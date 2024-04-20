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
// use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Ticker, Timer};
use futures::FutureExt;
use heapless::box_pool;
use heapless::pool::boxed::{Box, BoxBlock};
use panic_probe as _;
use tp_led_matrix::{Color, Image, Matrix};

box_pool!(POOL: Image);
static NEXT_IMAGE: Signal<ThreadModeRawMutex, Box<POOL>> = Signal::new();
// static IMAGE: Mutex<ThreadModeRawMutex, Image> = Mutex::new(Image::new_solid(Color::GREEN));

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

    #[allow(clippy::declare_interior_mutable_const)]
    unsafe {
        const BLOCK: BoxBlock<Image> = BoxBlock::new();
        static mut MEMORY: [BoxBlock<Image>; 3] = [BLOCK; 3];
        #[allow(static_mut_refs)]
        for block in &mut MEMORY {
            POOL.manage(block);
        }
    }

    // spawner.spawn(change_image()).unwrap();
    spawner
        .spawn(serial_receiver(p.USART1, p.PB6, p.PB7, p.DMA1_CH5))
        .unwrap();
    spawner.spawn(blinker(p.PB14)).unwrap();
    spawner.spawn(display(matrix)).unwrap();

    if let Ok(initial_image) = POOL.alloc(Image::gradient(Color::BLUE)) {
        NEXT_IMAGE.signal(initial_image);
    }

    /*
    loop {
        let Ok(initial_image) = POOL.alloc(Image::gradient(Color::BLUE)) else {
            defmt::error!("Failed to allocate initial image");
            loop {}
        };
        NEXT_IMAGE.signal(initial_image);
        Timer::after_secs(1).await;

        let Ok(initial_image) = POOL.alloc(Image::gradient(Color::GREEN)) else {
            defmt::error!("Failed to allocate initial image");
            loop {}
        };
        NEXT_IMAGE.signal(initial_image);
        Timer::after_secs(1).await;
    }
    */
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
    let mut image = NEXT_IMAGE.wait().await;
    loop {
        let new_image = NEXT_IMAGE.wait().now_or_never();
        if new_image.is_some() {
            image = new_image.unwrap();
        }
        matrix.display_image(&image, &mut ticker).await;
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn change_image() {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let colors = [Color::RED, Color::BLUE, Color::GREEN];
    let mut color_cycle = colors.iter().cycle();
    loop {
        let color = color_cycle.next().unwrap();
        let Ok(image) = POOL.alloc(Image::gradient(*color)) else {
            defmt::error!("Failed to allocate initial image");
            continue;
        };
        NEXT_IMAGE.signal(image);
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn serial_receiver(usart1: USART1, pb6: PB6, pb7: PB7, dma1_ch5: DMA1_CH5) {
    let mut config = usart::Config::default();
    config.baudrate = 38400;
    let mut serial = Uart::new(usart1, pb7, pb6, Irqs, NoDma, dma1_ch5, config).unwrap();
    // let mut buffer = [0 as u8; 192];

    loop {
        let mut character = [0_u8; 1];
        serial.read(&mut character).await.unwrap();
        if character[0] != 0xff {
            continue;
        }

        let Ok(mut image) = POOL.alloc(Image::default()) else {
            defmt::error!("Failed to allocate initial image");
            continue;
        };

        let mut offset_n = 0;
        'receive: loop {
            serial.read(&mut image.as_mut()[offset_n..]).await.unwrap();
            let pos_k = image.as_ref().iter().rev().position(|&c| c == 0xff);

            if let Some(pos_k) = pos_k {
                image.as_mut().rotate_right(pos_k);
                offset_n = pos_k;
                continue 'receive;
            }
            break;
        }

        // let mut image = IMAGE.lock().await;
        // *image.as_mut() = buffer;
        // drop(image);
        NEXT_IMAGE.signal(image);
    }
}

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<USART1>;
});
