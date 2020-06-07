//! IEEE 802.15.4 radio

use core::{
    cmp, fmt, mem, ops, ptr, slice,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

use binfmt::derive::binDebug;
use pac::RADIO;
use pool::Box;

use crate::{atomic::Atomic, clock, mem::P, Interrupt0, NotSendOrSync};

/// IEEE 802.15.4 channel
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Channel {
    /// 2405 MHz
    _11 = 5,
    /// 2410 MHz
    _12 = 10,
    /// 2415 MHz
    _13 = 15,
    /// 2420 MHz
    _14 = 20,
    /// 2425 MHz
    _15 = 25,
    /// 2430 MHz
    _16 = 30,
    /// 2435 MHz
    _17 = 35,
    /// 2440 MHz
    _18 = 40,
    /// 2445 MHz
    _19 = 45,
    /// 2450 MHz
    _20 = 50,
    /// 2455 MHz
    _21 = 55,
    /// 2460 MHz
    _22 = 60,
    /// 2465 MHz
    _23 = 65,
    /// 2470 MHz
    _24 = 70,
    /// 2475 MHz
    _25 = 75,
    /// 2480 MHz
    _26 = 80,
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Channel::_11 => "11",
            Channel::_12 => "12",
            Channel::_13 => "13",
            Channel::_14 => "14",
            Channel::_15 => "15",
            Channel::_16 => "16",
            Channel::_17 => "17",
            Channel::_18 => "18",
            Channel::_19 => "19",
            Channel::_20 => "20",
            Channel::_21 => "21",
            Channel::_22 => "22",
            Channel::_23 => "23",
            Channel::_24 => "24",
            Channel::_25 => "25",
            Channel::_26 => "26",
        };
        f.write_str(s)
    }
}

#[tasks::declare]
mod task {
    use core::mem::MaybeUninit;

    use pac::RADIO;
    use pool::Node;

    use crate::{mem::P, Interrupt0};

    use super::{Event, Lock, Packet, RxState, TxState, LOCK, RX_STATE, TX_STATE};

    // NOTE(unsafe) all interrupts are still globally masked (`CPSID I`)
    fn init() {
        #[uninit(unsafe)]
        static mut PACKETS: [MaybeUninit<Node<[u8; P::SIZE as usize]>>; 3] = [
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
        ];

        for packet in PACKETS {
            P::manage(packet)
        }

        // reserve peripherals for HAL use
        pac::RADIO::seal();

        RADIO::borrow_unchecked(|radio| {
            const IEEE802154: u8 = 15;
            radio.MODE.write(|w| w.MODE(IEEE802154));

            // set TX power to its maximum value
            radio.TXPOWER.write(|w| w.TXPOWER(8));

            radio.PCNF0.write(|w| {
                // length = 8 bits (but highest bit is reserved and must be 0)
                w.LFLEN(8)
                    // no S0
                    .S0LEN(0)
                    // no S1
                    .S1LEN(0)
                    // S1 not included in RAM
                    .S1INCL(0)
                    // no code indicator
                    .CILEN(0)
                    // 32-bit zero preamble
                    .PLEN(2)
                    // LENGTH field does NOT contain the CRC
                    .CRCINC(1)
                    // no TERM field
                    .TERMLEN(0)
            });

            radio.PCNF1.write(|w| {
                w.MAXLEN(Packet::CAPACITY + 2 /* CRC */) // payload
                    .STATLEN(0)
                    .BALEN(0)
                    // little endian
                    .ENDIAN(0)
                    .WHITEEN(0)
            });

            // CCA = Carrier sense
            radio.CCACTRL.rmw(|_, w| w.CCAMODE(1));

            // CRC configuration - x**16 + x**12 + x**5 + 1
            radio.CRCCNF.write(|w| w.LEN(2).SKIPADDR(2));
            radio.CRCPOLY.write(|w| w.CRCPOLY(0x11021));
            radio.CRCINIT.write(|w| w.CRCINIT(0));

            // permanent shortcuts
            radio.SHORTS.write(|w| w.CCAIDLE_TXEN(1).TXREADY_START(1));

            unsafe {
                radio
                    .INTENSET
                    .write(|w| w.CCABUSY(1).READY(1).FRAMESTART(1).END(1).PHYEND(1))
            }
        });

        unsafe {
            crate::unmask0(&[Interrupt0::RADIO]);
        }
    }

