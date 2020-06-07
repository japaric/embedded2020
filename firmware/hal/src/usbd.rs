//! USB device

use core::{
    cmp, ops, ptr, slice,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

use binfmt::derive::binDebug;
use pac::{
    usbd::{epdatastatus, epinen, epouten, eventcause},
    POWER, USBD,
};
use pool::Box;
use usb2::{cdc::acm, hid, GetDescriptor, Request, StandardRequest};

use crate::{atomic::Atomic, mem::P, Interrupt1, NotSendOrSync};

include!(concat!(env!("OUT_DIR"), "/descs.rs"));

#[derive(Clone, Copy, PartialEq, binDebug)]
#[repr(u8)]
enum Ep2InState {
    Off = 0,
    Idle,
    InUse,
}

derive!(Ep2InState);

static EP2IN_STATE: Atomic<Ep2InState> = Atomic::new();

#[tasks::declare]
mod task {
    use core::mem::MaybeUninit;

    use pac::{CLOCK, USBD};
    use pool::Node;

    use crate::{clock, errata, mem::P, util::Align4, Interrupt0, Interrupt1};

    use super::{
        Ep0State, Ep2InState, EpIn3State, EpOut3State, PowerEvent, PowerState, UsbdEvent,
        EP2IN_STATE, EPIN3_STATE, EPOUT3_STATE, TX_BUF,
    };

    static mut PCSTATE: PowerState = PowerState::Off;

    // NOTE(unsafe) all interrupts are still globally masked (`CPSID I`)
    fn init() {
        static mut PACKETS: [MaybeUninit<Node<[u8; P::SIZE as usize]>>; 3] = [
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
                w.EP0DATADONE(1)
                    .EP0SETUP(1)
                    .EPDATA(1)
                    .USBEVENT(1)
                    .USBRESET(1)
                    .ENDEPIN3(1)
                    .ENDEPOUT0(1)
                    .ENDEPOUT3(1)
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
        static mut USB_STATE: usb2::State = usb2::State::Default;
        static mut EP0_STATE: Ep0State = Ep0State::Idle;
        #[uninit(unsafe)]
        static mut EP2IN_BUF: Align4<[u8; 63]> = Align4([0; 63]);

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
                        usb2::State::Default | usb2::State::Address { .. } => {
                            *USB_STATE = usb2::State::Default;
                        }

                        usb2::State::Configured { .. } => {
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

                    if super::ep0setup(USB_STATE, EP0_STATE, &mut EP2IN_BUF.0).is_err() {
                        super::EP0STALL()
                    }
                }

                UsbdEvent::EP0DATADONE => {
                    match EP0_STATE {
                        Ep0State::Write { leftover } => {
                            semidap::info!("EPIN0: data transmitted");

                            if *leftover != 0 {
                                super::continue_epin0(leftover);
                            } else {
                                *EP0_STATE = Ep0State::Idle;
                            }
                        }

                        // nothing to do here; wait for ENDEPOUT0
                        Ep0State::Read => {
                            semidap::info!("EPIN0: data received");
                        }

                        Ep0State::Idle =>
                        {
                            #[cfg(debug_assertions)]
                            super::unreachable()
                        }
                    }
                }

                UsbdEvent::EPDATA => {
                    let status = super::EPDATASTATUS();

                    if status.EPIN2() != 0 {
                        crate::dma_end();
                        if EP2IN_STATE.load() != Ep2InState::InUse {
                            #[cfg(debug_assertions)]
                            super::unreachable()
                        }

                        unsafe { super::start_epin2(&mut EP2IN_BUF.0) }
                    }

                    if status.EPIN1() != 0 {
                        semidap::info!("EP1IN: data sent");
                    }

                    if status.EPOUT2() != 0 {
                        // discard received data
                        USBD::borrow_unchecked(|usbd| {
                            let n = usbd.SIZE_EPOUT2.read().SIZE();
                            semidap::info!("EP2OUT: received {} bytes (discarded)", n);
                            usbd.SIZE_EPOUT2.write(|w| w.SIZE(0))
                        });
                    }

                    if status.EPOUT3() != 0 {
                        semidap::info!("HID: received data");
                        EPOUT3_STATE.store(EpOut3State::DataReady);
                    }

                    if status.EPIN3() != 0 {
                        semidap::info!("HID: data sent");
                        EPIN3_STATE.store(EpIn3State::Idle);
                    }
                }

                UsbdEvent::ENDEPOUT0 => {
                    crate::dma_end();
                    *EP0_STATE = Ep0State::Idle;
                }

                UsbdEvent::ENDEPIN3 => {
                    semidap::info!("HID: data to send is ready");
                    EPIN3_STATE.store(EpIn3State::TransferEnd);
                }

                UsbdEvent::ENDEPOUT3 => {
                    semidap::info!("HID: received data has been copied");
                    EPOUT3_STATE.store(EpOut3State::Done);
                }

                UsbdEvent::TxWrite => {
                    let n = TX_BUF.read(&mut EP2IN_BUF.0) as u8;

                    semidap::info!("EP2IN: transferring {} bytes", n);

                    USBD::borrow_unchecked(|usbd| {
                        usbd.EPIN2_PTR
                            .write(|w| w.PTR(EP2IN_BUF.0.as_mut_ptr() as u32));
                        usbd.EPIN2_MAXCNT.write(|w| w.MAXCNT(n));
                        crate::dma_start();
                        usbd.TASKS_STARTEPIN2.write(|w| w.TASKS_STARTEPIN(1));
                    });
                    EP2IN_STATE.store(Ep2InState::InUse);
                }
            },
        }

        None
    }
}

