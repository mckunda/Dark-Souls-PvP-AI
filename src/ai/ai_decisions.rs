use crate::ai::{back_to_enum, guiPrint};
use crate::ai::ai_methods::TotalTimeInSectoReverseRoll;
use crate::ai::animation_mappings::{AnimationTypes, isAttackAnimation};
use crate::ai::character::{AttackSubanimationActiveDuringHurtbox, AttackSubanimationWindup, AttackSubanimationWindupClosing, Enemy, Player, PoiseBrokenSubanimation, SubanimationNeutral};
use crate::ai::gui::{LocationJoystick, LocationDetection};
use crate::ai::helper_util::{BackstabDetection, distance};
use crate::ai::memory::last_subroutine_states_self;
use crate::ai::sub_routines::{AttackId, DefenseId, OverrideLowPrioritySubroutines};
use crate::ai::weapon_data::IsWeaponShield;

back_to_enum!(
	#[derive(PartialEq, Clone)]
	pub enum PriorityDecision {
		EnemyNeutral = 0,//doesn't override anything, and doesn't define an attack or defense id
		DelayActions = 1,//doesnt define an attack or defense id
		EnterDodgeSubroutine = 2,//only defines defense ids
		EnterAttackSubroutine = 3,//only defines attack ids
	}
);

// union
#[derive(Clone, PartialEq)]
pub enum SubroutineId {
	attackid(AttackId),
	defenseid(DefenseId),
}

pub struct InstinctDecision {
	 pub priority_decision: PriorityDecision,
	 pub subroutine_id: SubroutineId,
}

