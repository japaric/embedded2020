//! USB device

use core::{
    cmp,
    convert::TryFrom,
    mem, ops, ptr, slice,
    sync::atomic::{self, AtomicBool, AtomicU8, Ordering},
    task::Poll,
};

use binfmt::derive::binDebug;
use pac::{
    usbd::{epdatastatus, epinen, epouten, eventcause},
    CLOCK, POWER, USBD,
};
use pool::{pool, Box};
use usbd::{
    bRequest, config,
    device::{self, bMaxPacketSize0, bcdUSB},
    ep, iface, DescriptorType, Direction,
};

use crate::{atomic::Atomic, mem::P, Interrupt1, NotSendOrSync};

const NCONFIGS: u8 = 1;

const DEVICE_DESC: device::Desc = device::Desc {
    // Interface Association Descriptor
    bDeviceClass: 239,
    bDeviceSubClass: 2,
    bDeviceProtocol: 1,

    bMaxPacketSize0: bMaxPacketSize0::B64,
    bNumConfigurations: 1,
    bcdDevice: 0x00_00,
    bcdUSB: bcdUSB::V20,
    iManufacturer: 0,
    iProduct: 0,
    iSerialNumber: 0,
    idProduct: consts::PID,
    idVendor: consts::VID,
};

const FULL_CONFIG_SIZE: u8 = config::Desc::SIZE + iface::Desc::SIZE + 2 * ep::Desc::SIZE;

const CONFIG_DESC: config::Desc = config::Desc {
    bConfigurationValue: 1,
    bMaxPower: 50, // 100 mA
    bNumInterfaces: 1,
    bmAttributes: config::bmAttributes {
        remote_wakeup: false,
        self_powered: true,
    },
    iConfiguration: 0,
    wTotalLength: FULL_CONFIG_SIZE as u16,
};

const IFACE_DESC: iface::Desc = iface::Desc {
    bAlternativeSetting: 0,
    bInterfaceClass: 10,
    bInterfaceNumber: 0,
    bInterfaceProtocol: 0,
    bInterfaceSubClass: 0,
    bNumEndpoints: 2,
    iInterface: 0,
};

const EPIN1_DESC: ep::Desc = ep::Desc {
    bEndpointAddress: ep::Address {
        direction: Direction::IN,
        number: 1,
    },
    bInterval: 0,
    bmAttributes: ep::bmAttributes::Bulk,
    wMaxPacketSize: ep::wMaxPacketSize::BulkControl {
        size: Packet::CAPACITY as u16,
    },
};

const EPOUT1_DESC: ep::Desc = ep::Desc {
    bEndpointAddress: ep::Address {
        direction: Direction::OUT,
        number: 1,
    },
    bInterval: 0,
    bmAttributes: ep::bmAttributes::Bulk,
    wMaxPacketSize: ep::wMaxPacketSize::BulkControl {
        size: Packet::CAPACITY as u16,
    },
};

// String descriptors
#[allow(dead_code)]
static STRINGS: &[&str] = &[];
#[allow(dead_code)]
const LANG_ID: u16 = 1033; // en-us

/// Puts together a configuration descriptor and its interface and endpoint
/// descriptors in a single packet
fn full_config() -> [u8; FULL_CONFIG_SIZE as usize] {
    let mut out = [0; FULL_CONFIG_SIZE as usize];

    let mut pos = 0;
    let mut push = |bytes: &[u8]| {
        let len = bytes.len();
        // NOTE(unsafe) avoid (unreachable) panicking branches: the buffer is big enough
        unsafe {
            out.get_unchecked_mut(pos..pos + len).copy_from_slice(bytes);
        }
        pos += len;
    };

    push(&CONFIG_DESC.bytes());
    push(&IFACE_DESC.bytes());
    push(&EPIN1_DESC.bytes());
    push(&EPOUT1_DESC.bytes());

    out
}