fn ep0setup(
    usb_state: &mut usb2::State,
    ep_state: &mut Ep0State,
    ep2in_buf: &mut [u8; 63],
) -> Result<(), ()> {
    let bmrequesttype = BMREQUESTTYPE();
    let brequest = BREQUEST();
    let wvalue = WVALUE();
    let windex = WINDEX();
    let wlength = WLENGTH();

    let req = Request::parse(bmrequesttype, brequest, wvalue, windex, wlength).map_err(|_| {
        semidap::error!(
            "EP0SETUP: unknown request (bmrequesttype={}, brequest={}, wvalue={}, windex={}, wlength={})",
            bmrequesttype,
            brequest,
            wvalue,
            windex,
            wlength
        );
    })?;

    match req {
        Request::Standard(req) => std_req(usb_state, ep_state, req)?,

        Request::Acm(req) => match *usb_state {
            usb2::State::Configured { .. } => acm_req(ep2in_buf, ep_state, req)?,

            _ => {
                semidap::error!("received ACM request but device is not yet Configured");
                return Err(());
            }
        },

        Request::Hid(req) => match *usb_state {
            usb2::State::Configured { .. } => hid_req(req)?,

            _ => {
                semidap::error!("received HID request but device is not yet Configured");
                return Err(());
            }
        },
    }

    Ok(())
}

