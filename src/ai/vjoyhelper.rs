use std::sync::Mutex;
use lazy_static::lazy_static;
use vjoy_sys::{DWORD, JOYSTICK_POSITION};
use crate::ai::helper_util::dcenter;
use crate::constants::MIDDLE;
lazy_static!(
    pub static ref iReport: Mutex<JOYSTICK_POSITION> = Mutex::new(JOYSTICK_POSITION{
        bDevice: 0,
        wThrottle: 0,
        wRudder: 0,
        wAileron: 0,
        wAxisX: 0,
        wAxisY: 0,
        wAxisZ: 0,
        wAxisXRot: 0,
        wAxisYRot: 0,
        wAxisZRot: 0,
        wSlider: 0,
        wDial: 0,
        wWheel: 0,
        wAxisVX: 0,
        wAxisVY: 0,
        wAxisVZ: 0,
        wAxisVBRX: 0,
        wAxisVBRY: 0,
        wAxisVBRZ: 0,
        lButtons: 0,
        bHats: 0,
        bHatsEx1: 0,
        bHatsEx2: 0,
        bHatsEx3: 0,
        lButtonsEx1: 0,
        lButtonsEx2: 0,
        lButtonsEx3: 0,
    });
);

pub fn ResetVJoyController() {
    let mut ir = iReport.lock().unwrap(); // TODO: error handling
    // reset struct info
    ir.wAxisX = MIDDLE;
    ir.wAxisY = MIDDLE;
    ir.wAxisZ = MIDDLE;//this is l2 and r2
    ir.wAxisYRot = MIDDLE;
    ir.wAxisXRot = MIDDLE;
    ir.lButtons = 0x0;
    ir.bHats = dcenter as DWORD;//d-pad center
}