static EPIN1_BUSY: AtomicBool = AtomicBool::new(false);
static EPOUT1_STATE: Atomic<EpOut1State> = Atomic::new();
static EPOUT1_SIZE: AtomicU8 = AtomicU8::new(0);

#[tasks::declare]
mod task {
    use core::{mem::MaybeUninit, sync::atomic::Ordering};

    use pac::{CLOCK, USBD};
    use pool::{Box, Node};

    use crate::{clock, errata, mem::P, Interrupt0, Interrupt1};

    use super::{
        Ep0State, EpOut1State, Packet, PowerEvent, PowerState, UsbdEvent, EPIN1_BUSY, EPOUT1_SIZE,
        EPOUT1_STATE,
    };

    static mut PCSTATE: PowerState = PowerState::Off;

    // NOTE(unsafe) all interrupts are still globally masked (`CPSID I`)
    fn init() {
        #[uninit(unsafe)]
        static mut PACKETS: [MaybeUninit<Node<[u8; P::SIZE]>>; 3] = [
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
        ];

        for packet in PACKETS {
            P::manage(packet)
        }

        // reserve peripherals for HAL use
        pac::POWER::seal();
        pac::USBD::seal();

        CLOCK::borrow_unchecked(|clock| unsafe { clock.INTENSET.write(|w| w.HFCLKSTARTED(1)) });
        pac::POWER::borrow_unchecked(|power| unsafe {
            power
                .INTENSET
                .write(|w| w.USBDETECTED(1).USBREMOVED(1).USBPWRRDY(1));
        });
        pac::USBD::borrow_unchecked(|usbd| unsafe {
            usbd.INTENSET.write(|w| {
                w.ENDEPIN0(1)
                    .ENDEPIN1(1)
                    .EP0SETUP(1)
                    .EPDATA(1)
                    .USBEVENT(1)
                    .USBRESET(1)
                    .ENDEPOUT1(1)
            });
        });

        unsafe {
            crate::unmask0(&[Interrupt0::POWER_CLOCK]);
            crate::unmask1(&[Interrupt1::USBD]);
        }
    }

    fn POWER() -> Option<()> {
        semidap::trace!("POWER");

        let event = PowerEvent::next();
        if let Some(event) = event {
            semidap::debug!("-> {}", event);
        }

        match PCSTATE {
            PowerState::Off => {
                if event? != PowerEvent::USBDETECTED {
                    #[cfg(debug_assertions)]
                    super::unreachable()
                }

                // turn on the USB peripheral
                unsafe { errata::e187a() }
                USBD::borrow_unchecked(|usbd| usbd.ENABLE.write(|w| w.ENABLE(1)));

                semidap::info!("enabled the USB peripheral");

                *PCSTATE = PowerState::RampUp {
                    clock: clock::is_stable(),
                    power: false,
                    usb: false,
                };
            }

            PowerState::RampUp { clock, power, usb } => {
                if !*clock && event.is_none() {
                    *clock = true;
                } else if !*power && event? == PowerEvent::USBPWRRDY {
                    *power = true;
                    semidap::info!("USB power supply ready");
                } else {
                    #[cfg(debug_assertions)]
                    super::unreachable()
                }

                if *clock && *power && *usb {
                    *PCSTATE = PowerState::Ready;
                    super::connect();
                }
            }

            // TODO handle powering down the HFXO?
            PowerState::Ready => super::todo(),
        }

        None
    }