fn std_req(
    usb_state: &mut usb2::State,
    ep_state: &mut Ep0State,
    req: StandardRequest,
) -> Result<(), ()> {
    match req {
        StandardRequest::GetDescriptor { descriptor, length } => {
            semidap::info!("GET_DESCRIPTOR [{}] ..", length as u8);

            match descriptor {
                GetDescriptor::Device => {
                    semidap::info!("GET_DESCRIPTOR Device");

                    start_epin0(
                        DEVICE_DESC.get(..length.into()).unwrap_or(&DEVICE_DESC),
                        ep_state,
                    );
                }

                GetDescriptor::DeviceQualifier => {
                    semidap::warn!("GET_DESCRIPTOR DeviceQualifier is not supported");
                    return Err(());
                }

                GetDescriptor::Configuration { index } => {
                    semidap::info!("GET_DESCRIPTOR Configuration {}", index);

                    if index == 0 {
                        start_epin0(
                            CONFIG_DESC.get(..length.into()).unwrap_or(&CONFIG_DESC),
                            ep_state,
                        );
                    } else {
                        semidap::error!("out of bounds GET_DESCRIPTOR Configuration request");
                        return Err(());
                    }
                }

                GetDescriptor::String { .. } => {
                    semidap::error!("requested string descriptor doesn't exist");
                    return Err(());
                }

                _ => {
                    semidap::error!("unsupported GET_DESCRIPTOR");
                    todo();
                }
            }
        }

        StandardRequest::SetAddress {
            address: new_address,
        } => {
            // nothing to do here; the hardware will complete the transaction
            semidap::info!(
                "SET_ADDRESS {}",
                new_address.map(|nz| nz.get()).unwrap_or(0)
            );

            match *usb_state {
                usb2::State::Default => {
                    if let Some(address) = new_address {
                        // move to the Address state
                        *usb_state = usb2::State::Address(address);

                        semidap::info!("moving to the Address state");
                    } else {
                        // stay in the Default state
                    }
                }

                usb2::State::Address(curr_address) => {
                    if let Some(new_address) = new_address {
                        if new_address != curr_address {
                            *usb_state = usb2::State::Address(new_address);

                            semidap::info!("changing host assigned address");
                        }
                    } else {
                        *usb_state = usb2::State::Default;

                        semidap::info!("returning to the Default state");
                    }
                }

                usb2::State::Configured { .. } => {
                    semidap::error!("invalid request in the Configured state");
                    return Err(());
                }
            }

            // nothing else to do here; the hardware will complete the transaction
        }

        StandardRequest::SetConfiguration { value } => {
            semidap::info!(
                "SET_CONFIGURATION {}",
                value.map(|nz| nz.get()).unwrap_or(0)
            );

            match *usb_state {
                usb2::State::Default => {
                    semidap::error!("invalid request in the Default state");
                    return Err(());
                }

                usb2::State::Address(address) => {
                    if let Some(value) = value {
                        if value == CONFIG_VAL {
                            semidap::info!("moving to the Configured state");
                            *usb_state = usb2::State::Configured { address, value };

                            USBD::borrow_unchecked(|usbd| {
                                usbd.EPINEN.write(|w| w.IN0(1).IN1(1).IN2(1).IN3(1));
                                // TODO? Rx support -- host sends back junk when Tx is used though
                                usbd.EPOUTEN.write(|w| w.OUT0(1).OUT3(1));

                                EPIN3_STATE.store(EpIn3State::Idle);

                                // start accepting data on EPOUT3
                                usbd.SIZE_EPOUT3.write(|w| w.SIZE(0));

                                // send a SerialState notification
                                start_epin1(&SERIAL_STATE.0);
                            })
                        } else {
                            semidap::error!("requested configuration is not supported");
                            return Err(());
                        }
                    } else {
                        // stay in the Address state
                    }
                }

                usb2::State::Configured {
                    address,
                    value: curr_value,
                } => {
                    if let Some(new_value) = value {
                        if new_value == curr_value {
                            // no change
                        } else {
                            // other configurations are not supported
                            semidap::error!("requested configuration is not supported");
                            return Err(());
                        }
                    } else {
                        // TODO disable endpoints and transfers
                        semidap::info!("returning to the Address state");
                        *usb_state = usb2::State::Address(address);
                    }
                }
            }

            // issue a status stage to acknowledge the request
            ep0status()
        }

        _ => {
            semidap::error!("unexpected standard request");
            return Err(());
        }
    }

    Ok(())
}

