use esp32c3_hal::prelude::interrupt;
use esp32c3_hal::{
    gpio::{Event, Gpio9, Input, Pin, PullDown, IO},
    interrupt,
    peripherals::Interrupt,
};

use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin as OtherPin;
use core::task::{Context, Poll, Waker};
use critical_section::Mutex;

pub struct EmbGPIO<F> {
    //hal_gpio : Gpio9<Input<PullDown>>,
    toggled_future: F,
}

#[derive(Clone)]
pub struct ToggledFuture {}

impl ToggledFuture {
    pub fn new() -> Self {
        ToggledFuture {}
    }
}

//impl Unpin for ToggledFuture {}

impl Copy for ToggledFuture {}

static recently_toggled: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));
static BUTTON: Mutex<RefCell<Option<Gpio9<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static waker: Mutex<RefCell<Option<Waker>>> = Mutex::new(RefCell::new(None));

impl Future for ToggledFuture {
    type Output = ();
    fn poll(
        mut self: core::pin::Pin<&mut ToggledFuture>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let mut outcome = Poll::Pending;

        critical_section::with(|cs| {
            let mut recently_toggled_ref = recently_toggled.borrow_ref_mut(cs);
            if *recently_toggled_ref == true {
                *recently_toggled_ref = false;
                outcome = Poll::Ready(());
            }

            waker.borrow_ref_mut(cs).replace(cx.waker().clone());
        });

        return outcome;
    }
}

impl EmbGPIO<ToggledFuture> {
    pub fn new<T>(gpio: Gpio9<T>) -> Self {
        let mut button = gpio.into_pull_down_input();
        button.listen(Event::FallingEdge);

        critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));

        interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

        EmbGPIO {
            toggled_future: ToggledFuture::new(),
        }
    }

    pub fn toggled(&self) -> impl Future<Output = ()> {
        self.toggled_future
    }
}

#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();

        *recently_toggled.borrow_ref_mut(cs) = true;

        waker.borrow_ref_mut(cs).as_mut().unwrap().wake_by_ref();
    });

    // TODO: call the waker
}
