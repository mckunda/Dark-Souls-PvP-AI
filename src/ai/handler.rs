use std::ffi::c_void;
use std::process::ExitCode;
use std::sync::Mutex;
use std::thread::sleep;
use std::time;
use const_format::concatcp;
use lazy_static::lazy_static;
use vjoy_sys::{DWORD, JOYSTICK_POSITION_V2, vJoyInterface};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use crate::ai::character::{Enemy, Player, player_base_add, ReadPlayer, ReadPointerEndAddresses};
use crate::ai::ffi::clock;
use crate::ai::gui::{LocationMemoryEnemy, LocationMemoryPlayer, LocationHandler};
use crate::ai::guiPrint;
use crate::ai::helper_util::{cross, dcenter, ddown, distance, square};
use crate::ai::initalize_fann::{GetTrainingData, SetupTraining, trainFromFile};
use crate::ai::memory_edits::FindPointerAddr;
use crate::ai::settings::{AutoRedSign, DisableAi, FeedNeuralNet, NeuralNetFolderLocation, TrainNeuralNet};
use crate::ai::source::{Exit, iInterface, MainLogicLoop, memorybase, processHandle, SetupandLoad};
use crate::ai::vjoyhelper::{iReport, ResetVJoyController};

//visual state. used for auto red signing
static Player_visual_offsets: &'static [u64;Player_visual_offsets_length] = &[ 0x28, 0x0, 0x30, 0xC, 0x70 ];
const Player_visual_offsets_length: usize = 5;
lazy_static!(
    static ref visualStatus_address: Mutex<u64> = Mutex::new(0);
);
lazy_static!(
    static ref visualStatus: Mutex<i32> = Mutex::new(0);
);

//current selected item
const SelectedItemBaseAddr: u64 = 0xF7F8F4;
static Player_selectedItem_offsets: &'static [u64;Player_selectedItem_offsets_length] = &[ 0x67C, 0xC, 0x18, 0x730, 0x2D4 ];
const Player_selectedItem_offsets_length: usize = 5;
lazy_static!(
    static ref selectedItem_address: Mutex<u64> = Mutex::new(0);
);

lazy_static!(
    static ref selectedItem: Mutex<i32> = Mutex::new(0);
);

const RedSoapstone: i32 = 101;

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
    static ref RedSignDown: Mutex<bool> = Mutex::new(false);
);

