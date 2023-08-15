use rp_pico::hal;
use crate::module_logic;
use embedded_hal::serial::Write;


pub struct Parser<D: hal::uart::UartDevice, P: hal::uart::ValidUartPinout<D>> {
    current_module_index: Option<u32>,
    fifo: hal::sio::SioFifo,
    write_uart: hal::uart::UartPeripheral<hal::uart::Enabled, D, P>,
}

impl<
    D: hal::uart::UartDevice,
    P: hal::uart::ValidUartPinout<D>
> Parser<D, P> {
    pub fn new(fifo: hal::sio::SioFifo, write_uart: hal::uart::UartPeripheral<hal::uart::Enabled, D, P>) ->Self {
        Self { current_module_index: None, fifo, write_uart }
    }

    fn forward(&mut self, c: u8) -> () {
        while !self.write_uart.uart_is_writable() {};
        self.write_uart.write(c).unwrap();
    }

    pub fn parse_char(&mut self, c: u8) -> () {
        match c {
            0xFF => {
                // reset
                self.fifo.write((0 << 16) + 0xFF);
                self.fifo.write((1 << 16) + 0xFF);
                self.fifo.write((2 << 16) + 0xFF);
                self.fifo.write((3 << 16) + 0xFF);
                self.forward(c)
            }
            0xFE => {
                // reset current target
                if let Some(mod_index) = self.current_module_index {
                    if mod_index < 4 {
                        self.fifo.write_blocking((mod_index << 16) | 0xFF)
                    } else {
                        self.forward(c)
                    }
                }
            }
            0xF0 => {
                // start char
                self.current_module_index = Some(0);
                self.forward(c)
            }
            letter if (letter <= module_logic::LETTER_COUNT) => {
                if let Some(mod_index) = self.current_module_index {
                    if mod_index < 4 {
                        self.fifo
                            .write_blocking((mod_index << 16) | (letter as u32))
                    } else {
                        self.forward(letter);
                    }

                    self.current_module_index = Some(mod_index + 1);
                }
            }
            _ => {}
        }
    }
}
