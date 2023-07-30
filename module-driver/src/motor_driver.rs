use embedded_hal::digital::v2::OutputPin;

pub const STEPS_PER_REVOLUTION: u32 = 2038; // 28BYJ-48

pub struct MotorDriver<'a, E> {
    pin_coil_a: &'a mut dyn OutputPin<Error = E>,
    pin_coil_b: &'a mut dyn OutputPin<Error = E>,
    pin_coil_c: &'a mut dyn OutputPin<Error = E>,
    pin_coil_d: &'a mut dyn OutputPin<Error = E>,
    current_index: u32,
    current_step: u8,
}

impl<'a, E> MotorDriver<'a, E> {
    pub fn new(
        pin_coil_a: &'a mut dyn OutputPin<Error = E>,
        pin_coil_b: &'a mut dyn OutputPin<Error = E>,
        pin_coil_c: &'a mut dyn OutputPin<Error = E>,
        pin_coil_d: &'a mut dyn OutputPin<Error = E>,
    ) -> MotorDriver<'a, E> {
        MotorDriver {
            pin_coil_a,
            pin_coil_b,
            pin_coil_c,
            pin_coil_d,
            current_index: 0,
            current_step: 0,
        }
    }

    pub fn current_index(&self) -> u32 {
        self.current_index
    }
    pub fn reset_index_to(&mut self, val: u32) {
        self.current_index = val;
    }

    /// Returns: the index after having advanced one step
    pub fn advance_step(&mut self) -> Result<u32, E> {
        self.current_step = match self.current_step % 4 {
            0 => {
                self.pin_coil_a.set_high()?;
                self.pin_coil_b.set_high()?;
                self.pin_coil_c.set_low()?;
                self.pin_coil_d.set_low()?;
                3
            }
            1 => {
                self.pin_coil_a.set_low()?;
                self.pin_coil_b.set_high()?;
                self.pin_coil_c.set_high()?;
                self.pin_coil_d.set_low()?;
                0
            }
            2 => {
                self.pin_coil_a.set_low()?;
                self.pin_coil_b.set_low()?;
                self.pin_coil_c.set_high()?;
                self.pin_coil_d.set_high()?;
                1
            }
            _ => {
                self.pin_coil_a.set_high()?;
                self.pin_coil_b.set_low()?;
                self.pin_coil_c.set_low()?;
                self.pin_coil_d.set_high()?;
                2
            }
        };
        self.current_index += 1;
        if self.current_index == STEPS_PER_REVOLUTION {
            self.current_index = 0;
        }
        Ok(self.current_index)
    }
    pub fn stop(&mut self) -> Result<u32, E> {
        self.pin_coil_a.set_low()?;
        self.pin_coil_b.set_low()?;
        self.pin_coil_c.set_low()?;
        self.pin_coil_d.set_low()?;
        Ok(self.current_index)
    }
}
