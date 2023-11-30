use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::Mutex;
use lazy_static::lazy_static;
use vjoy_sys::{BOOL, VjdStat, VjdStat_VJD_STAT_FREE, VjdStat_VJD_STAT_OWN, vJoyInterface, WORD};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use crate::ai::animation_mappings::{AnimationTypes, isAttackAnimation, isDodgeAnimation};
use crate::ai::character::{Character, Enemy};
use crate::ai::guiPrint;
use crate::ai::memory::{last_animation_types_enemy, last_animation_types_enemy_LENGTH};
use crate::ai::memory_edits::FindPointerAddr;
use crate::ai::settings::OolicelMap;
use crate::ai::weapon_data::StaminaDrainForAttack;
use crate::constants::{EnemyId, XRIGHT, YBOTTOM};

// int loadvJoy(UINT iInterface);

pub const HID_USAGE_X: u32 = 0x30;
pub const HID_USAGE_Y: u32 = 0x31;
pub const HID_USAGE_Z: u32 = 0x32;
pub const HID_USAGE_RX: u32 = 0x33;
pub const HID_USAGE_RY: u32 = 0x34;
pub const HID_USAGE_RZ: u32 = 0x35;
pub const HID_USAGE_SL0: u32 = 0x36;
pub const HID_USAGE_SL1: u32 = 0x37;
pub const HID_USAGE_WHL: u32 = 0x38;
pub const HID_USAGE_POV: u32 = 0x39;

pub struct longTuple {
	pub x_axis: i64,
	pub y_axis: i64,
}

//store camera settings
pub struct CameraSett {
	cam_x_addr: u64,
	cam_x: f32,
	cam_y_addr: u64,
	cam_y: f32,
	rot_x_addr: u64,
	rot_x: f32,
	rot_y_addr: u64,
	rot_y: f32,
}

lazy_static!(
    static ref camera: Mutex<CameraSett> = Mutex::new(CameraSett{
        cam_x_addr: 0,
        cam_x: 0f32,
        cam_y_addr: 0,
        cam_y: 0f32,
        rot_x_addr: 0,
        rot_x: 0f32,
        rot_y_addr: 0,
        rot_y: 0f32,
    });
);

//address data to read camera settings
lazy_static!(
    static ref camera_base: Mutex<u64> = Mutex::new(0x00F5CDD4);
);

static camera_y_offsets: &'static [u64]     = &[ 0x174, 0x4D4, 0x144, 0x320, 0xF8 ];
static camera_x_offsets: &'static [u64]     = &[ 0x174, 0x4D4, 0x144, 0x320, 0x90 ];
static camera_y_rot_offsets: &'static [u64] = &[ 0x174, 0x4D4, 0x144, 0x320, 0x150 ];
static camera_x_rot_offsets: &'static [u64] = &[ 0x174, 0x4D4, 0x144, 0x320, 0x144 ];
static camera_offsets_length: usize = 5;

//get straight line distance between player and enemy
pub fn distance(Player: &Character, Phantom: &Character) -> f32 {
    let delta_x: f32 = (Player.loc_x.abs() - Phantom.loc_x.abs()).abs();
    let delta_y: f32 = (Player.loc_y.abs() - Phantom.loc_y.abs()).abs();
    delta_x.hypot(delta_y)
}

//the absolute value of the angle the opponent is off from straight ahead (returns radians, only used as neural net input)
//TODO this only works from front 180, after which it mirrors. THIS TREATS FRONT SAME AS BACK. BAD.
pub fn angleDeltaFromFront(Player: &Character, Phantom: &Character) -> f32 {
    let delta_x: f32 = (Player.loc_x.abs() - Phantom.loc_x.abs()).abs();
    let delta_y: f32 = (Player.loc_y.abs() - Phantom.loc_y.abs()).abs();

    //if its closer to either 90 or 270 by 45, its x direction facing
    return if (Player.rotation > 45f32) && (Player.rotation < 135f32) || (Player.rotation > 225f32) && (Player.rotation < 315f32) {
        f32::atan(delta_y / delta_x)
    } else {
        f32::atan(delta_x / delta_y)
    }
}

