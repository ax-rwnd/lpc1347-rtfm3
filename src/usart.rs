#![allow(dead_code)]

extern crate lpc1347;
use lpc1347::Interrupt::USART;

/// The length of the USART RX buffer
pub const BUFFER_SIZE: usize = 1024;

/// U(S)ART Protocol Control Block
/// Specifies the details required to RX/TX
// Static UART_PCB: Pcb = Pcb {
//     initialized: false,
//     baud_rate: 0,
//     status: 0,
//     tx_data: 0,
//     rxfifo: UartBuffer {
//         ep_dir: 0,
//         length: 0,
//         rptr: 0,
//         wptr: 0,
//         buffer: [0; BUFFER_SIZE],
//     },
// };

/// Buffer content
pub struct UartBuffer {
    //pub ep_dir: u8,
    /// Length of content, currently
    pub length: usize,
    /// Read pointer
    pub rptr: usize,
    /// Write pointer
    pub wptr: usize,
    /// Data buffer
    pub buffer: [u8; BUFFER_SIZE],
}

/// Control structure
pub struct Pcb {
    //pub initialized: bool,
    /// Baud rate of connection
    pub baud_rate: u32,
    /// Status
    pub status: u32,
    /// Transmission data
    pub tx_data: u32,
    /// RX-buffer
    pub rxfifo: UartBuffer,
}

/// Initialize the USART controller
pub fn init(
    pcb: &mut Pcb,
    nvic: &mut lpc1347::NVIC,
    iocon: &lpc1347::IOCON,
    syscon: &lpc1347::SYSCON,
    usart: &lpc1347::USART,
    baudrate: u32,
    flow_control: bool,
) {
    nvic.disable(USART);
    init_buffer(pcb);

    unsafe {
        // RXD
        iocon.pio0_18.modify(|_, w| w.func().bits(0x1));
        // TXD
        iocon.pio0_19.modify(|_, w| w.func().bits(0x1));
    }

    // Setup flowcontrol (RTS/CTS)
    if flow_control {
        unsafe {
            // CTS
            iocon.pio0_7.modify(|_, w| w.func().bits(0x1));
            // RTS
            iocon.pio0_17.modify(|_, w| w.func().bits(0x1));
        }
    }

    // Start USART clock
    syscon.sysahbclkctrl.modify(|_, w| w.usart().bit(true));
    unsafe {
        syscon.uartclkdiv.modify(|_, w| w.div().bits(0x1));
    }

    // Setup Line Control Register
    usart.lcr.modify(|_, w| w.wls().bits(0x3));
    usart.lcr.modify(|_, w| w.ps().bits(0x0));
    usart.lcr.modify(|_, w| w.sbs().bit(false));
    usart.lcr.modify(|_, w| w.pe().bit(false));
    usart.lcr.modify(|_, w| w.bc().bit(false));
    usart.lcr.modify(|_, w| w.dlab().bit(true));

    // Setup baud rate
    {
        let register_value: u32 = syscon.uartclkdiv.read().div().bits() as u32;
        let fdiv: u32 = ((12000u32 / register_value) / 16u32) / baudrate as u32;
        unsafe {
            usart
                .dlm
                .modify(|_, w| w.dlmsb().bits((fdiv / 256u32) as u8));
            usart
                .dll
                .modify(|_, w| w.dllsb().bits((fdiv % 256u32) as u8));
        }
    }

    // Reset divisor latch access bit
    usart.lcr.modify(|_, w| w.dlab().bit(false));

    // Enable and clear FIFO
    usart.fcr.write(|w| w.fifoen().bit(true));
    usart.fcr.write(|w| w.rxfifores().bit(true));
    usart.fcr.write(|w| w.txfifores().bit(true));

    // Enable auto RTS/CTS
    if flow_control {
        usart.mcr.modify(|_, w| w.rtsen().bit(true));
        usart.mcr.modify(|_, w| w.ctsen().bit(true));
    }

    // Ensure clean start
    while !usart.lsr.read().temt().bit() && !usart.lsr.read().thre().bit() {}
    while usart.lsr.read().rdr().bit() {
        // Dump data
        let _register_value = usart.rbr.read().bits();
    }

    // Turn on USART once config is complete
    nvic.enable(USART);
    usart.ier.modify(|_, w| w.rbrinten().bit(true));
    usart.ier.modify(|_, w| w.rlsinten().bit(true));
}

/// Send an arbitrary region of data over USART
/// It is left as unsafe because of the raw-pointer reference
pub unsafe fn send(usart: &lpc1347::USART, buffer: *mut u8, length: isize) {
    if buffer.is_null() {
        panic!("USART buffer was null");
    }

    let mut pos: isize = 0;
    while pos < length {
        while !usart.lsr.read().thre().bit() {}
        usart.thr.write(|w| w.thr().bits(*buffer.offset(pos)));
        pos += 1;
    }
}

/// Send a single byte over USART
pub fn send_byte(usart: &lpc1347::USART, byte: u8) {
    while !usart.lsr.read().thre().bit() {}
    unsafe {
        usart.thr.write(|w| w.thr().bits(byte));
    }
}

/// Scaffold for interrupt handling
pub fn handle_interrupt(usart: &lpc1347::USART, pcb: &mut Pcb) {
    // Check Receiver Line Status
    let iir = usart.iir.read();
    match iir.intid().bits() {
        0b101 => {
            // Detect errors
            let lsr = usart.lsr.read();
            if lsr.oe().bit() || lsr.pe().bit() || lsr.fe().bit() || lsr.rxfe().bit()
                || lsr.bi().bit()
            {
                pcb.status = lsr.bits();
                let _dummy = usart.rbr.read();
                return;
            }

            if lsr.rdr().bit() {
                // TODO: write to buffer
            }
        }
        0b10 => {
            // TODO: write to buffer
        }
        0b110 => {
            pcb.status |= 0x100;
        }
        0b1 => {
            let lsr = usart.lsr.read();
            if lsr.thre().bit() {
                pcb.tx_data = 0;
            } else {
                pcb.tx_data = 1;
            }
        }
        _ => {}
    }
}

/// Write some data to the protocol buffer
pub fn write_buffer(pcb: &mut Pcb, data: u8) {
    pcb.rxfifo.buffer[pcb.rxfifo.wptr] = data;
    pcb.rxfifo.wptr = (pcb.rxfifo.wptr + 1) % BUFFER_SIZE;
    pcb.rxfifo.length += 1;
}

/// Clear the buffer by resetting the length
pub fn init_buffer(pcb: &mut Pcb) {
    pcb.rxfifo.length = 0;
}

/// Dump the next byte
pub fn read_buffer(pcb: &mut Pcb) -> u8 {
    let data = pcb.rxfifo.buffer[pcb.rxfifo.rptr];
    pcb.rxfifo.rptr = (pcb.rxfifo.rptr + 1) % BUFFER_SIZE;
    pcb.rxfifo.length -= 1;
    data
}

/// Dump the next bytes
pub fn read_array(pcb: &mut Pcb, target: &mut [u8], max_length: usize) -> bool {
    let ptr = 0;
    while pcb.rxfifo.length > 0 && ptr < max_length {
        target[ptr] = read_buffer(pcb);
    }
    ptr > 0
}

/// Empty FIFO and reset length
pub fn clear_fifo(pcb: &mut Pcb) {
    pcb.rxfifo.rptr = 0;
    pcb.rxfifo.wptr = 0;
    pcb.rxfifo.length = 0;
}

/// Check if there is data waiting
pub fn data_pending(pcb: &Pcb) -> bool {
    pcb.rxfifo.length > 0
}