    fn USBD() -> Option<()> {
        static mut USB_STATE: usbd::State = usbd::State::Default;
        static mut EP0_STATE: Ep0State = Ep0State::Idle;

        semidap::trace!("USBD");

        let event = UsbdEvent::next()?;

        semidap::debug!("-> {}", event);

        match PCSTATE {
            PowerState::Off =>
            {
                #[cfg(debug_assertions)]
                super::unreachable()
            }

            PowerState::RampUp { clock, power, usb } => {
                if !*usb && event == UsbdEvent::USBEVENT {
                    #[cfg(debug_assertions)]
                    if super::EVENTCAUSE().READY() == 0 {
                        super::unreachable();
                    }

                    *usb = true;
                    semidap::info!("USB controller is ready");

                    if *clock && *power && *usb {
                        *PCSTATE = PowerState::Ready;
                        super::connect();
                    }
                } else {
                    #[cfg(debug_assertions)]
                    super::unreachable()
                }
            }

            PowerState::Ready => match event {
                UsbdEvent::USBEVENT => {
                    let eventcause = super::EVENTCAUSE();

                    if eventcause.SUSPEND() != 0 {
                        super::suspend();
                    } else if eventcause.RESUME() != 0 {
                        super::resume()
                    } else {
                        super::todo()
                    }
                }

                UsbdEvent::USBRESET => {
                    semidap::info!("USB reset");

                    match USB_STATE {
                        usbd::State::Default | usbd::State::Address => {
                            *USB_STATE = usbd::State::Default;
                        }

                        usbd::State::Configured { .. } => {
                            // TODO need to cancel existing transfers?
                            // TODO disable end points
                            super::todo()
                        }
                    }
                }

                UsbdEvent::EP0SETUP => {
                    #[cfg(debug_assertions)]
                    if *EP0_STATE != Ep0State::Idle {
                        super::unreachable()
                    }

                    super::ep0setup(USB_STATE, EP0_STATE);
                }

                UsbdEvent::ENDEPIN0 => {
                    #[cfg(debug_assertions)]
                    if *EP0_STATE != Ep0State::Write {
                        super::unreachable()
                    }

                    // return the buffer to the memory pool
                    unsafe { drop(Box::<P>::from_raw(super::EPIN0_PTR() as *mut _)) }
                    *EP0_STATE = Ep0State::Idle;
                }

                UsbdEvent::ENDEPIN1 => {
                    // return memory to the pool
                    unsafe {
                        drop(Box::<P>::from_raw(
                            (super::EPIN1_PTR() as *mut u8)
                                .offset(-(Packet::PADDING as isize))
                                .cast(),
                        ))
                    }
                    semidap::info!("EPIN1: memory freed");
                }

                UsbdEvent::ENDEPOUT1 => {
                    if EPOUT1_STATE.load() != EpOut1State::TransferInProgress {
                        #[cfg(debug_assertions)]
                        super::unreachable()
                    }

                    super::EPOUT1_STATE.store(EpOut1State::Idle);
                    semidap::info!("EPOUT1: transfer done");
                }

                UsbdEvent::EPDATA => {
                    let epdatastatus = super::EPDATASTATUS();

                    if epdatastatus.EPIN1() != 0 {
                        semidap::info!("EPIN1: transfer done");
                        EPIN1_BUSY.store(false, Ordering::Relaxed);
                    }

                    if epdatastatus.EPOUT1() != 0 {
                        let state = EPOUT1_STATE.load();
                        match state {
                            EpOut1State::Idle => {
                                semidap::info!("EPOUT1: data ready");
                                EPOUT1_STATE.store(EpOut1State::DataReady)
                            }

                            EpOut1State::BufferReady => {
                                EPOUT1_STATE.store(EpOut1State::TransferInProgress);
                                let size = super::SIZE_EPOUT1();
                                EPOUT1_SIZE.store(size, Ordering::Relaxed);
                                super::EPOUT1_MAXCNT(size);
                                super::STARTEPOUT1();
                                semidap::info!("EPOUT1: transfer started ({}B)", size);
                            }

                            EpOut1State::DataReady | EpOut1State::TransferInProgress =>
                            {
                                #[cfg(debug_assertions)]
                                super::unreachable()
                            }
                        }
                    }
                }
            },
        }

        None
    }
}