pub fn AnglesWithinRange(ang1: f32, ang2: f32, range: f32) -> bool {
    let mut ang1Max: f32 = ang1 + range;

    if ang1Max > 360f32 {
        ang1Max -= 360.;
    }

    let mut ang1Min: f32 = ang1 - range;
    if ang1Min < 0f32 {
        ang1Min += 360.;
    }

    //dont have to worry about 0 problem
    if ang1Min < ang1Max {
        ang1Min <= ang2 && ang2 <= ang1Max
    } else {
        //split into two sides across the 0 mark, check if player in either
        (ang1Min <= ang2 && ang2 <= 360f32) || (0. <= ang2 && ang2 <= ang1Max)
    }
}

//since stamina isnt sent over wire estimate the enemy's from last_animation_types_enemy
pub fn StaminaEstimationEnemy() -> i32 {
    let mut staminaEstimate: i32 = 192;//assume their max stamina is max
    let mut l = last_animation_types_enemy.lock().unwrap(); // TODO: handle error
    for i in (0..last_animation_types_enemy_LENGTH).rev() {
        // backsteps. these have diff stamina drain from other rolls
        if l[i] == AnimationTypes::Backstep_1H as u16 || l[i] == AnimationTypes::Backstep_2H as u16 {
            staminaEstimate -= 19;
        }
        else if isDodgeAnimation(l[i]) {
            staminaEstimate -= 28;
        }
        else if isAttackAnimation(l[i] as u8) != 0 {
            let e = Enemy.lock(); // TODO: handle error
            let e = e.borrow();
            //assuming they haven't switched weapons during this time, use their right weapon and the attack type to get the stamina drain
            staminaEstimate -= StaminaDrainForAttack(e.r_weapon_id, e.animationType_id as u16);
        }
        //bug: this includes running, which drains stamina
        else if  l[i] == AnimationTypes::Nothing as u16
            || l[i] == AnimationTypes::Shield_Held_Up as u16
            || l[i] == AnimationTypes::Shield_Held_Up_walking as u16 {
            let e = Enemy.lock();
            let e = e.borrow(); // TODO: handle error
            staminaEstimate += e.staminaRecoveryRate / 10;
        }

        //cap max and min stam
        if staminaEstimate > 192 {
            staminaEstimate = 192;
        }
        else if staminaEstimate < -40 {
            staminaEstimate = -40;
        }
    }

    guiPrint!("{},5:Stamina Est:{}", EnemyId, staminaEstimate);
    return staminaEstimate;
}

//handles rollover from 360 to 0
//player is +-60 degrees relative to the enemy rotation (yes, thats all to it)
pub const BackstabDegreeRange: f32 = 60.;
pub fn InBackstabRange(enemy: f32, player: f32) -> bool {
    let mut enemypos: f32 = enemy + BackstabDegreeRange;
    if enemypos > 360f32 {
        enemypos -= 360.;
    }
    let mut enemyneg: f32 = enemy - BackstabDegreeRange;
    if enemyneg < 0f32 {
        enemyneg += 360.;
    }

    //dont have to worry about 0 problem
    if enemyneg < enemypos {
        enemyneg <= player && player <= enemypos
    } else {
        //split into two sides across the 0 mark, check if player in either
        enemyneg <= player && player <= 360f32 || 0. <= player && player <= enemypos
    }
}