    fn RADIO() -> Option<()> {
        semidap::trace!("RADIO");

        let event = Event::next()?;
        semidap::debug!("-> Event::{}", event);

        match event {
            Event::READY => {}

            Event::CCABUSY => {
                semidap::info!("TX: channel busy -- releasing the radio");
                RADIO::borrow_unchecked(|radio| radio.INTENCLR.write(|w| w.PHYEND(1)));
                LOCK.store(Lock::Free);
                TX_STATE.store(TxState::Busy)
            }

            Event::FRAMESTART => match LOCK.load() {
                Lock::Free => {
                    semidap::info!("RX: frame detected -- locking the RADIO");
                    LOCK.store(Lock::Rx);
                }

                _ =>
                {
                    #[cfg(debug_assertions)]
                    super::unreachable()
                }
            },

            Event::END => match LOCK.load() {
                Lock::Tx => TX_STATE.store(TxState::TransferEnd),

                Lock::Rx => {
                    #[cfg(debug_assertions)]
                    if RX_STATE.load() != RxState::Started {
                        super::unreachable()
                    }

                    // END & PHYEND are both set by a reception event
                    RADIO::borrow_unchecked(|radio| radio.EVENTS_PHYEND.zero());
                    LOCK.store(Lock::Free);
                    RX_STATE.store(RxState::Done);
                    semidap::info!("RX: received data -- releasing the radio");
                }

                Lock::Free =>
                {
                    #[cfg(debug_assertions)]
                    super::unreachable()
                }
            },

            Event::PHYEND => match LOCK.load() {
                Lock::Tx => {
                    unsafe { super::INTENSET_FRAMESTART() }
                    RADIO::borrow_unchecked(|radio| {
                        radio.SHORTS.rmw(|_, w| w.PHYEND_DISABLE(0));
                        radio.INTENCLR.write(|w| w.PHYEND(1));
                    });

                    LOCK.store(Lock::Free);
                    TX_STATE.store(TxState::Done);

                    semidap::info!("TX: transmission done -- releasing the radio");
                }

                _ =>
                {
                    #[cfg(debug_assertions)]
                    super::unreachable()
                }
            },
        }

        None
    }
}

#[derive(binDebug)]
enum Event {
    CCABUSY,
    END,
    FRAMESTART,
    PHYEND,
    READY,
}

impl Event {
    fn next() -> Option<Self> {
        RADIO::borrow_unchecked(|radio| {
            // NOTE this interrupt is sometimes unmasked so we need to clear the event to prevent a
            // random trigger
            if radio.EVENTS_FRAMESTART.read().bits() != 0 {
                radio.EVENTS_FRAMESTART.zero();
                if radio.INTENSET.read().FRAMESTART() != 0 {
                    return Some(Event::FRAMESTART);
                }
            }

            if radio.EVENTS_READY.read().bits() != 0 {
                radio.EVENTS_READY.zero();
                return Some(Event::READY);
            }

            if radio.EVENTS_CCABUSY.read().bits() != 0 {
                radio.EVENTS_CCABUSY.zero();
                return Some(Event::CCABUSY);
            }

            if radio.EVENTS_END.read().bits() != 0 {
                radio.EVENTS_END.zero();
                return Some(Event::END);
            }

            if radio.EVENTS_PHYEND.read().bits() != 0 {
                radio.EVENTS_PHYEND.zero();
                return Some(Event::PHYEND);
            }

            if cfg!(debug_assertions) {
                unreachable()
            } else {
                None
            }
        })
    }
}

/// Claims the radio interface
pub fn claim(chan: Channel) -> (Tx, Rx) {
    static TAKEN: AtomicBool = AtomicBool::new(false);

    if TAKEN
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        RADIO::borrow_unchecked(|radio| {
            radio.FREQUENCY.write(|w| w.FREQUENCY(chan as u8));
        });

        (
            Tx {
                _not_send_or_sync: NotSendOrSync::new(),
            },
            Rx {
                _not_send_or_sync: NotSendOrSync::new(),
            },
        )
    } else {
        semidap::panic!("`radio` interface has already been claimed")
    }
}

