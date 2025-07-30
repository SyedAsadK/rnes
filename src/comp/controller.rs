use bitflags::bitflags;
bitflags! {
       // https://wiki.nesdev.com/w/index.php/Controller_reading_code
       pub struct ControllerButtons: u8 {
           const RIGHT             = 0b10000000;
           const LEFT              = 0b01000000;
           const DOWN              = 0b00100000;
           const UP                = 0b00010000;
           const START             = 0b00001000;
           const SELECT            = 0b00000100;
           const BUTTON_B          = 0b00000010;
           const BUTTON_A          = 0b00000001;
       }
}
pub struct Controller {
    strobe: bool,
    button_idx: u8,
    button_status: ControllerButtons,
}
impl Controller {
    pub fn new() -> Self {
        Controller {
            strobe: false,
            button_idx: 0,
            button_status: ControllerButtons::from_bits_truncate(0),
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.button_idx = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_idx > 7 {
            return 1;
        }
        let resp = (self.button_status.bits & (1 << self.button_idx)) >> self.button_idx;
        if !self.strobe && self.button_idx <= 7 {
            self.button_idx += 1;
        }
        resp
    }

    pub fn set_button_pressed_status(&mut self, button: ControllerButtons, pressed: bool) {
        self.button_status.set(button, pressed);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_strobe_mode() {
        let mut joypad = Controller::new();
        joypad.write(1);
        joypad.set_button_pressed_status(ControllerButtons::BUTTON_A, true);
        for _x in 0..10 {
            assert_eq!(joypad.read(), 1);
        }
    }

    #[test]
    fn test_strobe_mode_on_off() {
        let mut joypad = Controller::new();

        joypad.write(0);
        joypad.set_button_pressed_status(ControllerButtons::RIGHT, true);
        joypad.set_button_pressed_status(ControllerButtons::LEFT, true);
        joypad.set_button_pressed_status(ControllerButtons::SELECT, true);
        joypad.set_button_pressed_status(ControllerButtons::BUTTON_B, true);

        for _ in 0..=1 {
            assert_eq!(joypad.read(), 0);
            assert_eq!(joypad.read(), 1);
            assert_eq!(joypad.read(), 1);
            assert_eq!(joypad.read(), 0);
            assert_eq!(joypad.read(), 0);
            assert_eq!(joypad.read(), 0);
            assert_eq!(joypad.read(), 1);
            assert_eq!(joypad.read(), 1);

            for _x in 0..10 {
                assert_eq!(joypad.read(), 1);
            }
            joypad.write(1);
            joypad.write(0);
        }
    }
}