//check if player behind enemy, and if they're in +-60 degree bs window
pub const BackstabRange: f32 = 1.5;
//the determiner for if behind someone also must include both characters body widths. or it could be a cone, instead of a line. not sure.
pub const RealBehindRange: f32 = 0.1;
pub fn BackstabDetection_CounterClockwise(player: &Character, enemy: &Character, distance: f32) -> u8 {
    let angle: f32 = enemy.rotation;
    let y_dist: f32 = enemy.loc_y - player.loc_y;
    let x_dist: f32 = enemy.loc_x - player.loc_x;

    //if enemy in 1st segmented area
    //each segment is 90 degrees
    if (angle <= 360f32 && angle >= 315f32) || (angle <= 45f32 && angle >= 0f32) {
        //player is behind them
        if y_dist > RealBehindRange {
            //player is in backstab rotation and distance in allowable range
            if InBackstabRange(angle, player.rotation) && distance <= BackstabRange {
                return 2;
            }
            return 1;
        }
        return 0;
    } else if angle <= 315f32 && angle >= 225f32 {
        if x_dist < -RealBehindRange {
            if InBackstabRange(angle, player.rotation) && distance <= BackstabRange {
                return 2;
            }
            return 1;
        }
        return 0;
    } else if angle <= 225f32 && angle >= 135f32 {
        if y_dist < -RealBehindRange {
            if InBackstabRange(angle, player.rotation) && distance <= BackstabRange {
                return 2;
            }
            return 1;
        }
        return 0;
    } else {
        if x_dist > RealBehindRange {
            if InBackstabRange(angle, player.rotation) && distance <= BackstabRange {
                return 2;
            }
            return 1;
        }
        return 0;
    }
}
    
//this tests if safe FROM backstabs and able TO backstab
//determines the state of a backstab from the perspecitve's view with respect to the target
//0 is no backstab, 1 is behind target, 2 is backstab
pub fn BackstabDetection(Perspective: &Character, Target: &Character, distance: f32) -> u8 {
    return BackstabDetection_CounterClockwise(Perspective, Target, distance);
}

pub fn rotationDifferenceFromSelf(Player: &Character, Phantom: &Character) -> f32 {
    ((Player.rotation) - (Phantom.rotation)).abs()
}