fn acm_req(ep2in_buf: &mut [u8; 63], ep_state: &mut Ep0State, req: acm::Request) -> Result<(), ()> {
    if req.interface != CDC_IFACE {
        semidap::error!("ACM request sent to the wrong interface");
        return Err(());
    }

    match req.kind {
        acm::Kind::GetLineCoding => {
            semidap::info!("ACM: GET_LINE_CODING");

            start_epin0(unsafe { &LINE_CODING }, ep_state);
        }

        acm::Kind::SetLineCoding => {
            semidap::info!("ACM: SET_LINE_CODING");

            if *ep_state != Ep0State::Idle {
                #[cfg(debug_assertions)]
                unreachable()
            }

            *ep_state = Ep0State::Read;

            semidap::info!("EP0OUT: accepting host data");

            // accept data into `LINE_CODING` buffer
            USBD::borrow_unchecked(|usbd| {
                unsafe {
                    usbd.EPOUT0_PTR
                        .write(|w| w.PTR(LINE_CODING.as_mut_ptr() as u32));
                    usbd.EPOUT0_MAXCNT
                        .write(|w| w.MAXCNT(LINE_CODING.len() as u8));
                }
                usbd.SHORTS.rmw(|_, w| w.EP0DATADONE_STARTEPOUT0(1));
                crate::dma_start();
                usbd.TASKS_EP0RCVOUT.write(|w| w.TASKS_EP0RCVOUT(1))
            });
        }

        acm::Kind::SetControlLineState { rts, dtr } => {
            semidap::info!(
                "ACM: SET_CONTROL_LINE_STATE rts={} dtr={}",
                rts as u8,
                dtr as u8
            );

            let state = EP2IN_STATE.load();

            if dtr {
                if state == Ep2InState::Off {
                    unsafe { start_epin2(ep2in_buf) }
                } else {
                    // ignore
                }
            } else {
                // FIXME should cancel on-going transfers
                EP2IN_STATE.store(Ep2InState::Off);
            }

            // issue a status stage to acknowledge the request
            semidap::info!("ACM request acknowledged");
            ep0status()
        }
    }

    Ok(())
}

fn hid_req(req: hid::Request) -> Result<(), ()> {
    if req.interface != HID_IFACE {
        semidap::error!("HID request sent to the wrong interface");
        return Err(());
    }

    match req.kind {
        hid::Kind::SetIdle {
            duration,
            report_id,
        } => {
            semidap::info!(
                "HID: SET_IDLE dur={} report={}",
                duration.map(|nz| nz.get()).unwrap_or(0),
                report_id.map(|nz| nz.get()).unwrap_or(0),
            );

            ep0status()
        }

        hid::Kind::GetDescriptor { descriptor, length } => match descriptor {
            hid::GetDescriptor::Report { index } => {
                semidap::info!("HID: GET_DESCRIPTOR REPORT {} [{}]", index, length);

                // FIXME we should return a valid HID report descriptor here but this seems enough
                // to use `hidapi` with this device on Linux at least

                return Err(());
            }
        },
    }

    Ok(())
}

fn start_epin1(buf: &'static [u8]) {
    let n = cmp::min(buf.len(), 64) as u8;
    semidap::info!("EP1IN: sending {} bytes", n);

    USBD::borrow_unchecked(|usbd| {
        usbd.EPIN1_PTR.write(|w| w.PTR(buf.as_ptr() as u32));
        usbd.EPIN1_MAXCNT.write(|w| w.MAXCNT(n));
        crate::dma_start();
        usbd.TASKS_STARTEPIN1.write(|w| w.TASKS_STARTEPIN(1));
    });
}

/// # Safety
/// This hands `buf` to the DMA. Caller must manually enforce that aliasing rules are respected
unsafe fn start_epin2(buf: &mut [u8; 63]) {
    let n = TX_BUF.read(buf) as u8;
    if n != 0 {
        semidap::info!("EP2IN: sending {} bytes", n);
        USBD::borrow_unchecked(|usbd| {
            usbd.EPIN2_PTR.write(|w| w.PTR(buf.as_ptr() as u32));
            usbd.EPIN2_MAXCNT.write(|w| w.MAXCNT(n));
            crate::dma_start();
            usbd.TASKS_STARTEPIN2.write(|w| w.TASKS_STARTEPIN(1));
        });
        EP2IN_STATE.store(Ep2InState::InUse);
    } else {
        EP2IN_STATE.store(Ep2InState::Idle);
    }
}

fn start_epin0(bytes: &'static [u8], ep_state: &mut Ep0State) {
    #[cfg(debug_assertions)]
    semidap::assert!(
        *ep_state == Ep0State::Idle,
        "tried to start a control read transfer before the previous one finished"
    );

    let len = bytes.len() as u16;

    let maxcnt = if len <= MAX_PACKET_SIZE0.into() {
        // done in a single transfer
        short_ep0datadone_ep0status();
        *ep_state = Ep0State::Write { leftover: 0 };
        len as u8
    } else {
        unshort_ep0datadone_ep0status();
        let maxcnt = MAX_PACKET_SIZE0;
        *ep_state = Ep0State::Write {
            leftover: len - u16::from(maxcnt),
        };
        maxcnt
    };

    semidap::info!("EPIN0: sending {}B of data", maxcnt);

    USBD::borrow_unchecked(|usbd| {
        usbd.EPIN0_MAXCNT.write(|w| w.MAXCNT(maxcnt));
        usbd.EPIN0_PTR.write(|w| w.PTR(bytes.as_ptr() as u32));

        usbd.TASKS_STARTEPIN0.write(|w| w.TASKS_STARTEPIN(1));
    })
}

