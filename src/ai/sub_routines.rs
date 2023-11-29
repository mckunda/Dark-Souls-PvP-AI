use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::ai::back_to_enum;

//subroutine states, routines that are ongoing over a period of frames.
//store what subroutine is active (defence/attack and the subroutine id), and what state it is in

pub const DodgeStateIndex: usize = 0;
pub const DodgeTypeIndex: usize  = 1;
pub const AttackStateIndex: usize = 2;
pub const AttackTypeIndex: usize = 3;

lazy_static!(
	pub static ref subroutine_states: Mutex<[u8;4]> = Mutex::new([0u8;4]);
);

// for timing subroutine operations
// allow attack and defense subroutines to operate independently
lazy_static!(
	pub static ref startTimeAttack: Mutex<i64> = Mutex::new(0);
);
lazy_static!(
	pub static ref startTimeDefense: Mutex<i64> = Mutex::new(0);
);

//Dodge Ids
back_to_enum!(
	#[derive(Clone, PartialEq)]
	pub enum DefenseId {
		DefNoneId, //should only be used for initalizing. Should never reach AiMethod code
		StandardRollId,
		BackstepId,
		OmnistepBackwardsId,
		CounterStrafeLeftId,
		CounterStrafeRightId,
		L1AttackId,
		ReverseRollBSId,
		ToggleEscapeId,
		PerfectBlockId,
		ParryId,
	}
);

//Attack Ids
back_to_enum!(
	#[derive(Clone, PartialEq)]
	pub enum AttackId {
		AtkNoneId,//should only be used for initalizing. Should never reach AiMethod code
		MoveUpId,
		GhostHitId,
		DeadAngleId,
		BackstabId,
		TwoHandId,
		SwitchWeaponId,
		HealId,
		PivotBSId,
	}
);

pub const SubroutineActive: u8 = 1;
pub const SubroutineExiting: u8 = 2;
pub const NoSubroutineActive: u8 = 0;




//find if we are currently in an active subroutine, to prevent simultanious subroutine conflicts
pub fn inActiveSubroutine() -> bool {
    inActiveDodgeSubroutine() || inActiveAttackSubroutine()
}

pub fn inActiveDodgeSubroutine() -> bool {
	let ss = subroutine_states.lock().unwrap(); // TODO: error handling
    ss[DodgeStateIndex] != 0
}

pub fn inActiveAttackSubroutine() -> bool {
	let ss = subroutine_states.lock().unwrap(); // TODO: error handling
    ss[AttackStateIndex] != 0
}

pub fn OverrideLowPriorityAttackSubroutines() {
	let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling
    if ss[AttackTypeIndex] == AttackId::MoveUpId as u8 || ss[AttackTypeIndex] == AttackId::SwitchWeaponId as u8 {
        ss[AttackTypeIndex] = NoSubroutineActive as u8;
        ss[AttackStateIndex] = NoSubroutineActive as u8;
    }
}

pub fn OverrideLowPriorityDefenseSubroutines(){
	let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling
	if ss[DodgeTypeIndex] == DefenseId::CounterStrafeLeftId as u8 || ss[DodgeTypeIndex] == DefenseId::CounterStrafeRightId as u8 {
        ss[DodgeTypeIndex] = NoSubroutineActive as u8;
        ss[DodgeStateIndex] = NoSubroutineActive as u8;
    }
}

//handles aborting low priority subroutines in case of immediate necessary change
//NOTE ensure this isn't called and then the same overridden subroutine isn't retriggered
pub fn OverrideLowPrioritySubroutines(){
    OverrideLowPriorityDefenseSubroutines();
    OverrideLowPriorityAttackSubroutines();
}

//saftly exit all subroutines in the exit state
//done so that a dodge subroutine exit doesn't allow an immediate enty into an attack subroutine until next tick, and dodge rechecks
pub fn SafelyExitSubroutines(){
	let mut ss = subroutine_states.lock().unwrap(); // TODO: error handling
    if ss[DodgeStateIndex] == SubroutineExiting as u8 {
        ss[DodgeStateIndex] = NoSubroutineActive as u8;
        ss[DodgeTypeIndex] = NoSubroutineActive as u8;
    }
    if ss[AttackStateIndex] == SubroutineExiting as u8 {
        ss[AttackStateIndex] = NoSubroutineActive as u8;
        ss[AttackTypeIndex] = NoSubroutineActive as u8;
    }
}