pub unsafe fn loadvJoy(iInterface: u32) -> i32 {
    let i = vJoyInterface::new("").unwrap(); // TODO: vJoy path & error handling

	// Get the driver attributes (Vendor ID, Product ID, Version Number)
	if 0 == i.vJoyEnabled() {
		println!("vJoy driver not enabled: Failed Getting vJoy attributes.\n");
		return -2;
	} else {
        let man = CString::from(CStr::from_ptr(i.GetvJoyManufacturerString() as *const c_char)).into_string().unwrap(); // TODO: error handling
        let prod = CString::from(CStr::from_ptr(i.GetvJoyProductString() as *const c_char)).into_string().unwrap(); // TODO: error handling
        let serial = CString::from(CStr::from_ptr(i.GetvJoySerialNumberString() as *const c_char)).into_string().unwrap(); // TODO: error handling
		println!("Vendor: {}\nProduct :{}\nVersion Number:{}\n", man, prod, serial);
	};

    let mut VerDll: WORD = 0;
    let mut VerDrv: WORD = 0;

    if i.DriverMatch(&mut VerDll, &mut VerDrv) == 0 {
		println!("Failed\r\nvJoy Driver (version {:04x}) does not match vJoyInterface DLL (version {:04x})\n", VerDrv, VerDll);
	} else{
		println!("OK - vJoy Driver and vJoyInterface DLL match vJoyInterface DLL (version {:04x})\n", VerDrv);
	}

	// Get the state of the requested device
	let status: VjdStat = i.GetVJDStatus(iInterface);
	match status {
        vjoy_sys::VjdStat_VJD_STAT_OWN => {
            println!("vJoy Device {} is already owned by this feeder\n", iInterface);
        },
        vjoy_sys::VjdStat_VJD_STAT_FREE => {
            println!("vJoy Device {} is free\n", iInterface);
        },
        vjoy_sys::VjdStat_VJD_STAT_BUSY => {
            println!("vJoy Device {} is already owned by another feeder\nCannot continue\n", iInterface);
            return -3;
        }
        vjoy_sys::VjdStat_VJD_STAT_MISS => {
            println!("vJoy Device {} is not installed or disabled\nCannot continue\n", iInterface);
            return -4;
        }
        _ => {
            println!("vJoy Device {} general error\nCannot continue\n", iInterface);
            return -1;
        }
	};


	// Check which axes are supported
	let AxisX: BOOL = i.GetVJDAxisExist(iInterface, HID_USAGE_X);
	let AxisY: BOOL = i.GetVJDAxisExist(iInterface, HID_USAGE_Y);
	let AxisZ: BOOL = i.GetVJDAxisExist(iInterface, HID_USAGE_Z);
	let AxisRX: BOOL = i.GetVJDAxisExist(iInterface, HID_USAGE_RX);
	let AxisRY: BOOL = i.GetVJDAxisExist(iInterface, HID_USAGE_RY);
	// Get the number of buttons and POV Hat switchessupported by this vJoy device
	let nButtons: i32 = i.GetVJDButtonNumber(iInterface);
	let ContPovNumber: i32 = i.GetVJDContPovNumber(iInterface);
	let DiscPovNumber: i32 = i.GetVJDDiscPovNumber(iInterface);

	// Print results
	println!("\nvJoy Device {} capabilities:\n", iInterface);
	println!("Numner of buttons\t\t{}\n", nButtons);
	println!("Numner of Continuous POVs\t{}\n", ContPovNumber);
	println!("Numner of Descrete POVs\t\t{}\n", DiscPovNumber);
	println!("Axis X\t\t{}\n", if AxisX != 0 { "Yes" } else { "No" });
	println!("Axis Y\t\t{}\n", if AxisY != 0 { "Yes" } else { "No" });
	println!("Axis Z\t\t{}\n", if AxisZ != 0 { "Yes" } else { "No" });
	println!("Axis Rx\t\t{}\n", if AxisRX != 0 { "Yes" } else { "No" });
	println!("Axis Ry\t\t{}\n", if AxisRY != 0 { "Yes" } else { "No" });

    if AxisX == 0 || AxisY == 0 || AxisZ == 0 || AxisRX == 0 || AxisRY == 0 || nButtons < 10 {
        println!("Invalid config\n");
        return -1;
    }

	// Acquire the target
	if status == VjdStat_VJD_STAT_OWN || (status == VjdStat_VJD_STAT_FREE && i.AcquireVJD(iInterface) == 0){
		println!("Failed to acquire vJoy device number {}.\n", iInterface);
		return -1;
	} else{
		println!("Acquired: vJoy device number {}.\n", iInterface);
	}
	return 0;
}

//given player and enemy coordinates, get the angle between the two
pub fn angleFromCoordinates(player_x: f32, phantom_x: f32, player_y: f32, phantom_y: f32) -> f64 {
    let mut delta_x: f64 = 0f64;
    let mut delta_y: f64 = 0f64;

    if OolicelMap != 0 {
        delta_x = (player_x.abs() - phantom_x.abs()) as f64;
        delta_y = (phantom_y.abs() - player_y.abs()) as f64;
    } else {
        delta_x = (phantom_x.abs() - player_x.abs()) as f64;
        delta_y = (player_y.abs() - phantom_y.abs()) as f64;
    }

	//convert this to 360 degrees
	let mut angle: f64 = (f64::atan2(delta_x, delta_y) + std::f64::consts::PI) * (180.0 / std::f64::consts::PI);

    if OolicelMap == 0 {
        angle -= 90.0;
    }

    return angle;
}

pub fn angleToJoystick_Clockwise(angle: f64, tuple: &mut longTuple) {
	tuple.x_axis = ((XRIGHT as f64 * ((angle * (std::f64::consts::PI / 180.0)).cos() + 1.)) / 2.0) as i64;
	tuple.y_axis = ((YBOTTOM as f64 * ((angle * (std::f64::consts::PI / 180.0)).sin() + 1.)) / 2.0) as i64;
}