// The state of the `Rx.read` operation
#[derive(Clone, Copy, PartialEq, binDebug)]
#[repr(u8)]
enum RxState {
    Idle = 0,
    Started = 1,
    Done = 2,
    // `Tx.write` takes priority
    Interrupted = 3,
}

derive!(RxState);

#[derive(Clone, Copy, PartialEq, binDebug)]
enum Lock {
    Free = 0,

    /// `Rx.read` is holding the lock on the RADIO
    /// Held from FRAMESTART to PHYEND (see Figure 124)
    Rx = 1,

    /// `Tx.write` is holding the lock on the RADIO
    /// Held from the start of `Tx.write` to PHYEND (see figure 123)
    Tx = 2,
}

derive!(Lock);

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum TxState {
    #[allow(dead_code)]
    Idle = 0,
    TransferStart = 1,
    TransferEnd = 2,
    Done = 3,
    Busy = 4,
}

derive!(TxState);

static LOCK: Atomic<Lock> = Atomic::new();
static RX_STATE: Atomic<RxState> = Atomic::new();
static TX_STATE: Atomic<TxState> = Atomic::new();

/// IEEE 802.15.4 radio (receiver half)
pub struct Rx {
    _not_send_or_sync: NotSendOrSync,
}

impl Rx {
    /// Returns the channel the radio is using
    pub fn channel(&self) -> Channel {
        RADIO::borrow_unchecked(|radio| unsafe {
            mem::transmute(radio.FREQUENCY.read().FREQUENCY())
        })
    }

    /// Reads one radio packet
    pub async fn read(&mut self, packet: &mut Packet) {
        clock::has_stabilized().await;

        let mut retry = true;
        while retry {
            crate::poll_fn(|| {
                match LOCK.load() {
                    // wait for TX lock to be released
                    Lock::Tx => Poll::Pending,

                    Lock::Free => {
                        let state = STATE();

                        match state {
                            // enable RX
                            State::Disabled => {
                                TASKS_RXEN();
                                semidap::info!("RX: enabling RADIO");
                                Poll::Pending
                            }

                            // wait for RX to be up
                            State::RxRu => Poll::Pending,

                            State::RxIdle => {
                                // NOTE on each `retry` we need to set PACKETPTR because `Tx.write`,
                                // the operation that can interrupt this one, also writes to that
                                // register

                                crate::dma_start();
                                SET_PACKETPTR(packet.len_ptr_mut() as u32);

                                RX_STATE.store(RxState::Started);
                                TASKS_START();
                                semidap::info!("RX: ready for data");

                                Poll::Ready(())
                            }

                            _ => {
                                semidap::error!("RX.read({})", state);
                                todo();
                            }
                        }
                    }

                    Lock::Rx => {
                        if cfg!(debug_assertions) {
                            unreachable()
                        } else {
                            Poll::Pending
                        }
                    }
                }
            })
            .await;

            crate::poll_fn(|| {
                match RX_STATE.load() {
                    RxState::Started => Poll::Pending,

                    // retry loop
                    RxState::Interrupted => {
                        RX_STATE.store(RxState::Idle);
                        Poll::Ready(())
                    }

                    // exit loop
                    RxState::Done => {
                        // `packet` handed back to us
                        crate::dma_end();

                        RX_STATE.store(RxState::Idle);
                        retry = false;
                        Poll::Ready(())
                    }

                    RxState::Idle => {
                        if cfg!(debug_assertions) {
                            unreachable()
                        } else {
                            Poll::Pending
                        }
                    }
                }
            })
            .await;
        }
    }
}

/// IEEE 802.15.4 radio (transmitter half)
pub struct Tx {
    _not_send_or_sync: NotSendOrSync,
}

impl Tx {
    /// Returns the channel the radio is using
    pub fn channel(&self) -> Channel {
        RADIO::borrow_unchecked(|radio| unsafe {
            mem::transmute(radio.FREQUENCY.read().FREQUENCY())
        })
    }

