use std::sync::Mutex;
use std::thread::sleep;
use std::time;
use lazy_static::lazy_static;
use vjoy_sys::{DWORD, JOYSTICK_POSITION_V2, vJoyInterface};
use crate::ai::source::iInterface;
use crate::ai::vjoyhelper::{iReport, ResetVJoyController};
use crate::constants::{cross, dcenter, ddown, square};

//visual state. used for auto red signing
pub static Player_visual_offsets: &'static [u64;Player_visual_offsets_length] = &[ 0x28, 0x0, 0x30, 0xC, 0x70 ];
pub const Player_visual_offsets_length: usize = 5;
lazy_static!(
    pub static ref visualStatus_address: Mutex<u64> = Mutex::new(0);
);
lazy_static!(
    pub static ref visualStatus: Mutex<i32> = Mutex::new(0);
);

//current selected item
pub const SelectedItemBaseAddr: u64 = 0xF7F8F4;
pub static Player_selectedItem_offsets: &'static [u64;Player_selectedItem_offsets_length] = &[ 0x67C, 0xC, 0x18, 0x730, 0x2D4 ];
pub const Player_selectedItem_offsets_length: usize = 5;
lazy_static!(
    pub static ref selectedItem_address: Mutex<u64> = Mutex::new(0);
);

lazy_static!(
    pub static ref selectedItem: Mutex<i32> = Mutex::new(0);
);

pub const RedSoapstone: i32 = 101;

pub unsafe fn BlackCrystalOut() {
    ResetVJoyController();
    let mut ir = iReport.lock().unwrap(); // TODO: error handling
    //switch to black crystal
    ir.bHats = ddown as DWORD;
    let vj = vJoyInterface::new("").unwrap(); // TODO: error handling
    vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
    sleep(time::Duration::from_millis(100));
    ir.bHats = dcenter as DWORD;
    vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
    sleep(time::Duration::from_millis(1000)); //gotta wait for menu to change
    //use
    ir.lButtons = square;
    vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
    sleep(time::Duration::from_millis(100));
    //yes i want to leave
    ir.lButtons = 0x0;
    vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
    sleep(time::Duration::from_millis(100));
    ir.lButtons = cross;
    vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
    sleep(time::Duration::from_millis(500));
    //down d pad again to go back to red sign
    ir.bHats = ddown as DWORD;
    ir.lButtons = 0x0;
    vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
    sleep(time::Duration::from_millis(100));
    //wait
    ir.bHats = dcenter as DWORD;
    vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
    sleep(time::Duration::from_millis(10000));//10 sec is how long it takes to black crystal
}

lazy_static!(
    pub static ref RedSignDown: Mutex<bool> = Mutex::new(false);
);

pub unsafe fn PutDownRedSign() {
    ResetVJoyController();
    let mut ir = iReport.lock().unwrap(); // TODO: error handling
    let vj = vJoyInterface::new("").unwrap(); // TODO: error handling
    let mut rsd = RedSignDown.lock().unwrap(); // TODO: error handling
	//press x (in case we have a message appearing), down to goto next item, and check if we selected RSS
	for _ in 0..5 {
		ir.bHats = cross as DWORD;
		vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
		sleep(time::Duration::from_millis(100));
		ResetVJoyController();
	}

// #if 0
//     while (selectedItem != RedSoapstone)
// 	{
// 		ResetVJoyController();
// 		UpdateVJD(iInterface, (PVOID)&iReport);
// 		Sleep(100);
//         iReport.bHats = ddown;
//         UpdateVJD(iInterface, (PVOID)&iReport);
//         Sleep(100);
//         iReport.bHats = dcenter;
//         UpdateVJD(iInterface, (PVOID)&iReport);
//         Sleep(1000); //gotta wait for menu to change
//         selectedItem_address = FindPointerAddr(processHandle, memorybase + SelectedItemBaseAddr, Player_selectedItem_offsets_length, Player_selectedItem_offsets);
//         ReadProcessMemory(processHandle, (LPCVOID)(selectedItem_address), &(selectedItem), 4, 0);
//         guiPrint(LocationHandler",2:Selected Item:%d", selectedItem);
//     }
// #endif
	//use RSS
    ir.lButtons = square;
    vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
    sleep(time::Duration::from_millis(100));

	ResetVJoyController();
	vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);

// #if 0
    // change back selected item for when summoned
    // iReport.bHats = ddown;
    // UpdateVJD(iInterface, (PVOID)&iReport);
    // Sleep(100);
    // iReport.bHats = dcenter;
    // UpdateVJD(iInterface, (PVOID)&iReport);
// #endif

    *rsd = true;
}

lazy_static!(
    pub static ref RereadPointerEndAddress: Mutex<bool> = Mutex::new(true);
);

lazy_static!(
    pub static ref LastRedSignTime: Mutex<i64> = Mutex::new(0);
);
