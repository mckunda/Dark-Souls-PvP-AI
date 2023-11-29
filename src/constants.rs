pub const EnemyId: i32 = 0;
pub const PlayerId: i32 = 1;
pub const Enemy_loc_x_offsets_length: usize = 4;
pub const Player_loc_x_offsets_length: usize = 5;
pub const Enemy_loc_y_offsets_length: usize = 5;
pub const Player_loc_y_offsets_length: usize = 5;
pub const Enemy_rotation_offsets_length: usize = 5;
pub const Player_rotation_offsets_length: usize = 4;
pub const Enemy_animationType_offsets_length: usize = 5;
pub const Player_animationType_offsets_length: usize = 5;
pub const Enemy_hp_offsets_length: usize = 3;
pub const Player_hp_offsets_length: usize = 5;
pub const Player_stamina_offsets_length: usize = 5;
pub const Enemy_r_weapon_offsets_length: usize = 5;
pub const Player_r_weapon_offsets_length: usize = 5;
pub const Enemy_l_weapon_offsets_length: usize = 5;
pub const Player_l_weapon_offsets_length: usize = 5;
//the current subanimation being executed
pub const AttackSubanimationWindup : u32 = 00;
pub const AttackSubanimationWindupClosing : u32 = 01;
pub const AttackSubanimationWindupGhostHit : u32 = 02;
pub const AttackSubanimationActiveDuringHurtbox : u32 = 11;
pub const LockInSubanimation : u32 = 12;
pub const AttackSubanimationActiveHurtboxOver : u32 = 13;
pub const PoiseBrokenSubanimation : u32 = 14;
pub const SubanimationRecover : u32 = 20;
pub const SubanimationNeutral : u32 = 30;
pub const Enemy_hurtboxActive_offsets_length: usize = 5;
pub const Enemy_animationTimer_offsets_length: usize = 5;
pub const Player_animationTimer_offsets_length: usize = 5;
pub const Enemy_animationTimer2_offsets_length: usize = 5;
pub const Player_animationTimer2_offsets_length: usize = 5;
pub const Enemy_animationID_offsets_length: usize = 5;
pub const Player_animationID_offsets_length: usize = 5;
pub const Enemy_animationID2_offsets_length: usize = 5;
pub const Player_animationID2_offsets_length: usize = 5;
pub const Enemy_animationID3_offsets_length: usize = 5;
pub const Player_animationID3_offsets_length: usize = 2;
pub const Player_readyState_offsets_length: usize = 5;
pub const Enemy_velocity_offsets_length: usize = 5;
pub const Player_Lock_on_offsets_length: usize = 5;
pub const Player_twohanding_offsets_length: usize = 5;
pub const Enemy_stamRecovery_offsets_length: usize = 5;
pub const Player_Poise_offsets_length: usize = 5;
pub const Enemy_Poise_offsets_length: usize = 5;
pub const Player_BleedStatus_offsets_length: usize = 2;
pub const PI: f32 = 3.14159265f32;

//NOTE: this is curently hardcoded until i find a dynamic way
//How To Find: Increase this value until the attack ends with the AI turned away from the enemy. Decrease till it doesnt.
pub const WeaponGhostHitTime_QFS: f32 = 0.22;

pub const CLOCKS_PER_SEC: i32 = 1000000;

pub const TimeForR3ToTrigger: i64 = 50;

pub const TimeForCameraToRotateAfterLockon: i64 = 180;

//how much time we give to allow the camera to rotate.
pub const TimeDeltaForGameRegisterAction: i64 = 170;

pub const TotalTimeInSectoReverseRoll: f32 = (TimeForR3ToTrigger + TimeForCameraToRotateAfterLockon + TimeDeltaForGameRegisterAction + 50) as f32 / (CLOCKS_PER_SEC as f32);

pub const inputDelayForStopCircle: i64 = 40;

pub const inputDelayForOmnistepWait: i64 = 40;

pub const inputDelayForStopOmnistepJoystickDirection: i64 = 40;

pub const inputDelayForStopStrafe: i64 = 800;

pub const inputDelayForStart: i64 = 10;

//if we exit move forward and go into attack, need this to prevent kick
pub const inputDelayForKick: i64 = 50;
