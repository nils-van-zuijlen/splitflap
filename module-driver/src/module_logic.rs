use crate::motor_driver::{MotorDriver, STEPS_PER_REVOLUTION};
use embedded_hal::digital::v2::InputPin;
use core::cmp::min;

pub const LETTER_COUNT: u8 = 45;

pub struct Module<'a, E> {
    motor: MotorDriver<'a, E>,
    sensor_pin: &'a dyn InputPin<Error = E>,
    target_step: u32,
    offset: u32,
    target_display: u8,
}

impl<'a, E> Module<'a, E> {
    pub fn new(motor: MotorDriver<'a, E>, sensor_pin: &'a dyn InputPin<Error = E>, offset: u32) -> Module<'a, E> {
        Module{motor, sensor_pin, target_display: 0, target_step: 0, offset}
    }

    pub fn reset<T>(&mut self, delay: &mut T) -> Result<(), E> where T: FnMut() -> () {
        while self.sensor_pin.is_high()? {
            self.motor.advance_step()?;
            delay();
        }
        while self.sensor_pin.is_low()? {
            self.motor.advance_step()?;
            delay();
        }
        self.motor.reset_index_to(self.offset);

        Ok(())
    }

    pub fn set_target(&mut self, target_display: u8) {
        self.target_display = min(target_display, LETTER_COUNT);

        self.target_step = (self.target_display as u32 * STEPS_PER_REVOLUTION) / LETTER_COUNT as u32;
    }

    pub fn move_to_target(&mut self) -> Result<(), E> {
        if self.moving() {
            self.motor.advance_step()?;
        }
        Ok(())
    }

    pub fn moving(&self) -> bool {
        self.motor.current_index() != self.target_step
    }
}
