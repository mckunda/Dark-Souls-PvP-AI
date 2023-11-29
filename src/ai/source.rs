use crate::ai::gui::LocationJoystick;
use crate::ai::ai_decisions::PriorityDecision;
use std::sync::Mutex;
use lazy_static::lazy_static;
use vjoy_sys::{JOYSTICK_POSITION_V2, vJoyInterface};
use windows::core::PCSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowA, SetForegroundWindow};
use crate::ai::ai_decisions::{InstinctDecision, InstinctDecisionMaking, SubroutineId};
use crate::ai::ai_decisions::PriorityDecision::EnterDodgeSubroutine;
use crate::ai::ai_methods::{attack, dodge};
use crate::ai::character::{Enemy, Enemy_base_add, Player, player_base_add, ReadPlayer, ReadPointerEndAddresses};
use crate::ai::guiPrint;
use crate::ai::gui::LocationDetection;
use crate::ai::helper_util::{distance, loadvJoy, readCamera};
use crate::ai::memory::AppendDistance;
use crate::ai::memory_edits::{GetModuleBase, GetProcessIdFromName};
use crate::ai::mind_routines::{attack_mind_input, AttackChoice, defense_mind_input, DefenseChoice, ReadyThreads, WaitForThread, WakeThread};
use crate::ai::sub_routines::{AttackId, inActiveAttackSubroutine, inActiveDodgeSubroutine, SafelyExitSubroutines, subroutine_states};
use crate::ai::vjoyhelper::{iReport, ResetVJoyController};
use crate::constants::{EnemyId, PlayerId};

lazy_static!(
    pub static ref processHandle: Mutex<HANDLE> = Mutex::new(HANDLE(0));
);
lazy_static!(
    pub static ref memorybase: Mutex<u64> = Mutex::new(0);
);
lazy_static!(
    pub static ref instinct_decision: Mutex<InstinctDecision> = Mutex::new(InstinctDecision{
        priority_decision: PriorityDecision::EnemyNeutral,
        subroutine_id: SubroutineId::attackid(AttackId::AtkNoneId),
    });
);

pub const iInterface: u32 = 1; // Default target vJoy device

//neural net and desicion making settings/variables

pub fn SetupandLoad() -> i32 {
    //memset to ensure we dont have unusual char attributes at starting
    // memset(&Enemy, 0, sizeof(Character));
    // memset(&Player, 0, sizeof(Character));
    //TODO temp hardcoding
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut ph = processHandle.lock().unwrap(); // TODO: error handling
    let mut mb = memorybase.lock().unwrap(); // TODO: error handling
    let mut pba = player_base_add.lock().unwrap(); // TODO: error handling
    let mut eba = Enemy_base_add.lock().unwrap(); // TODO: error handling
    let mut ir = iReport.lock().unwrap(); // TODO: error handling

    e.weaponRange = 6.;
    p.weaponRange = 2.5;

    //get access to dark souls memory
    let processName = "DarkSoulsRemastered.exe";
    //get the process id from the name
    let processId = unsafe {
        GetProcessIdFromName(processName)
    };

	if processId == 0xffffffff {
		println!("Unable to find DarkSouls.exe\n");
		return -1;
	}

    //open the handle
    *ph = unsafe {
        OpenProcess(PROCESS_ALL_ACCESS, false, processId).unwrap() // TODO: error handling
    };

    //get the base address of the process and append all other addresses onto it
    *mb = unsafe {
        GetModuleBase(processId, processName)
    };

    *eba += *mb;
    *pba += *mb;

    unsafe {
        ReadPointerEndAddresses(*ph);
    };

    //start gui TODO
    // guiStart();

    //get current camera details to lock
    unsafe {
        readCamera(*ph, *mb);
    }

    //load neural network and threads
    let error = ReadyThreads();
    if error != 0 {
        return error;
    }

    //TODO load vJoy driver(we ONLY want the driver loaded when program running)
    //want to use controller input, instead of keyboard, as analog stick is more precise movement
    let loadresult = unsafe {
        loadvJoy(iInterface)
    };

    if loadresult != 0 {
        return loadresult;
    }

    ir.bDevice = iInterface as u8;
    ResetVJoyController();

    //set window focus
    let hwnd = unsafe {
        FindWindowA(PCSTR::null(), PCSTR::from_raw("DARK SOULS".as_ptr()))
    };

    unsafe {
        SetForegroundWindow(hwnd);
    }
    unsafe {
        SetFocus(hwnd);
    }

    return 0;
}