fn ep0setup(usb_state: &mut usbd::State, ep_state: &mut Ep0State) {
    let bmrequesttype = BMREQUESTTYPE();
    let brequest = BREQUEST();

    match (bmrequesttype, bRequest::from(brequest)) {
        (0b1000_0000, bRequest::GET_DESCRIPTOR) => {
            // control read transfer

            let desc_type = WVALUEH();
            let desc_index = WVALUEL();
            let language_id = WINDEX();
            let wlength = WLENGTH();

            if let Ok(desc_type) = DescriptorType::try_from(desc_type) {
                semidap::info!(
                    "EP0SETUP: GET_DESCRIPTOR {} {} (lang={}, length={})",
                    desc_type,
                    desc_index,
                    language_id,
                    wlength
                );

                match desc_type {
                    DescriptorType::DEVICE if desc_index == 0 && language_id == 0 => {
                        if let Some(buf) = P::try_alloc() {
                            let bytes = DEVICE_DESC.bytes();

                            epin0(&bytes, buf);
                            *ep_state = Ep0State::Write;
                        } else {
                            semidap::warn!("EP0: not enough memory to handle this request");
                            EP0STALL();
                        }
                    }

                    DescriptorType::CONFIGURATION if language_id == 0 => {
                        if let Some(buf) = P::try_alloc() {
                            let full_config_desc;
                            let config_desc;
                            let bytes = if wlength == u16::from(config::Desc::SIZE) {
                                config_desc = CONFIG_DESC.bytes();
                                &config_desc[..]
                            } else {
                                full_config_desc = full_config();
                                &full_config_desc[..]
                            };

                            epin0(&bytes, buf);
                            *ep_state = Ep0State::Write;
                        } else {
                            semidap::warn!("EP0: not enough memory to handle this request");
                            EP0STALL();
                        }
                    }

                    // not supported; we are a full-speed device
                    DescriptorType::DEVICE_QUALIFIER => {
                        semidap::warn!("EP0: full-speed devices do not support this descriptor");
                        EP0STALL()
                    }

                    _ => todo(),
                }
            } else {
                semidap::error!("EP0SETUP: invalid GET_DESCRIPTOR request");
                EP0STALL()
            }
        }

        (0, bRequest::SET_ADDRESS) => {
            #[cfg(debug_assertions)]
            if *usb_state != usbd::State::Default {
                unreachable()
            }

            let addr = WVALUE();
            let windex = WINDEX();
            let wlength = WLENGTH();

            if addr < 128 && windex == 0 && wlength == 0 {
                let addr = addr as u8;
                semidap::info!("EP0SETUP: SET_ADDRESS {}", addr);

                // no need to issue a status stage; the peripheral takes care of that
                *usb_state = usbd::State::Address;
            } else {
                // invalid request
                semidap::error!("EP0SETUP: invalid SET_ADDRESS request");
                EP0STALL()
            }
        }

        (0, bRequest::SET_CONFIGURATION) => {
            let configuration = WVALUEL();
            let wvalueh = WVALUEH();
            let windex = WINDEX();
            let wlength = WLENGTH();

            if wvalueh == 0 && windex == 0 && wlength == 0 && configuration <= NCONFIGS {
                #[cfg(debug_assertions)]
                if *usb_state == usbd::State::Default {
                    unreachable()
                }

                semidap::info!("EP0SETUP: SET_CONFIGURATION {}", configuration);

                if configuration == 0 {
                    *usb_state = usbd::State::Address;

                    // need to cancel ongoing transfers
                    todo()
                } else {
                    *usb_state = usbd::State::Configured { configuration };

                    // enable bulk endpoints
                    USBD::borrow_unchecked(|usbd| {
                        usbd.EPINEN.write(|w| w.IN0(1).IN1(1));
                        usbd.EPOUTEN.write(|w| w.OUT0(1).OUT1(1));
                        usbd.SIZE_EPOUT1.write(|w| w.SIZE(0));

                        // no data transfer; issue a status stage
                        usbd.TASKS_EP0STATUS.write(|w| w.TASKS_EP0STATUS(1));
                    });
                }
            } else {
                // invalid request
                semidap::error!("invalid SET_CONFIGURATION request");
                EP0STALL()
            }
        }

        // TODO we need to handle more standard requests
        _ => todo(),
    }
}