    /// Sends the specified radio packet
    ///
    /// This method returns once `packet` can be used again but before the last bit of data has been
    /// transmitted
    pub async fn write(&mut self, packet: &Packet) -> Result<(), ()> {
        clock::has_stabilized().await;

        self.flush().await;

        crate::poll_fn(|| unsafe {
            // NOTE(atomic) because we may need to interrupt an RX task
            crate::atomic0(Interrupt0::RADIO, || {
                let lock = LOCK.load();
                match lock {
                    // wait for RX transfer to finish
                    Lock::Rx => Poll::Pending,

                    Lock::Tx | Lock::Free => {
                        if lock == Lock::Free {
                            // claim the lock
                            LOCK.store(Lock::Tx);

                            semidap::info!("TX: locked the RADIO");

                            SET_PACKETPTR(packet.len_ptr() as u32);

                            INTENCLR_FRAMESTART();
                            RADIO::borrow_unchecked(|radio| {
                                radio.SHORTS.rmw(|_, w| w.PHYEND_DISABLE(1))
                            });

                            let rx_state = RX_STATE.load();

                            // we have interrupted `Rx.read`
                            if rx_state == RxState::Started {
                                RX_STATE.store(RxState::Interrupted);
                                TASKS_STOP();

                                semidap::info!("TX: interrupted Rx.read");

                                // wait until next state transition
                                return Poll::Pending;
                            }
                        }

                        let state = STATE();

                        match state {
                            State::Disabled => {
                                TASKS_RXEN();

                                semidap::info!("TX: enabling RADIO");

                                Poll::Pending
                            }

                            State::RxRu => Poll::Pending,

                            State::RxIdle => {
                                TX_STATE.store(TxState::TransferStart);
                                INTENSET_PHYEND();

                                // TX transfer will start at some point after the CCA
                                crate::dma_start();
                                TASKS_CCASTART();

                                semidap::info!("TX: started CCA");

                                Poll::Ready(())
                            }

                            _ => {
                                semidap::error!("TX.write({})", state);
                                todo()
                            }
                        }
                    }
                }
            })
        })
        .await;

        // wait until END or CCABUSY
        let ok = crate::poll_fn(|| {
            let state = TX_STATE.load();
            if state != TxState::TransferStart {
                Poll::Ready(state != TxState::Busy)
            } else {
                Poll::Pending
            }
        })
        .await;

        if ok {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Waits until any pending write has completed
    pub async fn flush(&mut self) {
        crate::poll_fn(|| {
            if TX_STATE.load() != TxState::TransferEnd {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }
}

/// Radio packet
pub struct Packet {
    buffer: Box<P>,
}

impl Packet {
    /// How much data this packet can hold
    pub const CAPACITY: u8 = 127;
    const PADDING: usize = 3;

    /// Returns an empty IEEE 802.15.4 packet
    pub async fn new() -> Self {
        let buffer = P::alloc().await;
        let mut packet = Packet { buffer };
        unsafe { packet.len_ptr_mut().write(2) }
        packet
    }

    /// Fills the packet with given `src` data
    ///
    /// NOTE `src` data will be truncated to `Self::CAPACITY` bytes
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        let len = cmp::min(src.len(), Self::CAPACITY as usize) as u8;
        unsafe {
            self.len_ptr_mut().write(len + 2 /* CRC */);
            ptr::copy_nonoverlapping(src.as_ptr(), self.data_ptr_mut(), len.into())
        }
    }

    /// Returns the size of this packet
    pub fn len(&self) -> u8 {
        unsafe {
            self.len_ptr().read() - 2 /* CRC */
        }
    }

    /// Changes the `len` of the packet
    ///
    /// NOTE `len` will be truncated to `Self::CAPACITY` bytes
    pub fn set_len(&mut self, len: u8) {
        let len = cmp::min(len, Self::CAPACITY);
        unsafe {
            self.len_ptr_mut().write(len + 2 /* CRC */)
        }
    }

    /// Returns the LQI (Link Quality Indicator) of the received packet
    pub fn lqi(&self) -> u8 {
        unsafe { *self.data_ptr().add(self.len().into()) }
    }

    #[cfg(feature = "usb")]
    #[cfg(TODO)]
    pub(crate) fn from_parts(buffer: Box<P>, len: u8) -> Self {
        let mut packet = Packet { buffer };
        unsafe {
            packet.len_ptr_mut().write(len);
        }
        packet
    }

    fn len_ptr(&self) -> *const u8 {
        unsafe { self.buffer.as_ptr().add(Self::PADDING) }
    }

    fn len_ptr_mut(&mut self) -> *mut u8 {
        unsafe { self.buffer.as_mut_ptr().add(Self::PADDING) }
    }

    fn data_ptr(&self) -> *const u8 {
        unsafe {
            self.len_ptr().add(1 /* PHY_HDR */)
        }
    }

    fn data_ptr_mut(&mut self) -> *mut u8 {
        unsafe {
            self.len_ptr_mut().add(1 /* PHY_HDR */)
        }
    }
}

impl ops::Deref for Packet {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data_ptr(), self.len().into()) }
    }
}

impl ops::DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data_ptr_mut(), self.len().into()) }
    }
}

