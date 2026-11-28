#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{
    adc, gpio::{Level, Output}, peripherals::USB, usb::{self, Driver},
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_time::Timer;
use embassy_usb::{Config, UsbDevice};
use messages::{
    DOWNLINK_TOPICS, ENDPOINTS, UPLINK_TOPICS, WakeEndpoint, WakeRequest, WakeResponse,
};
use panic_halt as _;
use postcard_rpc::{
    define_dispatch,
    header::VarHeader,
    server::{
        Dispatch, Server,
        impls::embassy_usb_v0_5::{
            PacketBuffers, USB_FS_MAX_PACKET_SIZE,
            dispatch_impl::{WireRxBuf, WireRxImpl, WireSpawnImpl, WireStorage, WireTxImpl},
        },
    },
};
use static_cell::ConstStaticCell;

embassy_rp::bind_interrupts!(pub struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

struct Context {
    wake: Output<'static>,
}

type AppDriver = usb::Driver<'static, USB>;
type AppStorage = WireStorage<ThreadModeRawMutex, AppDriver, 256, 256, 64, 256>;
type BufStorage = PacketBuffers<1024, 1024>;
type AppTx = WireTxImpl<ThreadModeRawMutex, AppDriver>;
type AppRx = WireRxImpl<AppDriver>;
type AppServer = Server<AppTx, AppRx, WireRxBuf, MyApp>;

static PBUFS: ConstStaticCell<BufStorage> = ConstStaticCell::new(BufStorage::new());
static STORAGE: AppStorage = AppStorage::new();

define_dispatch! {
    app: MyApp;
    spawn_fn: spawn_fn;
    tx_impl: AppTx;
    spawn_impl: WireSpawnImpl;
    context: Context;
    endpoints: {
        list: ENDPOINTS;
        | EndpointTy   | kind  | handler      |
        | ------------ | ----- | ------------ |
        | WakeEndpoint | async | wake_handler |
    };
    topics_in: {
        list: DOWNLINK_TOPICS;
        | TopicTy   | kind        | handler      |
        | --------- | ----------- | ------------ |
    };
    topics_out: {
        list: UPLINK_TOPICS;
    };
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);
    let pbufs = PBUFS.take();
    let config = usb_config();

    let context = Context {
        wake: Output::new(p.PIN_25, Level::High),
    };
    let (device, tx_impl, rx_impl) = STORAGE.init(
        driver,
        config,
        pbufs.tx_buf.as_mut_slice(),
        USB_FS_MAX_PACKET_SIZE,
    );
    let dispatcher = MyApp::new(context, spawner.into());
    let vkk = dispatcher.min_key_len();
    let mut server: AppServer = Server::new(
        tx_impl,
        rx_impl,
        pbufs.rx_buf.as_mut_slice(),
        dispatcher,
        vkk,
    );
    spawner.spawn(usb_task(device).unwrap());

    loop {
        _ = server.run().await;
    }
}

fn usb_config() -> Config<'static> {
    let mut config = Config::new(0xbada, 0x5501);
    config.manufacturer = Some("MoreTec");
    config.product = Some("Table Raiser");
    config.serial_number = Some("TR-0001");

    // Required for windows compatibility.
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    config.max_power = 100;
    config.max_packet_size_0 = 64;

    config
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, AppDriver>) {
    usb.run().await;
}

async fn wake_handler(context: &mut Context, _header: VarHeader, WakeRequest(millis): WakeRequest) -> WakeResponse {
    context.wake.set_high();
    Timer::after_millis(millis).await;
    context.wake.set_low();
    WakeResponse
}