pub fn angleToJoystick_CounterClockwise(angle: f64, tuple: &mut longTuple) {
	tuple.x_axis = ((XRIGHT as f64 * ((angle * (std::f64::consts::PI / 180.0) + (std::f64::consts::PI / 2.0)).cos() + 1.)) / 2.0) as i64;
	tuple.y_axis = ((YBOTTOM as f64 * ((angle * (std::f64::consts::PI / 180.0) - (std::f64::consts::PI / 2.0)).sin() + 1.)) / 2.0) as i64;
}

/*
Basic Polar to Cartesian conversion

this will return a tuple of 2 values each in the range 0x1-0x8000(32768).
The first is the x direction, which has 1 as leftmost and 32768 as rightmost
second is y, which has 1 as topmost and 32768 as bottommost

MUST LOCK CAMERA for movement to work. it rotates with your movement direction, which messes with it.
aligning camera with 0 on rotation x points us along y axis, facing positive, and enemy moves clockwise around us
*/
pub fn angleToJoystick(angle: f64, tuple: &mut longTuple) {
    if OolicelMap != 1 {
        angleToJoystick_CounterClockwise(angle, tuple);
    } else {
        angleToJoystick_Clockwise(angle, tuple);
    }
}
//get current camera details to lock
pub unsafe fn readCamera(processHandle: HANDLE, memorybase: u64) {
    let mut cb = camera_base.lock().unwrap(); // TODO: error handling
    let mut c = camera.lock().unwrap(); // TODO: error handling

	//get camera base address
	*cb += memorybase;

	//get final address
	c.cam_y_addr = FindPointerAddr(processHandle, *cb, camera_offsets_length, camera_y_offsets);
	//read y location
	ReadProcessMemory(processHandle, c.cam_y_addr as *const c_void, &mut (c.cam_y) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: error handling

	//get final address
	c.cam_x_addr = FindPointerAddr(processHandle, *cb, camera_offsets_length, camera_x_offsets);
	//read x location
	ReadProcessMemory(processHandle, c.cam_x_addr as *const c_void, &mut (c.cam_x) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: error handling

	//get rotation addresses
	c.rot_y_addr = FindPointerAddr(processHandle, *cb, camera_offsets_length, camera_y_rot_offsets);
	c.rot_x_addr = FindPointerAddr(processHandle, *cb, camera_offsets_length, camera_x_rot_offsets);
}

//set the camera to a fixed position and rotation.
pub fn lockCamera(processHandle: HANDLE){
	//TODO do i need to attach to process in order to write?
	// let processHandle_nonPoint: HANDLE = *processHandle;
	//set x location
	/*WriteProcessMemory(processHandle_nonPoint, (LPCVOID)(camera->cam_x_addr), (LPCVOID)(camera->cam_x), 4, NULL);
	//set y location
	WriteProcessMemory(processHandle_nonPoint, (LPCVOID)(camera->cam_y_addr), (LPCVOID)(camera->cam_y), 4, NULL);
	//set x rotation to ???
	float pi = std::f64::consts::PI;
	WriteProcessMemory(processHandle_nonPoint, (LPCVOID)(camera->rot_x_addr), &pi, 4, NULL);
	//set y rotation to anything, this doesnt matter*/

}


/*#include <stdio.h>//println!

int main(void){
	longTuple a = CoordsToJoystickAngle(26.08102417, 31.13756943, -16.64873314, -17.59091759);
	println!("x: %i y: %i \n", a.first, a.second);
	double x = ((a.first / (double)32768) * (double)100);
	double y = ((a.second / (double)32768) * (double)100);
	println!("x: %f y: %f \n----------------------\n", x, y);
	return 0;
}*/