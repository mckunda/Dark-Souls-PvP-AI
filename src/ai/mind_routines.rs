//Since the neural networks are threaded, they cannot interface with the primary logic directly.
//Neural network thinking processes are called, and when they return they set flags here marking their desicion
//These flags are then read in the primary logic
//This is done to reduce delay by keeping immediate actions, and allowing complex actions to still be used, hopeful before they become outdated

use crate::ai::gui::LocationDetection;
use std::sync::Mutex;
use std::thread;
use const_format::concatcp;
use crate::fann::{Fann, FannType};
use lazy_static::lazy_static;
use libc::{rand, RAND_MAX};
use crate::ai::character::{Enemy, Player, SubanimationNeutral};
use crate::ai::guiPrint;
use crate::ai::helper_util::{angleDeltaFromFront, BackstabDetection, distance, rotationDifferenceFromSelf, StaminaEstimationEnemy};
use crate::ai::memory::{AIHPMemory, AIHPMemoryLENGTH, DistanceMemory, DistanceMemoryLENGTH, last_animation_types_enemy, last_animation_types_enemy_LENGTH};
use crate::ai::settings::{BackstabMetaOnly, NeuralNetFolderLocation};
use crate::ai::sub_routines::{AttackId, DefenseId};
use crate::ai::weapon_data::PoiseDamageForAttack;
use crate::constants::LockInSubanimation;

pub struct MindInput {
    pub mind: Option<Fann>,
    pub exit: bool,
    pub crit: i32,
    pub cond: i32,
    pub runNetwork: bool,
}

lazy_static!(
    pub static ref defense_mind_input: Mutex<MindInput> = Mutex::new(MindInput {
        mind: None,
        exit: false,
        crit: 0,
        cond: 0,
        runNetwork: false,
    });
);
lazy_static!(
    pub static ref DefenseChoice: Mutex<u8> = Mutex::new(0);
);
lazy_static!(
    pub static ref attack_mind_input: Mutex<MindInput> = Mutex::new(MindInput {
        mind: None,
        exit: false,
        crit: 0,
        cond: 0,
        runNetwork: false,
    });
);
lazy_static!(
    pub static ref AttackChoice: Mutex<u8> = Mutex::new(0);
);

fn SCALE(input: f32, minVal: f32, maxVal: f32) -> f32 {
    2. * (input - minVal) / (maxVal - minVal) - 2.
}

unsafe fn DefenseMindProcess() {
    let mut dmi = defense_mind_input.lock().unwrap(); // TODO: error handling
    let dm = DistanceMemory.lock().unwrap(); // TODO: error handling
    let mut ac = AttackChoice.lock().unwrap(); // TODO: error handling
    let mut dc = DefenseChoice.lock().unwrap(); // TODO: error handling
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling

    while !dmi.exit {
        //lock control of this resource
        // EnterCriticalSection(&(defense_mind_input->crit));
        //wait for the indicator this should run, and release lock in meantime
        while dmi.runNetwork == false {
            // SleepConditionVariableCS(&(defense_mind_input->cond), &(defense_mind_input->crit), INFINITE);
        }

        //generate inputs and scale from -1 to 1
        let input: &mut [FannType;8] = &mut [0.; 8];

        //copy inputs into input and scale
        let mostRecentDistance: f32 = distance(&p, &e);
        input[0] = SCALE(mostRecentDistance, 0., 10.);
        input[0] = if input[0] > 1. { 1. } else { input[0] };
        input[0] = if input[0] < -1. { -1. } else { input[0] };
        for i in 0..4 {
            input[i+1] = SCALE(dm[i], 0., 10.);
            //cut off above and below
            input[i+1] = if input[i+1] > 1. { 1. } else { input[i+1] };
            input[i+1] = if input[i+1] < -1. { -1. } else { input[i+1] };
        }
        input[5] = SCALE(angleDeltaFromFront(&p, &e), 0., 1.6);
        input[6] = SCALE(e.velocity, -0.18, -0.04);
        input[7] = SCALE(rotationDifferenceFromSelf(&p, &e), 0., 3.8);

        let out = dmi.mind.as_ref().unwrap().run(input).unwrap(); // TODO: error handling
        //printf("%f\n", *out);

        //backstab attempt detection and avoidance
        //TODO implement more types of backstab avoidance actions
        if out[0] < 10. && out[0] > 0.5
            && mostRecentDistance < 5. //hardcode bs distance
            && e.subanimation == SubanimationNeutral //enemy cant backstab when in animation
            //&& BackstabDetection(&Enemy, &Player, mostRecentDistance) == 0 //can't be backstabed when behind enemy
        {
            //TODO make this strafe in the same direction as the enemy strafe
            *dc = DefenseId::CounterStrafeLeftId as u8;
        }

        //if we're waking up from a bs, try to avoid chain
        if p.in_backstab != 0 {
            if rand() > libc::RAND_MAX / 2 {
                //randomly choose between chain escapes to through off predictions
                *dc = DefenseId::OmnistepBackwardsId as u8;
            } else {
                *dc = DefenseId::ReverseRollBSId as u8;
            }

        }

        //if the enemy is close behind us, and there's no possibilty of chain(which a bs cancel can't prevent) try to damage cancel their bs.
        if BackstabDetection(&mut e, &mut p, mostRecentDistance) != 0 && p.in_backstab == 0 && e.in_backstab == 0 {
            *ac = AttackId::GhostHitId as u8;
        }

        //prevent rerun
        dmi.runNetwork = false;
        //release lock
        // LeaveCriticalSection(&(defense_mind_input->crit));
        // WakeConditionVariable(&(defense_mind_input->cond));
    }

    // return 0;
}