fn epin0(bytes: &[u8], mut buf: Box<P>) {
    let len = bytes.len();

    if len <= DEVICE_DESC.bMaxPacketSize0 as usize {
        // done in a single transfer
        short_ep0done_ep0setup();
    } else {
        todo()
    }

    let len = len as u8;

    semidap::info!("EPIN0: sending {}B of data", len);

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf.as_mut_ptr(), len.into());
    }

    USBD::borrow_unchecked(|usbd| {
        usbd.EPIN0_MAXCNT.write(|w| w.MAXCNT(len));
        // NOTE(fence) the next write transfer ownership of the buffer to the DMA
        atomic::compiler_fence(Ordering::Release);
        usbd.EPIN0_PTR.write(|w| w.PTR(buf.into_raw() as u32));

        usbd.TASKS_STARTEPIN0.write(|w| w.TASKS_STARTEPIN(1));
    })
}

/// Bulk IN endpoint 1
pub struct BulkIn {
    _not_send_or_sync: NotSendOrSync,
}

/// Bulk OUT endpoint 1
pub struct BulkOut {
    _not_send_or_sync: NotSendOrSync,
}

/// Claims the USB interface
pub fn claim() -> (BulkIn, BulkOut) {
    static ONCE: AtomicBool = AtomicBool::new(false);

    if ONCE
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        (
            BulkIn {
                _not_send_or_sync: NotSendOrSync::new(),
            },
            BulkOut {
                _not_send_or_sync: NotSendOrSync::new(),
            },
        )
    } else {
        semidap::panic!("`usbd` interface has already been claimed")
    }
}