const DebuggingPacifyDef: bool = false;
const DebuggingPacifyAtk: bool = false;


pub fn MainLogicLoop() {
		//TODO lock the camera
		//lockCamera(&processHandle);

    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let ph = processHandle.lock().unwrap();
    let mut dmi = defense_mind_input.lock().unwrap(); // TODO: error handling
    let mut ami = attack_mind_input.lock().unwrap(); // TODO: error handling
    let mut dc = DefenseChoice.lock().unwrap(); // TODO: error handling
    let mut ac = AttackChoice.lock().unwrap(); // TODO: error handling
    let mut ir = iReport.lock().unwrap(); // TODO: error handling
    let mut ind = instinct_decision.lock().unwrap(); // TODO: error handling
    //read the data at these pointers, now that offsets have been added and we have a static address
    unsafe {
        ReadPlayer(&mut e, *ph, EnemyId);
    }
    unsafe {
        ReadPlayer(&mut p, *ph, PlayerId);
    }

    //log distance in memory
    unsafe {
        AppendDistance(distance(&p, &e));
    }

    //start the neural network threads
    WakeThread(&mut dmi);
    WakeThread(&mut ami);

    ResetVJoyController();

    //generate instinct decision
    // instinct_decision.subroutine_id.attackid = AttackId::AtkNoneId;
    // instinct_decision.subroutine_id.defenseid = DefenseId::DefNoneId;
    InstinctDecisionMaking(&mut ind);

    WaitForThread(&mut dmi);
    guiPrint!("{},1:Defense Neural Network detected {}, and Attack {}", LocationDetection, *dc, *ac);

    if DebuggingPacifyDef {
        *dc = 0;
    }

    if ind.priority_decision == EnterDodgeSubroutine || inActiveDodgeSubroutine() || (*dc > 0) {
        unsafe {
            dodge(&mut ir, &mut ind, *dc);
        }
    }

    WaitForThread(&mut ami);
    guiPrint!("{},2:Attack Neural Network decided {}", LocationDetection, *ac);
    if DebuggingPacifyAtk {
        *ac = 0;
    }

    if inActiveAttackSubroutine() || *ac != 0 && *dc == 0 {
        unsafe {
            attack(&mut ir, &mut ind, *ac);
        }
    }

    //unset neural network decisions
    *dc = 0;
    *ac = 0;

    //handle subroutine safe exits
    SafelyExitSubroutines();

    let ss = subroutine_states.lock().unwrap(); // TODO: error handling
    guiPrint!("{},5:Current Subroutine States ={{{},{},{},{}}}", LocationDetection, ss[0], ss[1], ss[2], ss[3]);

    //send this struct to the driver (only 1 call for setting all controls, much faster)
    guiPrint!("{},0:AxisX:{}\nAxisY:{}\nButtons:0x{:x}", LocationJoystick, ir.wAxisX, ir.wAxisY, ir.lButtons);
    let vj = unsafe {
        vJoyInterface::new("").unwrap() // TODO: error handling
    };

    unsafe {
        vj.UpdateVJD(iInterface, &mut ir as *mut _ as *mut JOYSTICK_POSITION_V2);
    }

    // SetForegroundWindow(h);
    // SetFocus(h);
}

pub fn Exit() {
    let vj = unsafe {
        vJoyInterface::new("").unwrap() // TODO: error handling
    };
	unsafe {
        vj.RelinquishVJD(iInterface);
    }
    defense_mind_input.lock().unwrap().exit = true; // TODO: error handling
    attack_mind_input.lock().unwrap().exit = true; // TODO: error handling
	unsafe {
        let ph = processHandle.lock().unwrap(); // TODO: error handling
        CloseHandle(*ph).unwrap(); // TODO: error handling
    }
    // guiClose();
}