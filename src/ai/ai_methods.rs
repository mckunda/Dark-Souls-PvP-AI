use std::sync::Mutex;
use lazy_static::lazy_static;
use vjoy_sys::{JOYSTICK_POSITION, LONG};
use crate::ai::ai_decisions::{InstinctDecision, PriorityDecision};
use crate::ai::ai_decisions::SubroutineId::{attackid, defenseid};
use crate::ai::animation_mappings::isDodgeAnimation;
use crate::ai::character::{Enemy, Player, SubanimationNeutral};
use crate::ai::ffi::clock;
use crate::ai::gui::LocationState;
use crate::ai::guiPrint;
use crate::ai::helper_util::{angleFromCoordinates, angleToJoystick, BackstabDetection, circle, distance, dleft, l1, l2, longTuple, MIDDLE, r1, r3, square, triangle, XLEFT, XRIGHT, YBOTTOM};
use crate::ai::memory::{AppendLastSubroutineSelf, last_subroutine_states_self};
use crate::ai::sub_routines::{AttackId, AttackStateIndex, AttackTypeIndex, DefenseId, DodgeStateIndex, DodgeTypeIndex, inActiveAttackSubroutine, inActiveDodgeSubroutine, inActiveSubroutine, NoSubroutineActive, startTimeAttack, startTimeDefense, subroutine_states, SubroutineActive, SubroutineExiting};
use crate::constants::{AttackSubanimationWindupGhostHit, LockInSubanimation, PoiseBrokenSubanimation};

use super::character::SubanimationRecover;

pub const CLOCKS_PER_SEC: i32 = 1000000;
pub const TimeForR3ToTrigger: i64 = 50;
pub const TimeForCameraToRotateAfterLockon: i64 = 180;//how much time we give to allow the camera to rotate.
pub const TimeDeltaForGameRegisterAction: i64 = 170;
pub const TotalTimeInSectoReverseRoll: f32 = (TimeForR3ToTrigger + TimeForCameraToRotateAfterLockon + TimeDeltaForGameRegisterAction + 50) as f32 / (CLOCKS_PER_SEC as f32);//convert above CLOCKS_PER_SEC ticks to seconds


/* ------------- DODGE Actions ------------- */