unsafe fn AttackMindProcess() {
    let p = Player.lock().unwrap(); // TODO: error handling
    let e = Enemy.lock();
    let e = e.borrow_mut(); // TODO: error handling

    let mut ami = attack_mind_input.lock().unwrap(); // TODO: error handling
    let mut ac = AttackChoice.lock().unwrap(); // TODO: error handling
    let ahm = AIHPMemory.lock().unwrap(); // TODO: error handling
    let late = last_animation_types_enemy.lock().unwrap(); // TODO: error handling
    let dm = DistanceMemory.lock().unwrap(); // TODO: error handling

    while !ami.exit
    {
        //lock control of this resource
        // EnterCriticalSection(&(attack_mind_input->crit));
        //wait for the indicator this should run, and release lock in meantime
        while ami.runNetwork == false {
            // SleepConditionVariableCS(&(attack_mind_input->cond), &(attack_mind_input->crit), INFINITE);
        }

        //generate inputs and scale from -1 to 1
        let input = &mut [0 as FannType;DistanceMemoryLENGTH + 5 + AIHPMemoryLENGTH + 1 + last_animation_types_enemy_LENGTH + 1];

        //copy inputs into input and scale
        let mostRecentDistance: f32 = distance(&p, &e);
        input[0] = SCALE(mostRecentDistance, 0., 10.);
        input[0] = if input[0] > 1. { 1. } else { input[0] };
        input[0] = if input[0] < -1. { -1. } else { input[0] };
        for i in 0..DistanceMemoryLENGTH {
            input[i+1] = SCALE(dm[i], 0., 10.);
            //cut off above and below
            input[i+1] = if input[i+1] > 1. { 1. } else { input[i+1] };
            input[i+1] = if input[i+1] < -1. { -1. } else { input[i+1] };
        }

        input[DistanceMemoryLENGTH] = SCALE(StaminaEstimationEnemy() as FannType, -40., 192.);
        input[DistanceMemoryLENGTH + 1] = SCALE(e.poise, 0., 120.);
        input[DistanceMemoryLENGTH + 2] = SCALE(PoiseDamageForAttack(p.r_weapon_id, 46), 0., 80.);
        input[DistanceMemoryLENGTH + 3] = SCALE(p.poise, 0., 120.);
        input[DistanceMemoryLENGTH + 4] = SCALE(PoiseDamageForAttack(e.r_weapon_id, 46), 0., 80.);

        for i in 0..AIHPMemoryLENGTH {
            input[i + DistanceMemoryLENGTH + 5] = SCALE(ahm[i] as FannType, 0., 2000.);
        }

        input[DistanceMemoryLENGTH + 5 + AIHPMemoryLENGTH] = SCALE(p.stamina as FannType, -40., 192.);

        for i in 0..last_animation_types_enemy_LENGTH {
            input[i + DistanceMemoryLENGTH + 5 + AIHPMemoryLENGTH + 1] = SCALE(late[i] as f32, 0., 255.);
        }

        input[DistanceMemoryLENGTH + 5 + AIHPMemoryLENGTH + 1 + last_animation_types_enemy_LENGTH] = SCALE(p.bleedStatus as FannType, 0., 255.);

        let out = ami.mind.as_ref().unwrap().run(input).unwrap(); // TODO: error handling

        //potentally move up if not in range
        if mostRecentDistance > p.weaponRange {
            *ac = AttackId::MoveUpId as u8;
        }

        //TODO desicion about going for a backstab. Note that these subroutines will attempt, not garuntee
        //AttackChoice = PivotBSId;

        //TODO chain bs's. if enemy in bs, try chain

        //Decision about standard attack
        if BackstabMetaOnly == 0 &&
            //sanity checks
            mostRecentDistance <= p.weaponRange && //in range
            p.stamina > 20 && //just to ensure we have enough to roll
            p.bleedStatus > 40 && //more than one attack to proc bleed
            //static checks for attack
            ((
                (p.stamina > 90) && //safety buffer for stamina
                (e.subanimation >= LockInSubanimation && e.subanimation < SubanimationNeutral)  //enemy in vulnerable state, and can't immediatly transition
            ) ||
                (out[0] > 0.5)//neural network says so
            )
        {
            //randomly choose dead angle or ghost hit
            //throw off enemy predictions
            if rand() > RAND_MAX / 2 {
                *ac = AttackId::DeadAngleId as u8;
            }
            else{
                *ac = AttackId::GhostHitId as u8;
            }
        }

        //prevent rerun
        ami.runNetwork = false;
        //release lock
        // LeaveCriticalSection(&(ami->crit));
        // WakeConditionVariable(&(ami->cond));
    }
    // return 0;
}