pub unsafe fn PutDownRedSign() {
    ResetVJoyController();
    let mut ir = iReport.lock().unwrap(); // TODO: error handling
    let vj = vJoyInterface::new("").unwrap(); // TODO: error handling
    let mut rsd = RedSignDown.lock().unwrap(); // TODO: error handling
	//press x (in case we have a message appearing), down to goto next item, and check if we selected RSS
	for _ in [0..5] {
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
    static ref RereadPointerEndAddress: Mutex<bool> = Mutex::new(true);
);

lazy_static!(
    static ref LastRedSignTime: Mutex<i64> = Mutex::new(0);
);

unsafe fn main() -> Result<ExitCode,()> {
    if FeedNeuralNet {
        trainFromFile(70,
                      concatcp!(NeuralNetFolderLocation, "/attack_training_data.train"),
                      concatcp!(NeuralNetFolderLocation, "/attack_training_data.test"),
                      concatcp!(NeuralNetFolderLocation, "/Attack_dark_souls_ai.net"));
        trainFromFile(30,
                      concatcp!(NeuralNetFolderLocation, "Neural Nets/backstab_training_data.train"),
                      concatcp!(NeuralNetFolderLocation, "/backstab_training_data.test"),
                      concatcp!(NeuralNetFolderLocation, "/Defense_dark_souls_ai.net"));
        sleep(time::Duration::from_millis(7000));
    }
    let Setuperror = SetupandLoad();
    if !DisableAi {
        if Setuperror != 0 {
            return Ok(ExitCode::FAILURE);
        }
    }
    if TrainNeuralNet {
        SetupTraining();
    }

    loop {
        if AutoRedSign {
            let mut rpea = RereadPointerEndAddress.lock().unwrap(); // TODO: error handling
            guiPrint!("{},0:RereadPointerEndAddress {}", LocationHandler, rpea);

            let mut vs = visualStatus.lock().unwrap(); // TODO: error handling
            let mut e = Enemy.lock().unwrap(); // TODO: error handling
            guiPrint!("{},1:Enemy.loc_x {}\nvisualStatus {}", LocationHandler, e.loc_x, vs);
            guiPrint!("{},2:", LocationHandler);

            let mut vsa = visualStatus_address.lock().unwrap(); // TODO: error handling
            let mut sia = selectedItem_address.lock().unwrap(); // TODO: error handling
            let mut si = selectedItem.lock().unwrap(); // TODO: error handling
            let mut p = Player.lock().unwrap(); // TODO: error handling
            let pba = player_base_add.lock().unwrap(); // TODO: error handling
            let mb = memorybase.lock().unwrap(); // TODO: error handling
            let mut rsd = RedSignDown.lock().unwrap(); // TODO: error handling
            let mut lrst = LastRedSignTime.lock().unwrap(); // TODO: error handling
            let ph = processHandle.lock().unwrap(); // TODO: error handling

            if *rpea {
                ReadPointerEndAddresses(*ph);
                *vsa = FindPointerAddr(*ph, *pba, Player_visual_offsets_length, Player_visual_offsets);
                *sia = FindPointerAddr(*ph, *mb + SelectedItemBaseAddr, Player_selectedItem_offsets_length, Player_selectedItem_offsets);
                ReadPlayer(&mut e, *ph, LocationMemoryEnemy);
                ReadPlayer(&mut p, *ph, LocationMemoryPlayer);
                ResetVJoyController();//just in case
                let vj = vJoyInterface::new("").unwrap(); // TODO: error handling
                let mut ir = iReport.lock().unwrap(); // TODO: error handling
                vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
            }
            //this memory read isnt directly AI related
            ReadProcessMemory(*ph, *vsa as *const c_void, &mut vs as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: error handling
            ReadProcessMemory(*ph, *sia as *const c_void, &mut si as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: error handling

            //if AI is a red phantom
            if *vs == 2 {
                *rsd = false;
                //check that we got the enemy's struct address by ensuring their x loc is pos.
                if e.loc_x > 0. {
                    *rpea = false;
                }
                //once one character dies
                if p.hp <= 0 || e.hp <= 0 {
                    *rpea = true;
                } else {
                    //enemy player is fairly close
                    if distance(&p, &e) < 50. {
                        if TrainNeuralNet {
                            GetTrainingData();
                        } else {
                            MainLogicLoop();
                        }
                    }
                    //last resort error catching
                    else {
                        *rpea = true;
                    }
                    //if enemy player far away, black crystal out
                    /*else if (!RereadPointerEndAddress){
                        guiPrint(LocationHandler",2:BlackCrystalOut");
                        RereadPointerEndAddress = true;
                        BlackCrystalOut();
                    }*/
                }
            }
            //if AI in host world, and red sign not down, put down red sign
            else if *vs == 0 {
                //ocasionally reput down red sign(failed to join session error catcher)
                if !*rsd {
                    guiPrint!("{},2:PutDownRedSign", LocationHandler);
                    sleep(time::Duration::from_millis(10000));//ensure we're out of loading screen
                    PutDownRedSign();
                } else if clock() >= *lrst + 360000 {//6 min
                    *lrst = clock();
                    *rsd = false;
                }
            } else {
                *rpea = true;
            }
        } else {
            if TrainNeuralNet {
                GetTrainingData();
            } else {
                MainLogicLoop();
            }
        }
    }

    Exit();
    Ok(ExitCode::SUCCESS)
}