pub unsafe fn StandardRoll(iReport: &mut JOYSTICK_POSITION) {
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let stde = startTimeDefense.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling
    let lsss = last_subroutine_states_self.lock().unwrap(); // TODO: error handling
    guiPrint!("{},0:dodge roll time:{}", LocationState, (curTime - *stde));

    //ensure we actually enter dodge roll in game so another subanimation cant override it
    //or we get poise broken out
    if p.subanimation == LockInSubanimation || p.subanimation == PoiseBrokenSubanimation || curTime > *stde + 900 {
        guiPrint!("{},0:end dodge roll", LocationState);
        ss[DodgeStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(DefenseId::StandardRollId as u8);
        return;
    }

    //roll
    //AUUUGH FUCK IT JUST HAMMER THE BUTTON
    if curTime < *stde + 50 ||
       (curTime > *stde + 100 && curTime < *stde + 150) ||
       (curTime > *stde + 200 && curTime < *stde + 250) ||
       (curTime > *stde + 300 && curTime < *stde + 350)
    {
        guiPrint!("{},1:circle", LocationState);
        iReport.lButtons = circle;
        //handle this subroutines intitation after a counterstrafe abort (handles being locked on)
        //this will cause this roll to occur in lockon state, but subroutine will exit without lockon. Nothing major
        if p.locked_on != 0 {
            iReport.lButtons += r3;
        }
    }

    //turning
    if curTime > *stde + 10 && curTime < *stde + 300 {
        let mut rollOffset: f64 = 90.0;
        //if we're behind enemy, but we have to roll, roll towards their back for potential backstab
		if BackstabDetection(&p, &e, distance(&p, &e)) == 1 {
            rollOffset = 0.;
        }
        //if we just rolled but have to roll again, ensure we roll away so we dont get caught in r1 spam
        else if lsss[0] == DefenseId::StandardRollId as u8 {
            rollOffset = 120.0;
        }
		//if we had to toggle escape, they're probably comboing. Roll away
		else if lsss[0] == DefenseId::ToggleEscapeId as u8 {
			rollOffset = 120.0;
		}

        let mut angle: f64 = angleFromCoordinates(p.loc_x, e.loc_x, p.loc_y, e.loc_y) - rollOffset;

        if angle < 0. {
            angle += 360.
        }//wrap around

        //angle joystick
		let mut jmove: longTuple = longTuple { x_axis: 0, y_axis: 0 };
		angleToJoystick(angle, &mut jmove);
        //Stupid bug with dark souls. Can only roll when one of these is very close to middle. Select whatever one is furthest
		let diffX: i64 = (jmove.x_axis - MIDDLE as i64).abs();
		let diffY: i64 = (jmove.y_axis - MIDDLE as i64).abs();
        if diffX > diffY {
			iReport.wAxisX = jmove.x_axis as LONG;
        } else{
			iReport.wAxisY = jmove.y_axis as LONG;
        }

        guiPrint!("{},1:offset angle {} angle roll {}", LocationState, rollOffset, angle);
    }
}

pub const inputDelayForStopCircle: i64 = 40;

pub unsafe fn Backstep(iReport: &mut JOYSTICK_POSITION){
    guiPrint!("{},0:Backstep", LocationState);
    let curTime = clock();
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;
    let stde = startTimeDefense.lock().unwrap(); // TODO: error handling;

    if curTime < *stde + inputDelayForStopCircle {
        iReport.lButtons = circle;
    }

    if  curTime > *stde + inputDelayForStopCircle// &&
        // if we've compleated the dodge move and we're in animation end state we can end
        // (Player->subanimation == SubanimationRecover)// ||
        // or we end if not in dodge type animation id, because we could get hit out of dodge subroutine
        // !isDodgeAnimation(Player->animation_id))
        {
        guiPrint!("{},0:end backstep", LocationState);
        ss[DodgeStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(DefenseId::BackstepId as u8);
    }
}

pub const inputDelayForOmnistepWait: i64 = 40;
pub const inputDelayForStopOmnistepJoystickDirection: i64 = 40;

pub unsafe fn Omnistep_Backwards(iReport: &mut JOYSTICK_POSITION){
	guiPrint!("{},0:Omnistep Backwards", LocationState);
    let curTime: i64 = clock();
    let p = Player.lock().unwrap(); // TODO: error handling
    let e = Enemy.lock();
    let e = e.borrow(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;
    let stde = startTimeDefense.lock().unwrap(); // TODO: error handling;


    if curTime < *stde + inputDelayForStopCircle {
		iReport.lButtons = circle;
	}
	//TODO kind of not working
	else if curTime > *stde + inputDelayForStopCircle + inputDelayForOmnistepWait && curTime < *stde + inputDelayForStopCircle + inputDelayForOmnistepWait + inputDelayForStopOmnistepJoystickDirection {
		let angle: f64 = angleFromCoordinates(p.loc_x, e.loc_x, p.loc_y, e.loc_y);
		//angle joystick
		let mut jmove = longTuple {
            x_axis: 0,
            y_axis: 0,
        };
		angleToJoystick(angle, &mut jmove);
		iReport.wAxisX = jmove.x_axis as LONG;
		iReport.wAxisY = jmove.y_axis as LONG;
	}
	else{
		guiPrint!("{},0:end Omnistep Backwards", LocationState);
		ss[DodgeStateIndex] = SubroutineExiting;
		AppendLastSubroutineSelf(DefenseId::OmnistepBackwardsId as u8);
	}
}

pub const inputDelayForStopStrafe: i64 = 800;

pub unsafe fn CounterStrafe(iReport: &mut JOYSTICK_POSITION, left_strafe: bool){
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;
    let stde = startTimeDefense.lock().unwrap(); // TODO: error handling;

    guiPrint!("{},0:CounterStrafe:{}", LocationState, (curTime - *stde));
    let distanceBt: f32 = distance(&p, &e);

    //have to lock on to strafe
    if curTime < *stde + 30 {
        iReport.lButtons = r3;
        guiPrint!("{},1:lockon cs", LocationState);
    }
    //need a delay for dark souls to respond
    else if curTime < *stde + 60 {
        iReport.lButtons = 0;
    }

    //keep going if we're behind enemy or very close to them: might get a bs
    else if curTime < *stde + inputDelayForStopStrafe || BackstabDetection(&mut p, &mut e, distanceBt) == 1 || distanceBt < 1.3 {
		if left_strafe {
			iReport.wAxisX = XLEFT;
		}else{
			iReport.wAxisX = XRIGHT;
		}
        iReport.wAxisY = MIDDLE / 2;//3/4 pushed up
        guiPrint!("{},1:strafe", LocationState);
    }

    //disable lockon
    else if curTime < *stde + inputDelayForStopStrafe + 30 {
        iReport.lButtons = r3;
        guiPrint!("{},1:un lockon", LocationState);
    }
    else if curTime < *stde + inputDelayForStopStrafe + 60 {
        iReport.lButtons = 0;
    }

    else{
        guiPrint!("{},0:end CounterStrafe", LocationState);
        ss[DodgeStateIndex] = SubroutineExiting;
		if left_strafe {
			AppendLastSubroutineSelf(DefenseId::CounterStrafeLeftId as u8);
		}else{
			AppendLastSubroutineSelf(DefenseId::CounterStrafeRightId as u8);
		}
    }

    //break early if we didnt lock on
    if p.locked_on == 0 && curTime > *stde + 60 {
        guiPrint!("{},0:end CounterStrafe", LocationState);
        ss[DodgeStateIndex] = SubroutineExiting;
		if left_strafe {
			AppendLastSubroutineSelf(DefenseId::CounterStrafeLeftId as u8);
		}else{
			AppendLastSubroutineSelf(DefenseId::CounterStrafeRightId as u8);
		}
	}
}

pub unsafe fn L1Attack(iReport: &mut JOYSTICK_POSITION){
    guiPrint!("{},0:L1", LocationState);
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;
    let stde = startTimeDefense.lock().unwrap(); // TODO: error handling;


    if curTime < *stde + 30 {
        let angle: f64 = angleFromCoordinates(p.loc_x, e.loc_x, p.loc_y, e.loc_y);
		let mut jmove = longTuple {
            x_axis: 0,
            y_axis: 0,
        };
		angleToJoystick(angle, &mut jmove);
		iReport.wAxisX = jmove.x_axis as LONG;
		iReport.wAxisY = jmove.y_axis as LONG;
        iReport.lButtons = l1;
    }

    if curTime > *stde + 30 {
        guiPrint!("{},0:end L1", LocationState);
        ss[DodgeStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(DefenseId::L1AttackId as u8);
    }
}

//reverse roll through enemy attack and roll behind their back
pub unsafe fn ReverseRollBS(iReport: &mut JOYSTICK_POSITION){
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let stde = startTimeDefense.lock().unwrap(); // TODO: error handling
    guiPrint!("{},0:Reverse Roll BS time:{}", LocationState, (curTime - *stde));

    //have to lock on to reverse roll (also handle for being locked on already)
    if curTime < *stde + TimeForR3ToTrigger && p.locked_on == 0 {
        iReport.lButtons = r3;
        guiPrint!("{},1:lockon rrbs", LocationState);
    }

    //backwards then circle to roll and omnistep via delockon
    if curTime > *stde + TimeForR3ToTrigger + TimeForCameraToRotateAfterLockon &&
        curTime < *stde + TimeForR3ToTrigger + TimeForCameraToRotateAfterLockon + TimeDeltaForGameRegisterAction {
        iReport.wAxisY = YBOTTOM;//have to do this because reverse roll is impossible on non normal camera angles
        iReport.lButtons = r3 + circle;
        guiPrint!("{},1:reverse roll", LocationState);
    }

    if curTime > *stde + TimeForR3ToTrigger + TimeForCameraToRotateAfterLockon + TimeDeltaForGameRegisterAction
    {
        guiPrint!("{},0:end ReverseRollBS", LocationState);
        let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling
        ss[DodgeStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(DefenseId::ReverseRollBSId as u8);
    }
}

//this is more of a bandaid to the fact that the ai is ever getting hit
pub unsafe fn ToggleEscape(iReport: &mut JOYSTICK_POSITION) {
    let curTime = clock();
    let stde = startTimeDefense.lock().unwrap(); // TODO: error handling
    guiPrint!("{},0:Toggle Escape:{}", LocationState, (curTime - *stde));

    if curTime < *stde + 30 {
        iReport.bHats = dleft;
    }

    if curTime > *stde + 60 {
        guiPrint!("{},0:end Toggle Escape", LocationState);
        let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling
        ss[DodgeStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(DefenseId::ToggleEscapeId as u8);
    }
}

pub unsafe fn PerfectBlock(iReport: &mut JOYSTICK_POSITION){
    guiPrint!("{},0:Perfect Block", LocationState);
    let stde = startTimeDefense.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling
    let curTime = clock();

    if curTime < *stde + 30 {
        iReport.lButtons = l1;
    }

    if curTime > *stde + 60 {
        guiPrint!("{},0:end Perfect Block", LocationState);
        ss[DodgeStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(DefenseId::PerfectBlockId as u8);
    }
}

pub unsafe fn ParrySubroutine(iReport: &mut JOYSTICK_POSITION){
    guiPrint!("{},0:Parry", LocationState);
    let curTime = clock();
    let stde = startTimeDefense.lock().unwrap(); // TODO: error handling

    if curTime < *stde + 30 {
        iReport.lButtons = l2;
    }

    if curTime > *stde + 60 {
        guiPrint!("{},0:end Parry", LocationState);
        let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling
        ss[DodgeStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(DefenseId::ParryId as u8);
    }
}


//enter or continue a dodge subroutine
//this reconciles the MindRoutine and AiDecision choices
//makes deeper decision about what action to take (type of dodge)
pub unsafe fn dodge(iReport: &mut JOYSTICK_POSITION, instinct_decision: &mut InstinctDecision, DefenseNeuralNetChoice: u8) {
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling
	//if we're not in active subroutine and we can enter one
    if !inActiveSubroutine() && p.subanimation >= LockInSubanimation {
		//instinct overides AiMethods, have to do immediate dodge
		if instinct_decision.priority_decision == PriorityDecision::EnterDodgeSubroutine {
            if let defenseid(def) = instinct_decision.subroutine_id.clone() {
                ss[DodgeTypeIndex] = def as u8;
            } else { panic!() };
		}
		//AiMethod defines less immediate dodges
		else{
			ss[DodgeTypeIndex] = DefenseNeuralNetChoice;
        }

		ss[DodgeStateIndex] = SubroutineActive;
		//set time for this subroutine
		let mut stde = startTimeDefense.lock().unwrap(); // TODO: error handling
        *stde = clock();
	}

    if inActiveDodgeSubroutine() {
        match ss[DodgeTypeIndex].try_into() {
            Ok(DefenseId::StandardRollId) => {
                StandardRoll(iReport);
            },
            Ok(DefenseId::BackstepId) => {
                Backstep(iReport);
            },
            Ok(DefenseId::BackstepId) => {
                Omnistep_Backwards(iReport);
            },
            Ok(DefenseId::CounterStrafeLeftId) => {
                CounterStrafe(iReport, true);
            },
            Ok(DefenseId::CounterStrafeRightId) => {
                CounterStrafe(iReport, false);
            },
            Ok(DefenseId::L1AttackId) => {
                L1Attack(iReport);
            },
            Ok(DefenseId::ReverseRollBSId) => {
                ReverseRollBS(iReport);
            },
            Ok(DefenseId::ToggleEscapeId) => {
                ToggleEscape(iReport);
            },
            Ok(DefenseId::PerfectBlockId) => {
                PerfectBlock(iReport);
            },
			Ok(DefenseId::ParryId) => {
                ParrySubroutine(iReport);
            },
            //may not do anything even though attack detected (ex we're staggered)
            _ => {
                ss[DodgeStateIndex] = NoSubroutineActive;
            }
        }
    }
}

/* ------------- ATTACK Actions ------------- */

pub const inputDelayForStart: i64 = 10;//if we exit move forward and go into attack, need this to prevent kick
pub const inputDelayForKick: i64 = 50;

pub unsafe fn ghostHit(iReport: &mut JOYSTICK_POSITION){
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let sta = startTimeAttack.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;

    guiPrint!("{},0:ghost hit time:{}", LocationState, (curTime - *sta));

    let mut angle = angleFromCoordinates(p.loc_x, e.loc_x, p.loc_y, e.loc_y);

    //handle entering with lockon
    if p.locked_on != 0 && curTime < *sta + inputDelayForKick {
        iReport.lButtons += r3;
    }

    //hold attack button for a bit
    if (curTime < *sta + inputDelayForKick) && (curTime > *sta + inputDelayForStart) {
        guiPrint!("{},1:r1", LocationState);
        iReport.lButtons += r1;
    }

    //start rotate back to enemy
    if p.subanimation == AttackSubanimationWindupGhostHit {
        guiPrint!("{},1:towards", LocationState);
		let mut jmove = longTuple {
            x_axis: 0,
            y_axis: 0,
        };
		angleToJoystick(angle,&mut jmove);
		iReport.wAxisX = jmove.x_axis as LONG;
		iReport.wAxisY = jmove.y_axis as LONG;
    }

	//cant angle joystick immediatly, at first couple frames this will register as kick
    //after timeout, point away from enemy till end of windup
    else if curTime > *sta + inputDelayForKick {
        guiPrint!("{},1:away", LocationState);
        angle = (180.0 + angle) % 360.0;
		let mut jmove = longTuple {
            x_axis: 0,
            y_axis: 0,
        };
		angleToJoystick(angle,&mut jmove);
		iReport.wAxisX = jmove.x_axis as LONG;
		iReport.wAxisY = jmove.y_axis as LONG;
	}

	//end subanimation on recover animation
    if (curTime > *sta + 500) &&
    (p.subanimation > AttackSubanimationWindupGhostHit) {
        guiPrint!("{},0:end sub ghost hit", LocationState);
        ss[AttackStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(AttackId::GhostHitId as u8);
	}
}

pub unsafe fn deadAngle(iReport: &mut JOYSTICK_POSITION){
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let mut sta = startTimeAttack.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;

    guiPrint!("{},0:sub dead angle time:{}", LocationState, (curTime - *sta));

    let mut angle: f64 = angleFromCoordinates(p.loc_x, e.loc_x, p.loc_y, e.loc_y);

    //handle entering with lock-on
    if p.locked_on != 0 && curTime < *sta + inputDelayForKick {
        iReport.lButtons += r3;
    }

    //if we enter from a roll, move to enter neutral animation so we don't kick
    if isDodgeAnimation(p.animationType_id as u16) {
		let mut jmove = longTuple {
            x_axis: 0,
            y_axis: 0,
        };
		angleToJoystick(angle,&mut jmove);
		iReport.wAxisX = jmove.x_axis as LONG;
		iReport.wAxisY = jmove.y_axis as LONG;
        *sta = curTime;//reset start time when we exit dodge, so we know how long to hold buttons for
    }
    //hold attack button for a bit
    else if curTime < *sta + inputDelayForKick {
        guiPrint!("{},1:r1", LocationState);
        iReport.lButtons += r1;
    }
    //point X degreees off angle from directly towards enemy
    else if curTime > *sta + inputDelayForKick {
        guiPrint!("{},1:angle towards enemy: {}", LocationState, angle);
        angle = -60.0 + angle;
        if angle > 360. {
            angle = angle - 360.;
        }

		let mut jmove = longTuple {
            x_axis: 0,
            y_axis: 0,
        };
		angleToJoystick(angle,&mut jmove);
		iReport.wAxisX = jmove.x_axis as LONG;
		iReport.wAxisY = jmove.y_axis as LONG;
    }

    if curTime > *sta + 500 {
        guiPrint!("{},0:end sub dead angle", LocationState);
        ss[AttackStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(AttackId::GhostHitId as u8);
    }
}

lazy_static!(
    static ref startTimeHasntBeenReset: Mutex<bool> = Mutex::new(true);
);

pub unsafe fn backStab(iReport: &mut JOYSTICK_POSITION){
    guiPrint!("{},0:backstab", LocationState);
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let mut notReset = startTimeHasntBeenReset.lock().unwrap(); // TODO: error handling
    let mut sta = startTimeAttack.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;


    //backstabs cannot be triggerd from queued action
    //move character towards enemy back to switch to neutral animation as soon as in ready state
    let angle: f64 = angleFromCoordinates(p.loc_x, e.loc_x, p.loc_y, e.loc_y);
	let mut jmove = longTuple {
        x_axis: 0,
        y_axis: 0,
    };
	angleToJoystick(angle,&mut jmove);
	iReport.wAxisX = jmove.x_axis as LONG;
	iReport.wAxisY = jmove.y_axis as LONG;

    //once backstab is possible (neutral), press r1
    if p.subanimation == SubanimationNeutral {
        iReport.lButtons = r1;
        if *notReset {
            *sta = curTime; //reset start time to allow exit timeout
            *notReset = false;
        }
    }

    //end subanimation after too long moving, or too long holding r1
    //exit if we either got the bs, or we incorrectly detected it
    if curTime > *sta + 100 {
        *notReset = true;
        guiPrint!("{},0:end backstab", LocationState);
        ss[AttackStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(AttackId::BackstabId as u8);
    }
}

pub const inputDelayForStopMove: i64 = 90;

pub unsafe fn MoveUp(iReport: &mut JOYSTICK_POSITION){
    //if we are not close enough, move towards 
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let sta = startTimeAttack.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;

    guiPrint!("{},0:move up time:{}", LocationState, (curTime - *sta));

    if p.locked_on != 0 && curTime < *sta + 40 {
        iReport.lButtons = r3;
    }

    if curTime < *sta + inputDelayForStopMove {
		let mut jmove = longTuple {
            x_axis: 0,
            y_axis: 0,
        };
		angleToJoystick(angleFromCoordinates(p.loc_x, e.loc_x, p.loc_y, e.loc_y),&mut jmove);
		iReport.wAxisX = jmove.x_axis as LONG;
		iReport.wAxisY = jmove.y_axis as LONG;
    }

    if curTime > *sta + inputDelayForStopMove {
        guiPrint!("{},0:end sub move up", LocationState);
        ss[AttackStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(AttackId::MoveUpId as u8);
    }
}

pub unsafe fn twoHand(iReport: &mut JOYSTICK_POSITION){
    let curTime = clock();
    let sta = startTimeAttack.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;

    guiPrint!("{},0:two hand time:{}", LocationState, (curTime - *sta));

    if curTime < *sta + 40 {
        iReport.lButtons = triangle;
    }

    if curTime > *sta + 40 {
        guiPrint!("{},0:end two hand", LocationState);
        ss[AttackStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(AttackId::TwoHandId as u8);
    }
}

//lock on roll back to keep distance: prevent bs's, attacks 
pub unsafe fn SwitchWeapon(iReport: &mut JOYSTICK_POSITION){
    guiPrint!("{},0:Switch Weapon", LocationState);
    let curTime = clock();
    let sta = startTimeAttack.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;

    if curTime < *sta + 30 {
        iReport.lButtons = r3;
    }
    else if curTime < *sta + 300 {
        iReport.wAxisY = YBOTTOM;
        iReport.bHats = dleft;
    }

    if curTime > *sta + 500 {
        guiPrint!("{},0:end Switch Weapon", LocationState);
        ss[AttackStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(AttackId::SwitchWeaponId as u8);
    }
}

pub unsafe fn Heal(iReport: &mut JOYSTICK_POSITION){
    guiPrint!("{},0:Heal", LocationState);
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;
    let mut sta = startTimeAttack.lock().unwrap(); // TODO: error handling;

    if curTime < *sta + 30 {
        iReport.lButtons = square;
    }

    //sometimes game doesnt register the heal. retry till it does.
    if p.subanimation != LockInSubanimation && curTime > *sta + 100 {
        *sta = curTime;
    }

    //1830 ms to use db
    if curTime > *sta + 1830 {
        guiPrint!("{},0:end Heal", LocationState);
        ss[AttackStateIndex] = SubroutineExiting;
        AppendLastSubroutineSelf(AttackId::HealId as u8);
    }
}

lazy_static!(
    pub static ref StartingPivotAngle: Mutex<f64> = Mutex::new(-1.);
);

lazy_static!(
    pub static ref BehindStartTime: Mutex<i64> = Mutex::new(-1);
);

pub unsafe fn PivotBS(iReport: &mut JOYSTICK_POSITION){
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut e = Enemy.lock();
    let mut e = e.borrow_mut(); // TODO: error handling
    let mut spa = StartingPivotAngle.lock().unwrap(); // TODO: error handling
    let sta = startTimeAttack.lock().unwrap(); // TODO: error handling
	guiPrint!("{},0:Pivot BS Time:{}", LocationState, (curTime - *sta));

	let dist: f32 = distance(&p, &e);
	let bsState: u8 = BackstabDetection(&mut p, &mut e, dist);

	//de lock on if locked on
	if p.locked_on != 0 && curTime < *sta + 40 {
		iReport.lButtons = r3;
	}

	//sprint up while in front of enemy
	if bsState == 0 {
		iReport.lButtons += circle;

		//save the starting angle so we dont constantly reangle
		if *spa == -1f64 {
			*spa = angleFromCoordinates(p.loc_x, e.loc_x, p.loc_y, e.loc_y) - 10.;//run to their right
			if *spa < 0f64 {
                *spa += 360.
            }//wrap around
			guiPrint!("{},1:{}", LocationState, *spa);
		}

		let mut jmove = longTuple {
            x_axis: 0,
            y_axis: 0,
        };

		angleToJoystick(*spa, &mut jmove);
		iReport.wAxisX = jmove.x_axis as LONG;
		iReport.wAxisY = jmove.y_axis as LONG;
	}

	//when behind enemy (with enough space), reposition to face their back
	if bsState == 1 && dist > 1f32 {
        let mut bst = BehindStartTime.lock().unwrap(); // TODO: error handling
		if *bst == -1 {
			*bst = curTime;
		}

		let mut jmove = longTuple{
            x_axis: 0,
            y_axis: 0,
        };

		//to prevent skid from sudden angle change when we get behind enemy
		//decrement angle we're pointing at over time(degrees per ms) to smooth out transition
		let mut smoothingAngle = *spa + (curTime - *bst) as f64;
		if smoothingAngle < 0. {
            smoothingAngle += 360.;
        }

        angleToJoystick(smoothingAngle, &mut jmove);
		
        iReport.wAxisX = jmove.x_axis as LONG;
		iReport.wAxisY = jmove.y_axis as LONG;
	}

	//end when we got a backstab or backstab avoidance is triggered(set threshold) or ???
	//reset BehindStartTime and StartingPivotAngle
}

//enter or continue an attack subroutine
//this reconciles the MindRoutine and AiDecision choices
//makes deeper decision about what action to take (type of attack)
pub unsafe fn attack(iReport: &mut JOYSTICK_POSITION, instinct_decision: &InstinctDecision, AttackNeuralNetChoice: u8){
    let curTime = clock();
    let mut p = Player.lock().unwrap(); // TODO: error handling
    let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling;
    let mut sta = startTimeAttack.lock().unwrap(); // TODO: error handling;
	guiPrint!("{},0:Pivot BS Time:{}", LocationState, curTime - *sta);
	//proceed with subroutine if we are not in one already
	//special case for asyncronous backstabs.
	if (!inActiveSubroutine() || instinct_decision.subroutine_id == attackid(AttackId::BackstabId)) && p.subanimation >= SubanimationRecover
	{
		if instinct_decision.priority_decision == PriorityDecision::EnterAttackSubroutine {
            let attackid(s) = instinct_decision.subroutine_id.clone() else { panic!() };
			ss[AttackTypeIndex] = s as u8;
		}
		//dont attack if enemy in windup
		else if instinct_decision.priority_decision == PriorityDecision::DelayActions {
			//TODO should allow some longer term actions as long as they arn't attack here
			//do allow move up though
			if AttackNeuralNetChoice == AttackId::MoveUpId as u8 {
				ss[AttackTypeIndex] = AttackId::MoveUpId as u8;
			}
		}
		else{
			ss[AttackTypeIndex] = AttackNeuralNetChoice;
		}

        if ss[AttackTypeIndex] != 0 {
            ss[AttackStateIndex] = 1;
            //set time for this subroutine
            *sta = clock();
        }
    }

    //may not actually enter subroutine
    if inActiveAttackSubroutine() {
        match ss[AttackTypeIndex].try_into() {
            Ok(AttackId::MoveUpId) => {
                MoveUp(iReport);
            }
            Ok(AttackId::GhostHitId) => {
                ghostHit(iReport);
            }
            Ok(AttackId::DeadAngleId) => {
                deadAngle(iReport);
            }
            Ok(AttackId::BackstabId) => {
                backStab(iReport);
            }
            Ok(AttackId::TwoHandId) => {
                twoHand(iReport);
            }
            Ok(AttackId::SwitchWeaponId) => {
                SwitchWeapon(iReport);
            }
            Ok(AttackId::HealId) => {
                Heal(iReport);
            }
			Ok(AttackId::PivotBSId) => {
                PivotBS(iReport);
            }
            _ => {
                guiPrint!("{},0:ERROR Unknown attack action\npriority_decision={}\nAttackNeuralNetChoice={}\nsubroutine_states[AttackTypeIndex]={}",
										LocationState, instinct_decision.priority_decision.clone() as u8, AttackNeuralNetChoice, ss[AttackTypeIndex]);
            }
        }
    }
}