//Helper Methods

pub fn ReadyThreads() -> i32 {
    //Defense Thread
    let defense_mind = Fann::from_file(concatcp!(NeuralNetFolderLocation, "/Defense_dark_souls_ai.net")); // TODO: error handling

    let defense_mind = match defense_mind {
        Ok(d) => d,
        Err(_) => {
            println!("Defense_dark_souls_ai.net neural network file not found");
            return -1
        }
    };

    let mut dmi = defense_mind_input.lock().unwrap(); // TODO: error handling
    dmi.mind = Some(defense_mind);

    // InitializeConditionVariable(&(defense_mind_input->cond));
    // InitializeCriticalSection(&(defense_mind_input->crit));
    // EnterCriticalSection(&(defense_mind_input->crit));
    let defense_mind_thread = thread::spawn(|| unsafe {DefenseMindProcess();});

    //Attack Thread
    let attack_mind = Fann::from_file(concatcp!(NeuralNetFolderLocation, "/Attack_dark_souls_ai.net"));
    let attack_mind = match attack_mind {
        Ok(a) => a,
        Err(_) => {
            println!("Attack_dark_souls_ai.net neural network file not found");
            return -1;
        }
    };

    let mut ami = attack_mind_input.lock().unwrap(); // TODO: error handling
    ami.mind = Some(attack_mind);

    // InitializeConditionVariable(&(attack_mind_input->cond));
    // InitializeCriticalSection(&(attack_mind_input->crit));
    // EnterCriticalSection(&(attack_mind_input->crit));
    let attack_mind_thread = thread::spawn(|| unsafe { AttackMindProcess(); });

    return 0;
}

pub fn WaitForThread(input: &mut MindInput){
    //get control of lock
    // EnterCriticalSection(&(input->crit));
    //wait for neural net thread to mark self as finished
    while input.runNetwork {
        let result = false; //SleepConditionVariableCS(&(input->cond), &(input->crit), 10);
        if !result {
            guiPrint!("{},2:Timeout in reacquiring thread", LocationDetection);
            break;
        }
    }
}


pub fn WakeThread(input: &mut MindInput){
    //trigger threads to run
    input.runNetwork = true;
    //release lock
    // LeaveCriticalSection(&(input->crit));
    //wake thread
    // WakeConditionVariable(&(input->cond));
}