fn continue_epin0(leftover: &mut u16) {
    USBD::borrow_unchecked(|usbd| {
        usbd.EPIN0_PTR
            .rmw(|r, w| w.PTR(r.PTR() + u32::from(MAX_PACKET_SIZE0)));

        let max_packet_size0 = u16::from(MAX_PACKET_SIZE0);
        if *leftover <= max_packet_size0 {
            let maxcnt = *leftover as u8;
            semidap::info!("EPIN0: sending last {}B of data", maxcnt);
            short_ep0datadone_ep0status();
            usbd.EPIN0_MAXCNT.write(|w| w.MAXCNT(maxcnt));
            *leftover = 0;
        } else {
            semidap::info!("EPIN0: sending next {}B of data", MAX_PACKET_SIZE0);
            *leftover -= max_packet_size0;
        }

        usbd.TASKS_STARTEPIN0.write(|w| w.TASKS_STARTEPIN(1));
    })
}

/// CDC ACM transmit (device to host) endpoint
pub struct Tx {
    _not_send_or_sync: NotSendOrSync,
}

/// Claims the USB CDC ACM interface
pub fn serial() -> Tx {
    static ONCE: AtomicBool = AtomicBool::new(false);

    if ONCE
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        Tx {
            _not_send_or_sync: NotSendOrSync::new(),
        }
    } else {
        semidap::panic!("`usbd::serial` interface has already been claimed")
    }
}

static TX_BUF: ring::Buffer = unsafe {
    ring::Buffer::new({
        #[link_section = ".uninit.TX_BUF"]
        static TX_BUF: [u8; 256] = [0; 256];
        &TX_BUF
    })
};

impl Tx {
    /// Sends data to the host
    pub fn write(&mut self, bytes: &[u8]) {
        // FIXME this should use `write_all`
        TX_BUF.write(bytes);
        crate::pend1(Interrupt1::USBD);
    }
}

/// HID OUT (host to device) endpoint
pub struct HidOut {
    _not_send_or_sync: NotSendOrSync,
}

derive!(EpOut3State);

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum EpOut3State {
    #[allow(dead_code)]
    Idle = 0,
    DataReady = 1,
    Done = 2,
}

static EPOUT3_STATE: Atomic<EpOut3State> = Atomic::new();