impl BulkOut {
    /// Reads a packet from the host
    pub async fn read(&mut self) -> Packet {
        // wait until the endpoint has been enabled
        crate::poll_fn(|| {
            if EPOUTEN().OUT1() != 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        let mut packet = Packet::new().await;

        let mut needs_len = true;
        let epstart = || {
            USBD::borrow_unchecked(|usbd| {
                const NO_DATA: u8 = u8::max_value();
                let mut size = NO_DATA;
                let state = EPOUT1_STATE.load();
                match state {
                    EpOut1State::Idle | EpOut1State::DataReady => {
                        usbd.EPOUT1_PTR
                            .write(|w| w.PTR(packet.data_ptr_mut() as u32));

                        if state == EpOut1State::DataReady {
                            size = SIZE_EPOUT1();
                            EPOUT1_MAXCNT(size);
                            packet.set_len(size);
                            needs_len = false;
                            EPOUT1_STATE.store(EpOut1State::TransferInProgress);
                        } else {
                            semidap::info!("EPOUT1: buffer ready");
                            EPOUT1_STATE.store(EpOut1State::BufferReady);
                        }
                    }

                    EpOut1State::BufferReady | EpOut1State::TransferInProgress =>
                    {
                        #[cfg(debug_assertions)]
                        unreachable()
                    }
                }

                if size != NO_DATA {
                    // NOTE the following operation handles the buffer to the `USBD` task
                    atomic::compiler_fence(Ordering::Release);
                    // start DMA transfer
                    STARTEPOUT1();
                    semidap::info!("EPOUT1: transfer started ({}B)", size);
                }
            })
        };
        unsafe { crate::atomic1(Interrupt1::USBD, epstart) }

        crate::poll_fn(|| {
            match EPOUT1_STATE.load() {
                EpOut1State::Idle | EpOut1State::DataReady => {
                    // NOTE the `USBD` task has handled the buffer back to us
                    atomic::compiler_fence(Ordering::Acquire);
                    Poll::Ready(())
                }

                EpOut1State::BufferReady | EpOut1State::TransferInProgress => Poll::Pending,
            }
        })
        .await;

        if needs_len {
            packet.set_len(EPOUT1_SIZE.load(Ordering::Relaxed));
        }

        packet
    }
}

impl BulkIn {
    /// Sends a packet to the host
    pub async fn write(&mut self, packet: Packet) {
        // wait until the endpoint has been enabled
        crate::poll_fn(|| {
            if EPINEN().IN1() != 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        crate::poll_fn(|| {
            if EPIN1_BUSY.load(Ordering::Relaxed) {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;

        USBD::borrow_unchecked(|usbd| {
            let len = packet.len();

            // NOTE(fence) the next store hands the `packet` to the USBD task
            atomic::compiler_fence(Ordering::Release);
            usbd.EPIN1_PTR.write(|w| w.PTR(packet.data_ptr() as u32));
            mem::forget(packet);
            usbd.EPIN1_MAXCNT.write(|w| w.MAXCNT(len));
            EPIN1_BUSY.store(true, Ordering::Relaxed);

            semidap::info!("EPIN1: transfer started ({}B)", len);

            usbd.TASKS_STARTEPIN1.write(|w| w.TASKS_STARTEPIN(1));
        });
    }
}

/// USB packet
pub struct Packet {
    buffer: Box<P>,
    len: u8,
}

impl Packet {
    /// How much data this packet can hold
    pub const CAPACITY: u8 = 64;

    const PADDING: usize = 4;

    /// Returns an empty USB packet
    pub async fn new() -> Self {
        Self {
            buffer: P::alloc().await,
            len: 0,
        }
    }

    /// Fills the packet with given `src` data
    ///
    /// NOTE `src` data will be truncated to `Self::CAPACITY` bytes
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        let len = cmp::min(src.len(), Packet::CAPACITY as usize);
        unsafe { ptr::copy_nonoverlapping(src.as_ptr(), self.data_ptr_mut(), len) }
        self.len = len as u8;
    }

    /// Returns the size of this packet
    pub fn len(&self) -> u8 {
        self.len
    }

    /// Changes the `len` of the packet
    ///
    /// NOTE `len` will be truncated to `Self::CAPACITY` bytes
    pub fn set_len(&mut self, len: u8) {
        self.len = cmp::min(len, Packet::CAPACITY);
    }

    #[cfg(feature = "radio")]
    pub(crate) unsafe fn from_parts(buffer: Box<P>, len: u8) -> Self {
        Self { buffer, len }
    }

    fn data_ptr(&self) -> *const u8 {
        unsafe { self.buffer.as_ptr().add(Self::PADDING) }
    }

    fn data_ptr_mut(&mut self) -> *mut u8 {
        unsafe { self.buffer.as_mut_ptr().add(Self::PADDING) }
    }
}

impl ops::Deref for Packet {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data_ptr(), self.len.into()) }
    }
}

impl ops::DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data_ptr_mut(), self.len.into()) }
    }
}

