use crate::motor_driver::{MotorDriver, STEPS_PER_REVOLUTION};
use core::cmp::min;
use embedded_hal::digital::v2::InputPin;

pub const LETTER_COUNT: u8 = 45;

enum Status {
    Reset1_1,
    Reset1_2,
    Reset2_1,
    Reset2_2,
    Work,
    Stop,
}

pub struct Module<'a, E> {
    motor: MotorDriver<'a, E>,
    sensor_pin: &'a dyn InputPin<Error = E>,
    target_step: u32,
    offset: u32,
    target_display: u8,
    status: Status,
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
            status: Status::Reset1_1,
        }
    }

    pub fn reset(&mut self) {
        self.status = Status::Reset1_1;
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
        match self.status {
            Status::Reset1_1 => {
                if self.sensor_pin.is_low()? {
                    self.motor.advance_step()?;
                    self.status = Status::Reset1_2;
                } else {
                    self.status = Status::Reset2_1;
                }
            }
            Status::Reset1_2 => {
                if self.sensor_pin.is_low()? {
                    self.status = Status::Reset1_1;
                } else {
                    self.status = Status::Reset2_1;
                }
            }
            Status::Reset2_1 => {
                if self.sensor_pin.is_high()? {
                    self.motor.advance_step()?;
                    self.status = Status::Reset2_2;
                } else {
                    self.motor.reset_index_to(self.offset);
                    self.status = Status::Work;
                }
            }
            Status::Reset2_2 => {
                if self.sensor_pin.is_high()? {
                    self.status = Status::Reset2_1;
                } else {
                    self.motor.reset_index_to(self.offset);
                    self.status = Status::Work;
                }
            }
            Status::Work => {
                if self.moving() {
                    self.motor.advance_step()?;
                } else {
                    self.status = Status::Stop;
                }
            }
            Status::Stop => {
                if self.moving() {
                    self.status = Status::Work;
                } else {
                    self.motor.stop()?;
                }}
        }
        Ok(())
    }

    fn moving(&self) -> bool {
        self.motor.current_index() != self.target_step
    }
}