//this handles actions that override any MindRoutine decisions
//makes decisions about what general types of actions the AI should take using standard discrete logic (i.e. should dodge, but not what type of dodge)
//composes the INSTINCT of the AI mind. Basic interactions/actions/reactions that are high accuracy, very fast, but not very complex.
pub fn InstinctDecisionMaking(instinct_decision: &mut InstinctDecision) {
	let mut p = Player.lock().unwrap(); // TODO: error handling
	let mut e = Enemy.lock();
	let mut e = e.borrow_mut(); // TODO: error handling
	instinct_decision.priority_decision = PriorityDecision::EnemyNeutral;
	let distanceByLine: f32 = distance(&p, &e);
	guiPrint!("{},1:Distance:{}", LocationJoystick, distanceByLine);
	let AtkID: u8 = isAttackAnimation(e.animationType_id);

	//Actions are organized in reverse order of importance
	//Higher actions are less important
	//TODO should formalize this in an actual order somehow

	//if not two handing
	if p.twoHanding == 0 && distanceByLine > e.weaponRange * 1.75 {
		instinct_decision.priority_decision = PriorityDecision::EnterAttackSubroutine;
		instinct_decision.subroutine_id = SubroutineId::attackid(AttackId::TwoHandId);
	}
	//l hand bare handed, not holding shield. safety distance
	if p.l_weapon_id == 900000 && distanceByLine > e.weaponRange*1.75 {
		instinct_decision.priority_decision = PriorityDecision::EnterAttackSubroutine;
		instinct_decision.subroutine_id = SubroutineId::attackid(AttackId::SwitchWeaponId);
	}
// #if 0
	//heal if enemy heals
	if (e.animationType_id == AnimationTypes::CrushUseItem as u8 || e.animationType_id == AnimationTypes::EstusSwig_part1 as u8 || e.animationType_id == AnimationTypes::EstusSwig_part2 as u8) && p.hp < 2000 {
		instinct_decision.priority_decision = PriorityDecision::EnterAttackSubroutine;
		instinct_decision.subroutine_id = SubroutineId::attackid(AttackId::HealId);
	}
// #endif
	//if enemy in range and we/enemy is not in invulnerable position (bs knockdown)
	if distanceByLine <= e.weaponRange && p.in_backstab == 0 && e.in_backstab == 0 {
		if (AtkID == 3 && e.subanimation <= AttackSubanimationActiveDuringHurtbox) ||
		//or animation where it is
		((AtkID == 2 || AtkID == 4) && (e.subanimation >= AttackSubanimationWindupClosing && e.subanimation <= AttackSubanimationActiveDuringHurtbox))
		{
			OverrideLowPrioritySubroutines();
			guiPrint!("{},0:about to be hit (anim type id:{}) (suban id:{})", LocationDetection, e.animationType_id, e.subanimation);
			instinct_decision.priority_decision = PriorityDecision::EnterDodgeSubroutine;

			//Decide on dodge action

			//if we got hit already, and are in a state we can't dodge from, toggle escape the next hit
			if p.subanimation == PoiseBrokenSubanimation && (e.dodgeTimeRemaining > 0.2 && e.dodgeTimeRemaining < 0.3)
			{
				instinct_decision.subroutine_id = SubroutineId::defenseid(DefenseId::ToggleEscapeId);
				return;
			}

			let lsss = last_subroutine_states_self.lock().unwrap(); // TODO: error handling
			//while staggered, dont enter any subroutines
			if p.subanimation != PoiseBrokenSubanimation
			{
				if distance(&p, &e) <= 3f32 && TotalTimeInSectoReverseRoll < e.dodgeTimeRemaining &&
				//if just reverse rolled and next incoming attack and weapon speed < ?, do normal roll
				(lsss[0] != DefenseId::ReverseRollBSId as u8 || TotalTimeInSectoReverseRoll + 0.3 > e.dodgeTimeRemaining)
				{
					instinct_decision.subroutine_id = SubroutineId::defenseid(DefenseId::ReverseRollBSId);
				}
				else if e.dodgeTimeRemaining < 0.15 && e.dodgeTimeRemaining > 0f32 &&
				//we have a shield equipped/are two handing
				(p.twoHanding != 0 || IsWeaponShield(p.l_weapon_id)) &&
				//we're in a neutral state
				p.subanimation == SubanimationNeutral
				{
					instinct_decision.subroutine_id = SubroutineId::defenseid(DefenseId::PerfectBlockId);
				}
				//otherwise, normal roll
				else{
					instinct_decision.subroutine_id = SubroutineId::defenseid(DefenseId::StandardRollId);
				}
			}
			//if we had to toggle escape, they're probably comboing. Get out.
			if lsss[0] == DefenseId::ToggleEscapeId as u8 {
				instinct_decision.subroutine_id = SubroutineId::defenseid(DefenseId::StandardRollId);
			}
		}
		//windup, attack coming
		else if AtkID == 1 || ((AtkID == 2 || AtkID == 4) && e.subanimation == AttackSubanimationWindup) {
			guiPrint!("{},0:dont attack, enemy windup", LocationDetection);
			instinct_decision.priority_decision = PriorityDecision::DelayActions;
		}
	}

	//backstab checks. If AI can BS, always take it
	let BackStabStateDetected: u8 = BackstabDetection(&mut p, &mut e, distanceByLine);
	if BackStabStateDetected != 0 {
		OverrideLowPrioritySubroutines();

		guiPrint!("{},0:backstab state {}", LocationDetection, BackStabStateDetected);
		//in position to bs
		if BackStabStateDetected == 2 {
			instinct_decision.priority_decision = PriorityDecision::EnterAttackSubroutine;
			instinct_decision.subroutine_id = SubroutineId::attackid(AttackId::GhostHitId);
		}
		//try and move up for bs
		else if BackStabStateDetected == 1 {
			instinct_decision.priority_decision = PriorityDecision::EnterAttackSubroutine;
			instinct_decision.subroutine_id = SubroutineId::attackid(AttackId::MoveUpId);
		}
	}

	if instinct_decision.priority_decision == PriorityDecision::EnemyNeutral {
		guiPrint!("{},0:not about to be hit (enemy animation type id:{}) (enemy subanimation id:{})", LocationDetection, e.animationType_id, e.subanimation);
	}
}