impl From<Packet> for crate::radio::Packet {
    fn from(packet: Packet) -> crate::radio::Packet {
        crate::radio::Packet::from_parts(packet.buffer, packet.len)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Ep0State {
    Idle,
    Write,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum EpOut1State {
    Idle = 0,
    DataReady = 1,
    BufferReady = 2,
    TransferInProgress = 3,
}

derive!(EpOut1State);

#[derive(Clone, Copy)]
enum PowerState {
    Off,
    RampUp { clock: bool, power: bool, usb: bool },
    Ready,
}

#[derive(Clone, Copy, PartialEq, binDebug)]
enum PowerEvent {
    USBDETECTED,
    USBREMOVED,
    USBPWRRDY,
}

impl PowerEvent {
    fn next() -> Option<Self> {
        POWER::borrow_unchecked(|power| {
            if power.EVENTS_USBDETECTED.read().bits() != 0 {
                power.EVENTS_USBDETECTED.zero();
                return Some(PowerEvent::USBDETECTED);
            }

            if power.EVENTS_USBREMOVED.read().bits() != 0 {
                power.EVENTS_USBREMOVED.zero();
                return Some(PowerEvent::USBREMOVED);
            }

            if power.EVENTS_USBPWRRDY.read().bits() != 0 {
                power.EVENTS_USBPWRRDY.zero();
                return Some(PowerEvent::USBPWRRDY);
            }

            None
        })
    }
}

#[derive(Clone, Copy, binDebug, PartialEq)]
enum UsbdEvent {
    ENDEPIN0,
    ENDEPIN1,
    ENDEPOUT1,
    EP0SETUP,
    EPDATA,
    USBEVENT,
    USBRESET,
}

impl UsbdEvent {
    fn next() -> Option<Self> {
        USBD::borrow_unchecked(|usbd| {
            if usbd.EVENTS_USBEVENT.read().bits() != 0 {
                usbd.EVENTS_USBEVENT.zero();
                return Some(UsbdEvent::USBEVENT);
            }

            if usbd.EVENTS_USBRESET.read().bits() != 0 {
                usbd.EVENTS_USBRESET.zero();
                return Some(UsbdEvent::USBRESET);
            }

            if usbd.EVENTS_ENDEPIN0.read().bits() != 0 {
                usbd.EVENTS_ENDEPIN0.zero();
                return Some(UsbdEvent::ENDEPIN0);
            }

            if usbd.EVENTS_EP0SETUP.read().bits() != 0 {
                usbd.EVENTS_EP0SETUP.zero();
                return Some(UsbdEvent::EP0SETUP);
            }

            if usbd.EVENTS_ENDEPIN1.read().bits() != 0 {
                usbd.EVENTS_ENDEPIN1.zero();
                return Some(UsbdEvent::ENDEPIN1);
            }

            if usbd.EVENTS_ENDEPOUT1.read().bits() != 0 {
                usbd.EVENTS_ENDEPOUT1.zero();
                return Some(UsbdEvent::ENDEPOUT1);
            }

            if usbd.EVENTS_EPDATA.read().bits() != 0 {
                usbd.EVENTS_EPDATA.zero();
                return Some(UsbdEvent::EPDATA);
            }

            if cfg!(debug_assertions) {
                unreachable()
            } else {
                None
            }
        })
    }
}

fn unreachable() -> ! {
    disconnect();
    semidap::panic!("unreachable")
}

fn todo() -> ! {
    disconnect();
    semidap::panic!("unimplemented")
}

fn short_ep0done_ep0setup() {
    USBD::borrow_unchecked(|usbd| {
        usbd.SHORTS.rmw(|_, w| w.EP0DATADONE_EP0STATUS(1));
    });
}

#[allow(non_snake_case)]
fn EVENTCAUSE() -> eventcause::R {
    USBD::borrow_unchecked(|usbd| {
        let r = usbd.EVENTCAUSE.read();
        usbd.EVENTCAUSE.write(|w| {
            *w = r.into();
            w
        });
        semidap::debug!("{}", r);
        r
    })
}

#[allow(non_snake_case)]
fn EPDATASTATUS() -> epdatastatus::R {
    USBD::borrow_unchecked(|usbd| {
        let r = usbd.EPDATASTATUS.read();
        usbd.EPDATASTATUS.write(|w| {
            *w = r.into();
            w
        });
        r
    })
}

// NOTE(borrow_unchecked) all these are either single instruction reads w/o side effects or single
// instruction writes to registers won't be RMW-ed
fn connect() {
    USBD::borrow_unchecked(|usbd| usbd.USBPULLUP.write(|w| w.CONNECT(1)));
    semidap::info!("pulled D+ up");
}

// simulate a disconnect so the host doesn't retry enumeration while the device is halted
fn disconnect() {
    USBD::borrow_unchecked(|usbd| usbd.USBPULLUP.zero());
    semidap::info!("detached from the bus");
}

#[allow(non_snake_case)]
fn EPIN0_PTR() -> u32 {
    USBD::borrow_unchecked(|usbd| usbd.EPIN0_PTR.read().bits())
}

#[allow(non_snake_case)]
fn SIZE_EPOUT1() -> u8 {
    USBD::borrow_unchecked(|usbd| usbd.SIZE_EPOUT1.read().bits())
}

#[allow(non_snake_case)]
fn EPINEN() -> epinen::R {
    USBD::borrow_unchecked(|usbd| usbd.EPINEN.read())
}

#[allow(non_snake_case)]
fn EPIN1_PTR() -> u32 {
    USBD::borrow_unchecked(|usbd| usbd.EPIN1_PTR.read().bits())
}

#[allow(non_snake_case)]
fn EPOUTEN() -> epouten::R {
    USBD::borrow_unchecked(|usbd| usbd.EPOUTEN.read())
}

#[allow(non_snake_case)]
fn EPOUT1_MAXCNT(cnt: u8) {
    USBD::borrow_unchecked(|usbd| usbd.EPOUT1_MAXCNT.write(|w| w.MAXCNT(cnt)))
}

#[allow(non_snake_case)]
fn EPOUT1_PTR() -> u32 {
    USBD::borrow_unchecked(|usbd| usbd.EPOUT1_PTR.read().bits())
}

#[allow(non_snake_case)]
fn STARTEPOUT1() {
    USBD::borrow_unchecked(|usbd| usbd.TASKS_STARTEPOUT1.write(|w| w.TASKS_STARTEPOUT(1)));
}

#[allow(non_snake_case)]
fn EP0STALL() {
    USBD::borrow_unchecked(|usbd| usbd.TASKS_EP0STALL.write(|w| w.TASKS_EP0STALL(1)));
    semidap::info!("EP0: stalled");
}

#[allow(non_snake_case)]
fn BMREQUESTTYPE() -> u8 {
    let r = USBD::borrow_unchecked(|usbd| usbd.BMREQUESTTYPE.read());
    semidap::debug!("{}", r);
    r.bits()
}

#[allow(non_snake_case)]
fn BREQUEST() -> u8 {
    let r = USBD::borrow_unchecked(|usbd| usbd.BREQUEST.read());
    semidap::debug!("{}", r);
    r.bits()
}

#[allow(non_snake_case)]
fn WVALUE() -> u16 {
    USBD::borrow_unchecked(|usbd| {
        u16::from(usbd.WVALUEL.read().bits()) | (u16::from(usbd.WVALUEH.read().bits()) << 8)
    })
}

#[allow(non_snake_case)]
fn WVALUEH() -> u8 {
    USBD::borrow_unchecked(|usbd| usbd.WVALUEH.read().bits())
}

#[allow(non_snake_case)]
fn WVALUEL() -> u8 {
    USBD::borrow_unchecked(|usbd| usbd.WVALUEL.read().bits())
}

#[allow(non_snake_case)]
fn WINDEX() -> u16 {
    USBD::borrow_unchecked(|usbd| {
        u16::from(usbd.WINDEXL.read().bits()) | (u16::from(usbd.WINDEXH.read().bits()) << 8)
    })
}

#[allow(non_snake_case)]
fn WLENGTH() -> u16 {
    USBD::borrow_unchecked(|usbd| {
        u16::from(usbd.WLENGTHL.read().bits()) | (u16::from(usbd.WLENGTHH.read().bits()) << 8)
    })
}

fn suspend() {
    semidap::info!("entering low power mode");
    USBD::borrow_unchecked(|usbd| usbd.LOWPOWER.write(|w| w.LOWPOWER(1)))
}

fn resume() {
    semidap::info!("leaving low power mode");
    USBD::borrow_unchecked(|usbd| usbd.LOWPOWER.zero())
}
