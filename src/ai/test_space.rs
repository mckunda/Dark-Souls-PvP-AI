use std::error::Error;
use std::ffi::c_void;
use std::fs::File;
use std::io::{Read, Write};
use std::process::ExitCode;
use std::sync::Mutex;
use std::thread;
use lazy_static::lazy_static;
use windows::core::imp::CloseHandle;
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use crate::ai::character::{Player, PlayerId, ReadPlayer};
use crate::ai::source::{processHandle, SetupandLoad};
lazy_static!(
    static ref listening1: Mutex<bool> = Mutex::new(true);
);

fn ListentoContinue1() -> i32 {
    println!("e to exit");

    loop {
        let mut l1 = listening1.lock().unwrap(); // TODO: error handling
        if !*l1 {
            break;
        }

        let mut input = [0u8;1];
        std::io::stdin().read_exact(&mut input).unwrap(); // TODO: error handling
        let input = input[0] as char;
        if input == 'e' {
            //exit
            *l1 = false;
        }
    }

    return 0;
}

// FILE* fpdef;

fn DumpStaminaMem() -> Result<(), Box<dyn Error>> {
    // 99872 / 4 = 24968
    let mut staminaarray = &[0i32;24968];
    let ph = processHandle.lock().unwrap(); // TODO: error handling
    unsafe {
        ReadProcessMemory(*ph, 0x03B5D4A8 as *const c_void, &mut staminaarray as *mut _ as *mut c_void, 99872, None).unwrap(); // TODO: error handling
    }

    let mut fpdef = File::open("E:/Code Workspace/Dark Souls AI C/out.txt")?;
    for i in (0..staminaarray.len()).step_by(2) {
        let animType = staminaarray[i + 1];
        if animType != 40 && animType != 50 &&
            animType != 240 && animType != 250 &&
            animType != 400 &&
            animType != 430 && animType != 440 && animType != 450 && animType != 490 &&
            animType != 505 && animType != 510 && animType != 515 && animType != 516 &&
            animType != 600
        {
            fpdef.write(format!("{},{},{}\n", staminaarray[i + 0], animType, staminaarray[i + 5]).as_bytes()).unwrap(); // TODO: error handling
        }
    }

    fpdef.sync_all()?;
    Ok(())
}

lazy_static!(
    static ref lastAid: Mutex<i32> = Mutex::new(0);
);

lazy_static!(
    static ref curAid: Mutex<i32> = Mutex::new(0);
);

fn ReadWeaponTiming() {
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let ph = processHandle.lock().unwrap(); // TODO: error handling
    let mut ca = curAid.lock().unwrap(); // TODO: error handling
    let mut la = lastAid.lock().unwrap(); // TODO: error handling

    unsafe {
        ReadPlayer(&mut p, *ph, PlayerId)
    };

    let mut hurtbox: u8 = 0;
    unsafe {
        ReadProcessMemory(*ph, 0x06D70AC7 as *const c_void, &mut hurtbox as *mut _ as *mut c_void, 1, None).unwrap(); // TODO: error handling
    }
    unsafe {
        ReadProcessMemory(*ph, p.animationId_address as *const c_void, &mut ca as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: error handling
    }

    let mut timer: f32 = 0.;
    unsafe {
        ReadProcessMemory(*ph, 0x0707E17C as *const c_void, &mut timer as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: error handling
    }

    if hurtbox != 0 && *la != *ca {
        println!("{} {}\n", *ca, timer);
        *la = *ca;
    }
}

fn mainTESTING() -> Result<ExitCode, Box<dyn Error>> {
    SetupandLoad();

    thread::spawn(ListentoContinue1);
    // HANDLE thread = CreateThread(NULL, 0, ListentoContinue1, NULL, 0, NULL);

    loop {
        let l1 = listening1.lock();
        if l1.is_err() {
            println!("{}", l1.unwrap_err());
            continue;
        }

        let l1 = l1.unwrap();
        if !*l1 {
            break;
        }
    }


    match processHandle.lock() {
        Ok(p) => unsafe {
            CloseHandle(p.0);
        },
        Err(e) => {
            println!("{}", e);
        }
    };


    return Ok(ExitCode::SUCCESS);
}