impl HidOut {
    /// Receives a HID packet
    pub async fn read(&mut self) -> Packet {
        // wait until the endpoint has received data
        crate::poll_fn(|| {
            if EPOUT3_STATE.load() == EpOut3State::DataReady {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        let mut packet = Packet::new().await;

        // move data from USBD to `packet`
        packet.len = USBD::borrow_unchecked(|usbd| {
            let size = usbd.SIZE_EPOUT3.read().SIZE();
            usbd.EPOUT3_PTR
                .write(|w| w.PTR(packet.data_ptr_mut() as u32));
            usbd.EPOUT3_MAXCNT.write(|w| w.MAXCNT(Packet::CAPACITY + 1));

            // omitted because no memory operation is performed on `packet`
            // crate::dma_start();
            usbd.TASKS_STARTEPOUT3.write(|w| w.TASKS_STARTEPOUT(1));
            size
        });

        // wait until transfer is done
        crate::poll_fn(|| {
            if EPOUT3_STATE.load() == EpOut3State::Done {
                crate::dma_end();
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        packet
    }
}

/// HID IN (device to host) endpoint
pub struct HidIn {
    _not_send_or_sync: NotSendOrSync,
}

derive!(EpIn3State);

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum EpIn3State {
    Off = 0,
    Idle = 1,
    TransferStart = 2,
    TransferEnd = 3,
}

static EPIN3_STATE: Atomic<EpIn3State> = Atomic::new();

impl HidIn {
    /// Sends a HID packet
    ///
    /// Note that this returns after `packet` can be used but before the data has been put "on the
    /// wire"
    pub async fn write(&mut self, packet: &Packet) {
        // wait until the endpoint has been enabled
        crate::poll_fn(|| {
            if EPIN3_STATE.load() == EpIn3State::Off {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;

        self.flush().await;

        USBD::borrow_unchecked(|usbd| {
            usbd.EPIN3_PTR.write(|w| w.PTR(packet.as_ptr() as u32));
            usbd.EPIN3_MAXCNT.write(|w| w.MAXCNT(packet.len()));

            EPIN3_STATE.store(EpIn3State::TransferStart);
            crate::dma_start();
            usbd.TASKS_STARTEPIN3.write(|w| w.TASKS_STARTEPIN(1));
        });

        // wait until data has been transferred
        crate::poll_fn(|| {
            let state = EPIN3_STATE.load();
            if state == EpIn3State::TransferEnd || state == EpIn3State::Idle {
                crate::dma_end();
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    /// Waits until the any pending write completes
    pub async fn flush(&mut self) {
        crate::poll_fn(|| {
            let state = EPIN3_STATE.load();
            if state != EpIn3State::TransferEnd {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }
}

/// HID packet
pub struct Packet {
    buffer: Box<P>,
    len: u8,
}

impl Packet {
    const PADDING: usize = 4;

    /// How much data this packet can hold
    pub const CAPACITY: u8 = 64;

    /// Returns a new, empty HID packet with report ID set to 0
    pub async fn new() -> Self {
        Packet {
            buffer: P::alloc().await,
            len: 0,
        }
    }

    /// Returns the length of the packet
    pub fn len(&self) -> u8 {
        self.len
    }

    /// Fills the packet with given `src` data
    ///
    /// NOTE `src` data will be truncated to `Self::CAPACITY` bytes
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        let len = cmp::min(src.len(), Self::CAPACITY as usize) as u8;
        unsafe { ptr::copy_nonoverlapping(src.as_ptr(), self.data_ptr_mut(), len.into()) }
        self.len = len;
    }

    /// Changes the `len` of the packet
    ///
    /// NOTE `len` will be truncated to `Self::CAPACITY` bytes
    pub fn set_len(&mut self, len: u8) {
        self.len = cmp::min(len, Self::CAPACITY);
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

/// Claims the USB HID interface
pub fn hid() -> (HidOut, HidIn) {
    static ONCE: AtomicBool = AtomicBool::new(false);

    if ONCE
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        (
            HidOut {
                _not_send_or_sync: NotSendOrSync::new(),
            },
            HidIn {
                _not_send_or_sync: NotSendOrSync::new(),
            },
        )
    } else {
        semidap::panic!("`usbd::hid` interface has already been claimed")
    }
}

/// USB packet
#[cfg(TODO)]
pub struct Packet {
    buffer: Box<P>,
    len: u8,
}

#[cfg(TODO)]
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

#[cfg(TODO)]
impl ops::Deref for Packet {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data_ptr(), self.len.into()) }
    }
}

#[cfg(TODO)]
impl ops::DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data_ptr_mut(), self.len.into()) }
    }
}

#[cfg(TODO)]
#[cfg(feature = "radio")]
impl From<Packet> for crate::radio::Packet {
    fn from(packet: Packet) -> crate::radio::Packet {
        crate::radio::Packet::from_parts(packet.buffer, packet.len)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Ep0State {
    Idle,
    Read,
    Write { leftover: u16 },
}

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
    ENDEPOUT0,
    ENDEPOUT3,
    ENDEPIN3,
    EP0DATADONE,
    EP0SETUP,
    EPDATA,
    TxWrite,
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

            if usbd.EVENTS_EP0DATADONE.read().bits() != 0 {
                usbd.EVENTS_EP0DATADONE.zero();
                return Some(UsbdEvent::EP0DATADONE);
            }

            if usbd.EVENTS_EP0SETUP.read().bits() != 0 {
                usbd.EVENTS_EP0SETUP.zero();
                return Some(UsbdEvent::EP0SETUP);
            }

            if usbd.EVENTS_EPDATA.read().bits() != 0 {
                usbd.EVENTS_EPDATA.zero();
                return Some(UsbdEvent::EPDATA);
            }

            if usbd.EVENTS_ENDEPOUT0.read().bits() != 0 {
                usbd.EVENTS_ENDEPOUT0.zero();
                return Some(UsbdEvent::ENDEPOUT0);
            }

            if usbd.EVENTS_ENDEPOUT3.read().bits() != 0 {
                usbd.EVENTS_ENDEPOUT3.zero();
                return Some(UsbdEvent::ENDEPOUT3);
            }

            if usbd.EVENTS_ENDEPIN3.read().bits() != 0 {
                usbd.EVENTS_ENDEPIN3.zero();
                return Some(UsbdEvent::ENDEPIN3);
            }

            if EP2IN_STATE.load() == Ep2InState::Idle && TX_BUF.bytes_to_read() != 0 {
                return Some(UsbdEvent::TxWrite);
            }

            None
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

fn short_ep0datadone_ep0status() {
    USBD::borrow_unchecked(|usbd| {
        usbd.SHORTS.rmw(|_, w| w.EP0DATADONE_EP0STATUS(1));
    });
}

fn unshort_ep0datadone_ep0status() {
    USBD::borrow_unchecked(|usbd| {
        usbd.SHORTS.rmw(|_, w| w.EP0DATADONE_EP0STATUS(0));
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
// instruction writes to registers that won't be RMW-ed
fn connect() {
    USBD::borrow_unchecked(|usbd| usbd.USBPULLUP.write(|w| w.CONNECT(1)));
    semidap::info!("pulled D+ up");
}

// simulate a disconnect so the host doesn't retry enumeration while the device is halted
fn disconnect() {
    USBD::borrow_unchecked(|usbd| usbd.USBPULLUP.zero());
    semidap::info!("detached from the bus");
}

#[allow(dead_code)]
#[allow(non_snake_case)]
fn SIZE_EPOUT1() -> u8 {
    USBD::borrow_unchecked(|usbd| usbd.SIZE_EPOUT1.read().bits())
}

#[allow(dead_code)]
#[allow(non_snake_case)]
fn EPINEN() -> epinen::R {
    USBD::borrow_unchecked(|usbd| usbd.EPINEN.read())
}

#[allow(dead_code)]
#[allow(non_snake_case)]
fn EPIN1_PTR() -> u32 {
    USBD::borrow_unchecked(|usbd| usbd.EPIN1_PTR.read().bits())
}

#[allow(dead_code)]
#[allow(non_snake_case)]
fn EPOUTEN() -> epouten::R {
    USBD::borrow_unchecked(|usbd| usbd.EPOUTEN.read())
}

#[allow(dead_code)]
#[allow(non_snake_case)]
fn EPOUT1_MAXCNT(cnt: u8) {
    USBD::borrow_unchecked(|usbd| usbd.EPOUT1_MAXCNT.write(|w| w.MAXCNT(cnt)))
}

#[allow(dead_code)]
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
    USBD::borrow_unchecked(|usbd| usbd.BMREQUESTTYPE.read().bits())
}

#[allow(non_snake_case)]
fn BREQUEST() -> u8 {
    USBD::borrow_unchecked(|usbd| usbd.BREQUEST.read().bits())
}

#[allow(non_snake_case)]
fn WVALUE() -> u16 {
    USBD::borrow_unchecked(|usbd| {
        u16::from(usbd.WVALUEL.read().bits()) | (u16::from(usbd.WVALUEH.read().bits()) << 8)
    })
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

fn ep0status() {
    USBD::borrow_unchecked(|usbd| {
        usbd.TASKS_EP0STATUS.write(|w| w.TASKS_EP0STATUS(1));
    });
}

fn suspend() {
    semidap::info!("entering low power mode");
    USBD::borrow_unchecked(|usbd| usbd.LOWPOWER.write(|w| w.LOWPOWER(1)))
}

fn resume() {
    semidap::info!("leaving low power mode");
    USBD::borrow_unchecked(|usbd| usbd.LOWPOWER.zero())
}
