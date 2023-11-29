pub mod ai;
mod fann;
mod fann_sys;
mod constants;

extern crate lazy_static;

use std::ffi::c_void;
use std::process::ExitCode;
use std::thread::sleep;
use std::time;
use const_format::concatcp;
use vjoy_sys::{JOYSTICK_POSITION_V2, vJoyInterface};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use crate::ai::character::{Enemy, Player, player_base_add, ReadPlayer, ReadPointerEndAddresses};
use crate::ai::ffi::clock;
use crate::ai::gui::{LocationMemoryEnemy, LocationMemoryPlayer, LocationHandler};
use crate::ai::guiPrint;
use crate::ai::initalize_fann::{GetTrainingData, SetupTraining, trainFromFile};
use crate::ai::memory_edits::FindPointerAddr;
use crate::ai::settings::{AutoRedSign, DisableAi, FeedNeuralNet, NeuralNetFolderLocation, TrainNeuralNet};
use crate::ai::source::{Exit, iInterface, MainLogicLoop, memorybase, processHandle, SetupandLoad};
use crate::ai::handler::{LastRedSignTime, Player_selectedItem_offsets, Player_selectedItem_offsets_length, Player_visual_offsets, Player_visual_offsets_length, PutDownRedSign, RedSignDown, RereadPointerEndAddress, selectedItem, selectedItem_address, SelectedItemBaseAddr, visualStatus, visualStatus_address};
use crate::ai::helper_util::distance;
use crate::ai::vjoyhelper::{iReport, ResetVJoyController};

fn main() -> Result<ExitCode, ()> {
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
        unsafe { SetupTraining(); }
    }

    loop {
        if AutoRedSign {
            let mut rpea = RereadPointerEndAddress.lock().unwrap(); // TODO: error handling
            guiPrint!("{},0:RereadPointerEndAddress {}", LocationHandler, rpea);

            let mut vs = visualStatus.lock().unwrap(); // TODO: error handling
            let mut e = Enemy.lock();
            let mut e = e.borrow_mut(); // TODO: error handling
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
                unsafe { ReadPointerEndAddresses(*ph); }
                unsafe { *vsa = FindPointerAddr(*ph, *pba, Player_visual_offsets_length, Player_visual_offsets); }
                unsafe { *sia = FindPointerAddr(*ph, *mb + SelectedItemBaseAddr, Player_selectedItem_offsets_length, Player_selectedItem_offsets); }
                unsafe { ReadPlayer(&mut e, *ph, LocationMemoryEnemy); }
                unsafe { ReadPlayer(&mut p, *ph, LocationMemoryPlayer); }
                ResetVJoyController();//just in case
                let vj = unsafe { vJoyInterface::new("").unwrap() }; // TODO: error handling
                let mut ir = iReport.lock().unwrap(); // TODO: error handling
                unsafe { vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2); }
            }
            //this memory read isnt directly AI related
            unsafe { ReadProcessMemory(*ph, *vsa as *const c_void, &mut vs as *mut _ as *mut c_void, 4, None).unwrap(); } // TODO: error handling
            unsafe { ReadProcessMemory(*ph, *sia as *const c_void, &mut si as *mut _ as *mut c_void, 4, None).unwrap(); } // TODO: error handling

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
                            unsafe { GetTrainingData(); }
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
                unsafe {
                    if !*rsd {
                        guiPrint!("{},2:PutDownRedSign", LocationHandler);
                        sleep(time::Duration::from_millis(10000));//ensure we're out of loading screen
                        unsafe { PutDownRedSign(); }
                    } else if clock() >= *lrst + 360000 {//6 min
                        *lrst = clock();
                        *rsd = false;
                    }
                }
            } else {
                *rpea = true;
            }
        } else {
            if TrainNeuralNet {
                unsafe { GetTrainingData(); }
            } else {
                MainLogicLoop();
            }
        }
    }

    Exit();
    Ok(ExitCode::SUCCESS)
}
