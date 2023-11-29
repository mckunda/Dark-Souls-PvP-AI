//program to train, teach, and create neural net

use std::ffi::c_void;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::thread::sleep;
use std::time;
use const_format::concatcp;
use crate::fann::{ActivationFunc, ErrorFunc, Fann, RpropParams, StopFunc, TrainAlgorithm, TrainData};
use crate::fann_sys::{fann_errno_enum, fann_train_data, fann_type};
use lazy_static::lazy_static;
use libc::{c_char, c_uint, FILE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use rand::prelude::*;
use crate::ai::animation_mappings::isAttackAnimation;
use crate::ai::character::{Character, Enemy, EnemyId, Player, player_base_add, PlayerId, ReadPlayer};
use crate::ai::ffi::clock;
use crate::ai::helper_util::{angleDeltaFromFront, distance, rotationDifferenceFromSelf, StaminaEstimationEnemy};
use crate::ai::memory::{AIHPMemory, AIHPMemoryLENGTH, AppendDistance, DistanceMemory, DistanceMemoryLENGTH, last_animation_types_enemy, last_animation_types_enemy_LENGTH};
use crate::ai::memory_edits::FindPointerAddr;
use crate::ai::settings::{DisableAi, NeuralNetFolderLocation, TrainAttackNet, TrainBackstabNet};
use crate::ai::source::{MainLogicLoop, processHandle};
use crate::ai::weapon_data::PoiseDamageForAttack;

pub const TwoSecStoreLength: usize = 40;

lazy_static!(
    static ref TwoSecStore: Mutex<[Option<Character>;TwoSecStoreLength]> = Mutex::new([None;TwoSecStoreLength]);
);

lazy_static!(
    static ref lastCopyTime: Mutex<i64> = Mutex::new(0);
);
lazy_static!(
    static ref lastBsCheckTime: Mutex<i64> = Mutex::new(0);
);

static Player_AnimationId3_offsets: &'static [u64;Player_AnimationId3_offsets_length] = &[ 0x3C, 0x10C ];
pub const Player_AnimationId3_offsets_length: usize = 2;
lazy_static!(
    static ref AnimationId3_Addr: Mutex<u64> = Mutex::new(0);
);
lazy_static!(
    static ref AnimationId3: Mutex<i32> = Mutex::new(0);
);
static Player_Timer3_offsets: &'static [u64;Player_Timer3_offsets_length]  = &[ 0x3C, 0x28, 0x18, 0x7DC, 0x98 ];
pub const Player_Timer3_offsets_length: usize = 5;
lazy_static!(
    static ref Timer3_Addr: Mutex<u64> = Mutex::new(0);
);

lazy_static!(
    static ref Timer3: Mutex<f32> = Mutex::new(0f32);
);

static fpatk: &'static str = concatcp!(NeuralNetFolderLocation, "/attack_training_data.train");
static fpatk_test: &'static str = concatcp!(NeuralNetFolderLocation, "/attack_training_data.test");

static fpdef: &'static str = concatcp!(NeuralNetFolderLocation, "/backstab_training_data.train");
static fpdef_test: &'static str = concatcp!(NeuralNetFolderLocation, "/backstab_training_data.test");

pub unsafe fn GetTrainingData() {
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let ai3a = AnimationId3_Addr.lock().unwrap(); // TODO: error handling
    let mut ai3 = AnimationId3.lock().unwrap(); // TODO: error handling
    let mut t3a = Timer3_Addr.lock().unwrap(); // TODO: error handling
    let mut t3 = Timer3.lock().unwrap(); // TODO: error handling
    let mut lct = lastCopyTime.lock().unwrap(); // TODO: error handling
    let dm = DistanceMemory.lock().unwrap(); // TODO: error handling
    let ahm = AIHPMemory.lock().unwrap(); // TODO: error handling
    let mut tss = TwoSecStore.lock().unwrap(); // TODO: error handling
    let mut lbct = lastBsCheckTime.lock().unwrap(); // TODO: error handling
    let ph = processHandle.lock().unwrap(); // TODO: error handling
    let late = last_animation_types_enemy.lock().unwrap(); // TODO: error handling

    let mut rng = thread_rng();

    if DisableAi {
        ReadPlayer(&mut e, *ph, EnemyId);
        ReadPlayer(&mut p, *ph, PlayerId);
        AppendDistance(distance(&p, &e));
    } else {
        MainLogicLoop();
    }

    ReadProcessMemory(*ph, *ai3a as *const c_void, &mut ai3 as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: error handling
    ReadProcessMemory(*ph, *t3a as *const c_void, &mut t3 as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: error handling

    //store copy of player and enemy structs every 100 ms for 3.5 sec
    if clock() - *lct > 100 {
        // free(TwoSecStore[TwoSecStoreLength-1]);
        // free(TwoSecStore[TwoSecStoreLength-2]);
        for i in (0..TwoSecStoreLength).rev() {
            tss[i] = tss[i - 2];
        }
        tss[1] = Some(e.clone());
        tss[0] = Some(p.clone());

        *lct = clock();
    }

    //have random attacks. if it doesnt get hit, sucess. if it gets hit, fail.
    if isAttackAnimation(p.animationType_id) != 0 && dm[DistanceMemoryLENGTH-1] != 0. && TrainAttackNet != 0 {
        let (fpa, is_test) = match rng.gen_range(0..3) {
            0 => (fpatk_test, true),
            _ => (fpatk, true),
        };

        let mut f = File::open(fpa).unwrap(); // TODO: error handling

        let startingHp: u32 = p.hp;
        let startingHpEnemy: u32 = e.hp;

        //output the array of distance values
        for i in 0..DistanceMemoryLENGTH {
            f.write(format!("{} ", dm[i]).as_bytes()).unwrap(); // TODO: error handling
        }

        //output estimated stamina of enemy
        f.write(format!("{} ", StaminaEstimationEnemy() as f32).as_bytes()).unwrap(); // TODO: error handling
        //output the enemy's current poise
        f.write(format!("{} ", e.poise).as_bytes()).unwrap(); // TODO: error handling
        //output the AI's attack's poise damage (just r1 for now)
        f.write(format!("{} ", PoiseDamageForAttack(p.r_weapon_id, 46)).as_bytes()).unwrap(); // TODO: error handling
        //output the AI's current poise
        f.write(format!("{} ", p.poise).as_bytes()).unwrap(); // TODO: error handling
        //base poise damage of enemy's attack (treat r1 as base)
        f.write(format!("{} ", PoiseDamageForAttack(e.r_weapon_id, 46)).as_bytes()).unwrap(); // TODO: error handling
        //output array of AI's HP over time
        for i in 0..AIHPMemoryLENGTH {
            f.write(format!("{} ", ahm[i] as f32).as_bytes()).unwrap(); // TODO: error handling
        }
        //stamina of AI
        f.write(format!("{} ", p.stamina as f32).as_bytes()).unwrap(); // TODO: error handling
        //output array of enemy animation types
        for i in 0..last_animation_types_enemy_LENGTH {
            f.write(format!("{} ", late[i] as f32).as_bytes()).unwrap(); // TODO: error handling
        }
        //current bleed built up
        f.write(format!("{} ", p.bleedStatus as f32).as_bytes()).unwrap(); // TODO: error handling
        f.sync_all().unwrap(); // TODO: error handling
        //2 seconds
        let startTime = clock();
        while clock() - startTime < 2000 {
            MainLogicLoop();
        }

        let mut result: f32 = 0.;
        //bad outcome
        if startingHp != p.hp {
            result = -1.;
        }
        //neutral outcome
        else if startingHp == p.hp && startingHpEnemy == e.hp {
            result = 0.;
        }
        //good outcome
        else if startingHp == p.hp && startingHpEnemy != e.hp {
            result = 1.;
        }

        //output result
        f.write(format!("\n{}\n", result).as_bytes()).unwrap(); // TODO: error handiling
		println!("Attack result:{} in {}\n", result, if is_test { "Test" } else { "Train" });

        // TODO: enable and test
        // let resethp: u32 = 2000;

        //reset hp so we dont die
        // WriteProcessMemory(processHandle, p.hp_address as *mut c_void, &resethp as *const _ as *const c_void, 4, None).unwrap(); // TODO: error handling
    }

    let backstabCheckTime = clock() - *lbct > 3000;// RRAND(2500, 4000);

    //player in backstab state when animation id 3 is 9000, 9420
    if (((*ai3 == 9000 || *ai3 == 9420) && (*t3 < 0.1 && *t3 > 0.)) || (rng.gen::<i32>() < 1000 && backstabCheckTime)) && tss[TwoSecStoreLength-1].is_some() && TrainBackstabNet != 0 {
        let (fpd, is_test) = match rng.gen_range(0..3) {
            0 => (fpdef_test, true),
            _ => (fpdef, false),
        };

        let mut outFile = File::open(fpd).unwrap(); // TODO: error handling

        //output an array of 5 distance values from 3500 ms ago
        for i in TwoSecStoreLength-5..TwoSecStoreLength {
            outFile.write(format!("{} ", dm[i]).as_bytes()).unwrap(); // TODO: error handling
        }

        outFile.write(
            format!("{} {} {}\n{}\n",
                    angleDeltaFromFront(&tss[TwoSecStoreLength-6].unwrap(), &tss[TwoSecStoreLength - 7].unwrap()), // TODO: error handling
                    tss[TwoSecStoreLength-7].unwrap().velocity,
                    rotationDifferenceFromSelf(&tss[TwoSecStoreLength-6].unwrap(), &tss[TwoSecStoreLength-7].unwrap()),
                    match *ai3 { 9000 => 1.0, 9420 => 1.0, _ => -1.0 }
            ).as_bytes()).unwrap(); // TODO: error handling
		outFile.sync_all().unwrap(); // TODO: error handling
        println!("BackStab result:{} in {}\n", match *ai3 { 9000 => 1.0, 9420 => 1., _ => -1. }, if is_test { "Test" } else { "Train" });
        sleep(time::Duration::from_millis(100));
    }

    if backstabCheckTime {
        *lbct = clock();
    }
}

#[repr(C)]
pub struct FannTrainData {
    errno_f: fann_errno_enum,
    error_log: *mut FILE,
    errstr: *mut c_char,
    num_data: c_uint,
    num_input: c_uint,
    num_output: c_uint,
    pub input: *mut *mut fann_type,
    pub output: *mut *mut fann_type,
}

//use the file to train the network
pub fn trainFromFile(max_neurons: u32, training_file: &str, testing_file: &str, output_file: &str) {
    let desired_error: f32 = 0.05;
    let neurons_between_reports: u32 = 5;

    println!("Reading data.");

    let mut train_data = TrainData::from_file(training_file).unwrap(); // TODO: error handling
    let test_data = TrainData::from_file(testing_file).unwrap(); // TODO: error handling

    train_data.scale(-1., 1.).unwrap(); // TODO: error handling

    println!("Creating network.");
    println!("input number:{}, output number:{}", train_data.num_input(), train_data.num_output());

    let mut ann = Fann::new_shortcut(&[train_data.num_input(), train_data.num_output()]).unwrap(); // TODO: error handling
    ann.set_train_algorithm(TrainAlgorithm::Rprop(RpropParams {
        decrease_factor: 0.5,
        increase_factor: 1.2,
        delta_min: 0.0,
        delta_max: 50.0,
        delta_zero: 0.1,
    }));

    ann.set_activation_func_hidden(ActivationFunc::SigmoidSymmetric);
    ann.set_activation_func_output(ActivationFunc::Linear);
    ann.set_error_func(ErrorFunc::Linear);
    ann.set_bit_fail_limit(0.9);
    ann.set_stop_func(StopFunc::Bit);
    ann.print_parameters();

    println!("Training network.");
    let mut trainer = ann
        .on_data(&train_data)
        .cascade()
        .with_reports(neurons_between_reports);

    trainer.train(max_neurons, desired_error).unwrap(); // TODO: error handling
    ann.print_connections();

    let mse_train: f32 = ann.test_data(&train_data).unwrap(); // TODO: error handling
    let bit_fail_train: u32 = ann.get_bit_fail();
    let mse_test: f32 = ann.test_data(&test_data).unwrap(); // TODO: error handling
    let bit_fail_test: u32 = ann.get_bit_fail();

    println!("\nTrain error: {}, Train bit-fail: {}, Test error: {}, Test bit-fail: {}\n",
             mse_train, bit_fail_train, mse_test, bit_fail_test);

    let data_input = unsafe {
        let a = (&mut *train_data.get_raw() as &mut _ as &mut fann_train_data).input;
        std::slice::from_raw_parts(a, train_data.length() as usize)
    };

    let data_output = unsafe {
        let a = (&mut *train_data.get_raw() as &mut _ as &mut fann_train_data).output;
        std::slice::from_raw_parts(a, train_data.length() as usize)
    };

    for i in 0..(train_data.length() as usize) {
        let output = unsafe {
            ann.run(std::slice::from_raw_parts(data_input[i], train_data.num_input() as usize)).unwrap() // TODO: error handling
        };

        let o1 = unsafe {
            std::slice::from_raw_parts(data_output[i], train_data.num_output() as usize)[0]
        };

        if  (o1 - output[0]).abs() > 5.
            || o1 >= 0. && output[0] <= 0.
            || o1 <= 0. && output[0] >= 0.
        {
            println!("ERROR: {} does not match {}", o1, output[0]);
        }
    }

    println!("Saving network.");
    ann.save(output_file).unwrap(); // TODO: error handling

    println!("Cleaning up.");
}


pub unsafe fn SetupTraining() {
    let ph = processHandle.lock().unwrap(); // TODO: error handling
    let mut ai3a = AnimationId3_Addr.lock().unwrap(); // TODO: error handling
    let mut t3a = Timer3_Addr.lock().unwrap(); // TODO: error handling
    let pba = player_base_add.lock().unwrap(); // TODO: error handling
    *ai3a = FindPointerAddr(*ph, *pba, Player_AnimationId3_offsets_length, Player_AnimationId3_offsets);
    *t3a = FindPointerAddr(*ph, *pba, Player_Timer3_offsets_length, Player_Timer3_offsets);
}