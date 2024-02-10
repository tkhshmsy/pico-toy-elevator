type Result<T = (), E = core::fmt::Error> = core::result::Result<T, E>;

pub struct Voice<UART> {
    uart: UART,
}

impl<UART> Voice<UART>
where
    UART: embedded_hal::serial::Write<u8> + core::fmt::Write,
{
    pub fn new(uart: UART) -> Result<Self> {
        let voice = Voice { uart: uart };
        Ok(voice)
    }

    pub fn talk(&mut self, msg: &str) {
        writeln!(self.uart, "{}", msg).unwrap();
    }
}