#[cfg(TODO)]
#[cfg(feature = "usb")]
impl crate::usbd::Packet {
    pub fn try_from(packet: Packet) -> Result<crate::usbd::Packet, Packet> {
        let len = packet.len();
        if len <= crate::usbd::Packet::CAPACITY {
            Ok(unsafe { crate::usbd::Packet::from_parts(packet.buffer, len) })
        } else {
            Err(packet)
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, binDebug)]
#[repr(u8)]
enum State {
    Disabled = 0,
    RxRu = 1,
    RxIdle = 2,
    Rx = 3,
    RxDisable = 4,
    TxRu = 9,
    TxIdle = 10,
    Tx = 11,
    TxDisable = 12,
}

// NOTE(borrow_unchecked) all these are either single instruction reads w/o side effects or single
// instruction writes to registers won't be RMW-ed
#[allow(non_snake_case)]
fn TASKS_CCASTART() {
    RADIO::borrow_unchecked(|radio| radio.TASKS_CCASTART.write(|w| w.TASKS_CCASTART(1)))
}

#[allow(non_snake_case)]
fn TASKS_RXEN() {
    RADIO::borrow_unchecked(|radio| radio.TASKS_RXEN.write(|w| w.TASKS_RXEN(1)))
}

#[allow(non_snake_case)]
fn TASKS_START() {
    RADIO::borrow_unchecked(|radio| radio.TASKS_START.write(|w| w.TASKS_START(1)))
}

#[allow(non_snake_case)]
fn TASKS_STOP() {
    RADIO::borrow_unchecked(|radio| radio.TASKS_STOP.write(|w| w.TASKS_STOP(1)))
}

#[allow(non_snake_case)]
fn SET_PACKETPTR(ptr: u32) {
    RADIO::borrow_unchecked(|radio| radio.PACKETPTR.write(|w| w.PACKETPTR(ptr)));
}

#[allow(non_snake_case)]
unsafe fn INTENSET_FRAMESTART() {
    RADIO::borrow_unchecked(|radio| radio.INTENSET.write(|w| w.FRAMESTART(1)));
}

#[allow(non_snake_case)]
unsafe fn INTENSET_PHYEND() {
    RADIO::borrow_unchecked(|radio| radio.INTENSET.write(|w| w.PHYEND(1)));
}

#[allow(non_snake_case)]
fn INTENCLR_FRAMESTART() {
    RADIO::borrow_unchecked(|radio| radio.INTENCLR.write(|w| w.FRAMESTART(1)));
}

#[allow(non_snake_case)]
fn STATE() -> State {
    RADIO::borrow_unchecked(|radio| {
        let bits = radio.STATE.read().bits();
        let state = unsafe { mem::transmute(bits) };
        semidap::debug!("State::{}", state);
        state
    })
}

fn todo() -> ! {
    semidap::panic!("unimplemented")
}

fn unreachable() -> ! {
    semidap::panic!("unreachable")
}
