use crate::motor_driver::{MotorDriver, STEPS_PER_REVOLUTION};
use core::cmp::min;
use embedded_hal::digital::v2::InputPin;

pub const LETTER_COUNT: u8 = 45;

pub struct Module<'a, E> {
    motor: MotorDriver<'a, E>,
    sensor_pin: &'a dyn InputPin<Error = E>,
    target_step: u32,
    offset: u32,
    target_display: u8,
}

impl<'a, E> Module<'a, E> {
    pub fn new(
        motor: MotorDriver<'a, E>,
        sensor_pin: &'a dyn InputPin<Error = E>,
        offset: u32,
    ) -> Module<'a, E> {
        Module {
            motor,
            sensor_pin,
            target_display: 0,
            target_step: 0,
            offset,
        }
    }

    pub fn reset<T>(&mut self, delay: &mut T) -> Result<(), E>
    where
        T: FnMut() -> (),
    {
        while self.sensor_pin.is_low()? {
            self.motor.advance_step()?;
            delay();
        }
        while self.sensor_pin.is_high()? {
            self.motor.advance_step()?;
            delay();
        }
        self.motor.reset_index_to(self.offset);
        self.motor.stop()?;

        Ok(())
    }

    /// Set the target letter for the module
    ///
    /// Letter 0 is the blank flap, 1 is the letter A, etc. up to 45
    pub fn set_target(&mut self, target_display: u8) {
        self.target_display = min(target_display, LETTER_COUNT);

        self.target_step =
            (self.target_display as u32 * STEPS_PER_REVOLUTION) / LETTER_COUNT as u32;
    }

    /// If motor should move, advance it of one step
    pub fn move_to_target(&mut self) -> Result<(), E> {
        if self.moving() {
            self.motor.advance_step()?;
        } else {
            self.motor.stop()?;
        }
        Ok(())
    }

    pub fn moving(&self) -> bool {
        self.motor.current_index() != self.target_step
    }
}
