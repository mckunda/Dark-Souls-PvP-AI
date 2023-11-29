use std::cell::RefCell;
use crate::ai::gui::LocationDetection;
use std::ffi::c_void;
use std::sync::Mutex;
use lazy_static::lazy_static;
use parking_lot::ReentrantMutex;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use crate::ai::animation_mappings::{AnimationCombineReturn, CombineLastAnimation, dodgeTimings, isAttackAnimation, isDodgeAnimation, isVulnerableAnimation};
use crate::ai::guiPrint;
use crate::ai::memory::{AppendAIHP, AppendAnimationTypeEnemy, AppendLastAnimationIdEnemy};
use crate::ai::memory_edits::FindPointerAddr;
use crate::constants::{Enemy_animationType_offsets_length, Enemy_hp_offsets_length, Enemy_loc_x_offsets_length, Enemy_loc_y_offsets_length, Enemy_rotation_offsets_length, EnemyId, Player_animationType_offsets_length, Player_hp_offsets_length, Player_loc_x_offsets_length, Player_loc_y_offsets_length, Player_rotation_offsets_length, Player_stamina_offsets_length, PlayerId};

#[derive(Clone, Copy)]
pub struct Character {
	//data for x location
	pub location_x_address: u64,
	pub loc_x: f32,
	//data for y location
    pub location_y_address: u64,
	pub loc_y: f32,
	//data for rotation
	pub rotation_address: u64,
	pub rotation: f32,
	//current animation type id
	pub animationType_address: u64,
    pub animationType_id: u8,
	//hp
	pub hp_address: u64,
	pub hp: u32,
    //stamina
    pub stamina_address: u64,
    pub stamina: i32,
	//current Right hand weapon they are holding
	pub r_weapon_address: u64,
	pub r_weapon_id: u32,
	//current left hand weapon they are holding
	pub l_weapon_address: u64,
	pub l_weapon_id: u32,
	//hurtbox size(range) of weapon. Bows/Magic have high range
	pub weaponRange: f32,
    //encompases the various states of an animation
    pub subanimation: u32,
    //the current animation id
    pub animationId_address: u64,
    //secondary animation id. Used rarely
    pub animationId2_address: u64,
	//tertiary animation id. Only used for backstabs?
	pub animationId3_address: u64,
	pub in_backstab: u8,
	//animation timer.
    pub animationTimer_address: u64,
    //secondary animation timer. Used rarely
    pub animationTimer2_address: u64,
	//hurtbox state
    pub hurtboxActive_address: u64,
    //ready/animation switchable state
    pub readyState_address: u64,
    //velocity. used for backstab detection
    pub velocity_address: u64,
    pub velocity: f32,
    //if locked on. used for verification for counter strafe
    pub locked_on_address: u64,
    pub locked_on: u8,
    //time left before enemy hurtbox activates. Used for reverse roll vs dodge roll check
    pub dodgeTimeRemaining: f32,
    //if player is two handing or not
    pub twoHanding_address: u64,
    pub twoHanding: u8,
    //stamina recovery rate
    pub staminaRecoveryRate_address: u64,
    pub staminaRecoveryRate: i32,
    //current poise
    pub poise_address: u64,
    pub poise: f32,
    //current bleed state
    pub bleedStatus_address: u64,
    pub bleedStatus: i32,
}

//initalize the phantom and player
lazy_static!(
    pub static ref Enemy: ReentrantMutex<RefCell<Character>> = ReentrantMutex::new(RefCell::new(
        Character {
            location_x_address: 0,
            loc_x: 0f32,
            location_y_address: 0,
            loc_y: 0f32,
            rotation_address: 0,
            rotation: 0f32,
            animationType_address: 0,
            animationType_id: 0,
            hp_address: 0,
            hp: 0,
            stamina_address: 0,
            stamina: 0,
            r_weapon_address: 0,
            r_weapon_id: 0,
            l_weapon_address: 0,
            l_weapon_id: 0,
            weaponRange: 0f32,
            subanimation: 0,
            animationId_address: 0,
            animationId2_address: 0,
            animationId3_address: 0,
            in_backstab: 0,
            animationTimer_address: 0,
            animationTimer2_address: 0,
            hurtboxActive_address: 0,
            readyState_address: 0,
            velocity_address: 0,
            velocity: 0f32,
            locked_on_address: 0,
            locked_on: 0,
            dodgeTimeRemaining: 0f32,
            twoHanding_address: 0,
            twoHanding: 0,
            staminaRecoveryRate_address: 0,
            staminaRecoveryRate: 0,
            poise_address: 0,
            poise: 0f32,
            bleedStatus_address: 0,
            bleedStatus: 0, 
        })
    );
);
lazy_static!(
    pub static ref Player: Mutex<Character> = Mutex::new(
        Character {
            location_x_address: 0,
            loc_x: 0f32,
            location_y_address: 0,
            loc_y: 0f32,
            rotation_address: 0,
            rotation: 0f32,
            animationType_address: 0,
            animationType_id: 0,
            hp_address: 0,
            hp: 0,
            stamina_address: 0,
            stamina: 0,
            r_weapon_address: 0,
            r_weapon_id: 0,
            l_weapon_address: 0,
            l_weapon_id: 0,
            weaponRange: 0f32,
            subanimation: 0,
            animationId_address: 0,
            animationId2_address: 0,
            animationId3_address: 0,
            in_backstab: 0,
            animationTimer_address: 0,
            animationTimer2_address: 0,
            hurtboxActive_address: 0,
            readyState_address: 0,
            velocity_address: 0,
            velocity: 0f32,
            locked_on_address: 0,
            locked_on: 0,
            dodgeTimeRemaining: 0f32,
            twoHanding_address: 0,
            twoHanding: 0,
            staminaRecoveryRate_address: 0,
            staminaRecoveryRate: 0,
            poise_address: 0,
            poise: 0f32,
            bleedStatus_address: 0,
            bleedStatus: 0, 
        }
    );
);


//TODO prune as many of these as possible. what needs to be kept for only one char?

//basic values and offsets we use
//the base address, which offsets are added to
//this MUST be 64 bits to account for max possible address space
lazy_static!(
    pub static ref Enemy_base_add: Mutex<u64> = Mutex::new(0x00F7DC70);
);
lazy_static!(
    pub static ref player_base_add: Mutex<u64> = Mutex::new(0x00F7D644);
);

//offsets and length for x location
static Enemy_loc_x_offsets: &'static [u64] = &[ 0x4, 0x4, 0x2C, 0x260 ];
static Player_loc_x_offsets: &'static [u64] = &[ 0x3C, 0x330, 0x4, 0x20C, 0x3C0 ];
//offsets and length for y location
static Enemy_loc_y_offsets: &'static [u64] = &[ 0x4, 0x4, 0x28, 0x54, 0x268 ];
static Player_loc_y_offsets: &'static [u64] = &[ 0x3C, 0x330, 0x4, 0x20C, 0x3C8 ];
//offsets and length for rotation.
static Enemy_rotation_offsets: &'static [u64] = &[ 0x4, 0x4, 0x28, 0x54, 0x34 ];
static Player_rotation_offsets: &'static [u64] = &[ 0x3C, 0x28, 0x1C, 0x4 ];
//offsets and length for animation type id
static Enemy_animationType_offsets: &'static [u64] = &[ 0x4, 0x4, 0x28, 0x54, 0x1EC ];
static Player_animationType_offsets: &'static [u64] = &[ 0x288, 0xC, 0xC, 0x10, 0x41C ];
//hp
static Enemy_hp_offsets: &'static [u64] = &[ 0x4, 0x4, 0x2D4 ];
static Player_hp_offsets: &'static [u64] = &[ 0x288, 0xC, 0x330, 0x4, 0x2D4 ];
//stamina
static Player_stamina_offsets: &'static [u64] = &[ 0x288, 0xC, 0x330, 0x4, 0x2E4 ];
//R weapon id
static Enemy_r_weapon_offsets: &'static [u64] = &[ 0x4, 0x4, 0x34C, 0x654, 0x1F8 ];
static Player_r_weapon_offsets: &'static [u64] = &[ 0x3C, 0x30, 0xC, 0x654, 0x1F8 ];
pub const Enemy_r_weapon_offsets_length: usize = 5;
pub const Player_r_weapon_offsets_length: usize = 5;
//L weapon id
static Enemy_l_weapon_offsets: &'static [u64] = &[ 0x4, 0x4, 0x34C, 0x654, 0x1B8 ];
static Player_l_weapon_offsets: &'static [u64] = &[ 0x3C, 0x30, 0xC, 0x654, 0x1B4 ];
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
//if enemy's weapon's hurtbox is active
static Enemy_hurtboxActive_offsets: &'static [u64] = &[ 0x4, 0x0, 0xC, 0x3C, 0xF ];
pub const Enemy_hurtboxActive_offsets_length: usize = 5;
//time animation has been active
static Enemy_animationTimer_offsets: &'static [u64] = &[ 0x4, 0x4, 0x28, 0x18, 0x4DC ];
pub const Enemy_animationTimer_offsets_length: usize = 5;
static Player_animationTimer_offsets: &'static [u64] = &[ 0x28, 0x0, 0x148, 0x4C8, 0x4DC ];
pub const Player_animationTimer_offsets_length: usize = 5;
//second timer for animation. Note sometimes due to lag this will cut itself off early to that timer 1 can start at correct time
static Enemy_animationTimer2_offsets: &'static [u64] = &[ 0x4, 0x4, 0x28, 0x18, 0x440 ];
pub const Enemy_animationTimer2_offsets_length: usize = 5;
static Player_animationTimer2_offsets: &'static [u64] = &[ 0x28, 0x0, 0x148, 0x4C8, 0x440 ];
pub const Player_animationTimer2_offsets_length: usize = 5;
//current animation id
static Enemy_animationID_offsets: &'static [u64] = &[ 0x4, 0x4, 0x28, 0x18, 0x444 ];
pub const Enemy_animationID_offsets_length: usize = 5;
static Player_animationID_offsets: &'static [u64] = &[ 0x288, 0xC, 0x618, 0x28, 0x7B0 ];
pub const Player_animationID_offsets_length: usize = 5;
//second animation id
static Enemy_animationID2_offsets: &'static [u64] = &[ 0x4, 0x4, 0x28, 0x18, 0x3A8 ];
pub const Enemy_animationID2_offsets_length: usize = 5;
static Player_animationID2_offsets: &'static [u64] = &[ 0x3C, 0x28, 0x18, 0x8C, 0x1D4 ];
pub const Player_animationID2_offsets_length: usize = 5;
//teriary animation id
static Enemy_animationID3_offsets: &'static [u64] = &[ 0x4, 0x4, 0x65C, 0x268, 0x770 ];
pub const Enemy_animationID3_offsets_length: usize = 5;
static Player_animationID3_offsets: &'static [u64] = &[ 0x3C, 0x10C ];
pub const Player_animationID3_offsets_length: usize = 2;
//if in a ready/animation switchable state
static Player_readyState_offsets: &'static [u64] = &[ 0x3C, 0x30, 0xC, 0x20C, 0x7D2 ];
pub const Player_readyState_offsets_length: usize = 5;
//speed the opponent is approaching at. Player doesnt need to know their own. Idealy would like just if sprinting or not, actual velocity isnt important
//-0.04 slow walk
//-0.13 walk
//-0.16 - 18 sprint
static Enemy_velocity_offsets: &'static [u64] = &[ 0x4, 0x4, 0x658, 0x5C, 0x3BC ];
pub const Enemy_velocity_offsets_length: usize = 5;
//if player is locked on. used for verification only
static Player_Lock_on_offsets: &'static [u64] = &[ 0x3C, 0x170, 0x2C, 0x390, 0x128 ];
pub const Player_Lock_on_offsets_length: usize = 5;
//handed state of player
static Player_twohanding_offsets: &'static [u64] = &[ 0x28, 0x0, 0x148, 0x4C8, 0x0 ];
pub const Player_twohanding_offsets_length: usize = 5;
//stamina recovery rate of enemy
static Enemy_stamRecovery_offsets: &'static [u64] = &[ 0x4, 0x4, 0x170, 0x34C, 0x408 ];
pub const Enemy_stamRecovery_offsets_length: usize = 5;
//current poise
static Player_Poise_offsets: &'static [u64] = &[ 0x28, 0x18, 0xE0, 0xC, 0x1C0 ];
pub const Player_Poise_offsets_length: usize = 5;
static Enemy_Poise_offsets: &'static [u64] = &[ 0x4, 0x4, 0x60, 0x8, 0x1C0 ];
pub const Enemy_Poise_offsets_length: usize = 5;
//bleed status
static Player_BleedStatus_offsets: &'static [u64] = &[ 0x3C, 0x308 ];
pub const Player_BleedStatus_offsets_length: usize = 2;

pub const PI: f32 = 3.14159265f32;

//NOTE: this is curently hardcoded until i find a dynamic way
//How To Find: Increase this value until the attack ends with the AI turned away from the enemy. Decrease till it doesnt.
pub const WeaponGhostHitTime_QFS: f32 = 0.22;

lazy_static!(
    static ref waitingForAnimationTimertoCatchUp: Mutex<bool> = Mutex::new(false);
);

//read memory for the character's variables
pub unsafe fn ReadPlayer(c: &mut Character, processHandle: HANDLE, characterId: i32){
    //TODO read large block that contains all data, then parse in process
    //read x location
    ReadProcessMemory(processHandle, c.location_x_address as *const c_void, &mut (c.loc_x) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error // TODO: handle error
    guiPrint!("{},0:X:{}", characterId, c.loc_x);
    //read y location
    ReadProcessMemory(processHandle, c.location_y_address as *const c_void, &mut (c.loc_y) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
    guiPrint!("{},1:Y:{}", characterId, c.loc_y);
    //read rotation of player
    ReadProcessMemory(processHandle, c.rotation_address as *const c_void, &mut (c.rotation) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
    //Player rotation is pi. 0 to pi,-pi to 0. Same as atan2
    //convert to radians, then to degrees
    c.rotation = (c.rotation + PI) * (180.0 / PI);
    guiPrint!("{},2:Rotation:{}", characterId, c.rotation);
    //read current animation type
    ReadProcessMemory(processHandle, c.animationType_address as *const c_void, &mut (c.animationType_id) as *mut _ as *mut c_void, 2, None).unwrap(); // TODO: handle error
    guiPrint!("{},3:Animation Type:{}", characterId, c.animationType_id);
    //remember enemy animation types
    if characterId == EnemyId {
        let id = Enemy.lock();
        let id = id.borrow().animationType_id;// TODO: error handling
        AppendAnimationTypeEnemy(id as u16);
    }
    //read hp
    ReadProcessMemory(processHandle, c.hp_address as *const c_void, &mut (c.hp) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
    guiPrint!("{},4:HP:{}", characterId, c.hp);
    if characterId == PlayerId {
        let hp = Player.lock().unwrap().hp; // TODO: error handling
        AppendAIHP(hp);
    }
    //read stamina
    if c.stamina_address != 0 {
        ReadProcessMemory(processHandle, c.stamina_address as *const c_void, &mut (c.stamina) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
        guiPrint!("{},5:Stamina:{}", characterId, c.stamina);
    }
    //read what weapon they currently have in right hand
    ReadProcessMemory(processHandle, c.r_weapon_address as *const c_void, &mut c.r_weapon_id as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
    guiPrint!("{},6:R Weapon:{}", characterId, c.r_weapon_id);
    //read what weapon they currently have in left hand
    ReadProcessMemory(processHandle, c.l_weapon_address as *const c_void, &mut (c.l_weapon_id) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
    guiPrint!("{},7:L Weapon:{}", characterId, c.l_weapon_id);

    //read if hurtbox is active on enemy weapon
    if c.hurtboxActive_address != 0 {
        let mut hurtboxActiveState: u8 = 0;
        ReadProcessMemory(processHandle, c.hurtboxActive_address as *const c_void, &mut hurtboxActiveState as *mut _ as *mut c_void, 1, None).unwrap(); // TODO: handle error
        if hurtboxActiveState != 0 {
            c.subanimation = AttackSubanimationActiveDuringHurtbox;
        }
    }
    let mut animationid: i32 = 0;
    ReadProcessMemory(processHandle, c.animationId_address as *const c_void, &mut animationid as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
    //need a second one b/c the game has a second one. the game has a second one b/c two animations can overlap.
    let mut animationid2: i32 = 0;
    ReadProcessMemory(processHandle, c.animationId2_address as *const c_void, &mut animationid2 as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
	//haven't discovered what the 3rd animation address is for besides backstabs
	let mut animationid3: i32 = 0;
	ReadProcessMemory(processHandle, c.animationId3_address as *const c_void, &mut animationid3 as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
	if animationid3 > 0 {
		c.in_backstab = 1;
	} else {
		c.in_backstab = 0;
	}

    //keep track of enemy animations in memory
    if characterId == EnemyId{
        let mut w = waitingForAnimationTimertoCatchUp.lock().unwrap(); // TODO: handle error
        if animationid != -1 {
            *w |= AppendLastAnimationIdEnemy(animationid);
        } else {
            *w |= AppendLastAnimationIdEnemy(animationid2);
        }
    }

    guiPrint!("{},8:Animation Id 1/2:{}/{}", characterId, animationid, animationid2);

    let attackAnimationInfo: u8 = isAttackAnimation(c.animationType_id);

    //---any subanimation that is based purely off animation id should be prioritized in subanimation state setting---
    if isVulnerableAnimation(animationid) != 0
    {
        c.subanimation = LockInSubanimation;
    }
    else if animationid >= 2000 && animationid <= 2056 {//animation states for poise breaks, knockdowns, launches, staggers
        c.subanimation = PoiseBrokenSubanimation;
    }
    //---subanimations based on animation type---
    else if isDodgeAnimation(c.animationType_id as u16) && animationid != -1 {//in theory these two should never conflict. In practice, one might be slow.
        c.subanimation = LockInSubanimation;
    }

    //read how long the animation has been active, check with current animation, see if hurtbox is about to activate
    //what i want is a countdown till hurtbox is active
    //cant be much higher b/c need spell attack timings
    //also check that this is an attack that involves subanimation
    else if attackAnimationInfo == 2 || attackAnimationInfo == 4 || attackAnimationInfo == 5 {
        let mut curAnimationTimer_address: u64 = 0;
        let mut curAnimationid: i32 = 0;

        //need a second one b/c the game has a second one. the game has a second one b/c two animations can overlap.
        if animationid2 > 1000 {
            curAnimationTimer_address = c.animationTimer2_address;
            curAnimationid = animationid2;
        }
        else if animationid > 1000 {
            //if kick or parry (aid ends in 100), use catch all aid
            if animationid % 1000 == 100 {
                curAnimationid = 100;
            } else{
                curAnimationid = animationid;
            }
            curAnimationTimer_address = c.animationTimer_address;
        }
        else{
            guiPrint!("{},3:ALERT: Animation type found but not animation ids", LocationDetection);
        }

        if curAnimationid != 0 {
            let mut animationTimer: f32 = 0f32;
            let mut w = waitingForAnimationTimertoCatchUp.lock().unwrap(); // TODO: handle error
            ReadProcessMemory(processHandle, curAnimationTimer_address as *const c_void, &mut animationTimer as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error

            //handle the timer not being reset to 0 as soon as a new animation starts
            //wait for animation timer to go below 0.1(a tell for its been reset, since no animation is short enough to have held it at 0.1), then we can stop manually resetting it
            if *w && animationTimer > 0.1 {
                animationTimer = 0.0;
            } else{
                *w = false;
            }

            //sometimes, due to lag, dark souls cuts one animation short and makes the next's hurtbox timing later. handle this for the animations that do it by treating the two animations as one.
			let mut animationToCombine = AnimationCombineReturn {
                animationId: 0,
                partNumber: 0,
            };

			CombineLastAnimation(curAnimationid, &mut animationToCombine);
            if animationToCombine.animationId != 0 {
                curAnimationid = animationToCombine.animationId;//combine the two animations and treat as one id 
                if animationToCombine.partNumber != 0 {
                    //this uses the fact that animation timers are not reset by the game after use
                    let mut animationTimer2: f32 = 0f32;
                    ReadProcessMemory(processHandle, c.animationTimer2_address as *const c_void, &mut animationTimer2 as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
                    animationTimer += animationTimer2;
                }
            }

            let dodgeTimer: f32 = dodgeTimings(curAnimationid);
            let timeDelta: f32 = dodgeTimer - animationTimer;
            c.dodgeTimeRemaining = timeDelta;

            if timeDelta >= 1.0 {
                c.subanimation = SubanimationNeutral;
            } else if timeDelta < 1.0 && timeDelta > 0.55 {
                c.subanimation = AttackSubanimationWindup;
            }
            //between 0.55 and 0.15 sec b4 hurtbox. If we have less that 0.15 we can't dodge.
            else if timeDelta <= 0.55 && timeDelta >= 0.15 {
                c.subanimation = AttackSubanimationWindupClosing;
            }
            //just treat this as the hurtbox is activated
            else if timeDelta < 0.15 && timeDelta >= 0f32 {
                c.subanimation = AttackSubanimationActiveDuringHurtbox;
            }
            else if timeDelta < 0f32 {
                c.subanimation = AttackSubanimationActiveHurtboxOver;
            }

            // time before the windup ends where we can still alter rotation (only for player)
			if animationTimer > WeaponGhostHitTime_QFS && timeDelta >= -0.3 && characterId == PlayerId {
                c.subanimation = AttackSubanimationWindupGhostHit;
            }

            guiPrint!("{},9:Animation Timer:{}\nDodge Time:{}", characterId, animationTimer, dodgeTimer);
        }
    }
    else if attackAnimationInfo == 1 {
        c.subanimation = AttackSubanimationWindup;
    }
    else if attackAnimationInfo == 3 {
        c.subanimation = AttackSubanimationActiveDuringHurtbox;
    }
    else{
        //else if (c->animationType_id == 0){//0 when running, walking, standing. all animation can immediatly transition to new animation. Or animation id = -1
        c.subanimation = SubanimationNeutral;
    }

    //read if in ready state(can transition to another animation)
    if c.readyState_address != 0 {
        let mut readyState: u8 = 0;
        ReadProcessMemory(processHandle, c.readyState_address as *const c_void, &mut readyState as *mut _ as *mut c_void, 1, None).unwrap(); // TODO: handle error
        if readyState != 0 {
            c.subanimation = SubanimationRecover;
        } /*else{ Not adding this now because it would lock out subanimations every time i move
            c->subanimation = LockInSubanimation;
            }*/
    }
    guiPrint!("{},10:Subanimation:{}", characterId, c.subanimation);

    //read the current velocity
    //player doesnt use this, and wont have the address set. enemy will
    if c.velocity_address != 0 {
        ReadProcessMemory(processHandle, c.velocity_address as *const c_void, &mut (c.velocity) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
        guiPrint!("{},11:Velocity:{}", characterId, c.velocity);
    }
    //read if the player is locked on
    if c.locked_on_address != 0 {
        ReadProcessMemory(processHandle, c.locked_on_address as *const c_void, &mut (c.locked_on) as *mut _ as *mut c_void, 1, None).unwrap(); // TODO: handle error
        guiPrint!("{},12:Locked On:{}", characterId, c.locked_on);
    }
    //read two handed state of player
    if c.twoHanding_address != 0 {
        ReadProcessMemory(processHandle, c.twoHanding_address as *const c_void, &mut (c.twoHanding) as *mut _ as *mut c_void, 1, None).unwrap(); // TODO: handle error
        guiPrint!("{},13:Two Handing:{}", characterId, c.twoHanding);
    }
    //read stamina recovery of enemy
    if c.staminaRecoveryRate_address != 0 {
        ReadProcessMemory(processHandle, c.staminaRecoveryRate_address as *const c_void, &mut (c.staminaRecoveryRate) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
        guiPrint!("{},14:Stamina Recovery Rate:{}", characterId, c.staminaRecoveryRate);
    }
    //read current poise
    ReadProcessMemory(processHandle, c.poise_address as *const c_void, &mut (c.poise) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
    guiPrint!("{},15:Poise:{}", characterId, c.poise);
    //read current bleed status
    if c.bleedStatus_address != 0 {
        ReadProcessMemory(processHandle, c.bleedStatus_address as *const c_void, &mut (c.bleedStatus) as *mut _ as *mut c_void, 4, None).unwrap(); // TODO: handle error
        guiPrint!("{},16:Bleed Status:{}", characterId, c.bleedStatus);
    }
}

pub fn ReadPlayerDEBUGGING(c: &mut Character) {
    c.loc_x = 1045.967773;
    c.loc_y = 864.3547974;
    c.rotation = 360f32;//facing kinda towards bonfire, same as pi/-pi
    c.animationType_id = 255; // 4294967295;
    c.hp = 1800;
    c.r_weapon_id = 301015;
    c.l_weapon_id = 900000;
    c.subanimation = SubanimationNeutral;
    c.velocity = 0f32;
}

pub unsafe fn ReadPointerEndAddresses(processHandle: HANDLE) {
    //add the pointer offsets to the address. This can be slow because its startup only
    let mut e = Enemy.lock(); // TODO: error handling
    let eba = Enemy_base_add.lock().unwrap(); // TODO: error handling
    let pba = player_base_add.lock().unwrap(); // TODO: error handling
    let mut e = e.borrow_mut();
    e.location_x_address = FindPointerAddr(processHandle, *eba, Enemy_loc_x_offsets_length, Enemy_loc_x_offsets);
    e.location_y_address = FindPointerAddr(processHandle, *eba, Enemy_loc_y_offsets_length, Enemy_loc_y_offsets);
    e.rotation_address = FindPointerAddr(processHandle, *eba, Enemy_rotation_offsets_length, Enemy_rotation_offsets);
    e.animationType_address = FindPointerAddr(processHandle, *eba, Enemy_animationType_offsets_length, Enemy_animationType_offsets);
    e.hp_address = FindPointerAddr(processHandle, *eba, Enemy_hp_offsets_length, Enemy_hp_offsets);
    e.stamina_address = 0;
    e.r_weapon_address = FindPointerAddr(processHandle, *eba, Enemy_r_weapon_offsets_length, Enemy_r_weapon_offsets);
    e.l_weapon_address = FindPointerAddr(processHandle, *eba, Enemy_l_weapon_offsets_length, Enemy_l_weapon_offsets);
    e.animationTimer_address = FindPointerAddr(processHandle, *eba, Enemy_animationTimer_offsets_length, Enemy_animationTimer_offsets);
    e.animationTimer2_address = FindPointerAddr(processHandle, *eba, Enemy_animationTimer2_offsets_length, Enemy_animationTimer2_offsets);
    e.animationId_address = FindPointerAddr(processHandle, *eba, Enemy_animationID_offsets_length, Enemy_animationID_offsets);
    e.animationId2_address = FindPointerAddr(processHandle, *eba, Enemy_animationID2_offsets_length, Enemy_animationID2_offsets);
	e.animationId3_address = FindPointerAddr(processHandle, *eba, Enemy_animationID3_offsets_length, Enemy_animationID3_offsets);
    e.hurtboxActive_address = FindPointerAddr(processHandle, *eba, Enemy_hurtboxActive_offsets_length, Enemy_hurtboxActive_offsets);
    e.readyState_address = 0;
    e.velocity_address = FindPointerAddr(processHandle, *eba, Enemy_velocity_offsets_length, Enemy_velocity_offsets);
    e.locked_on_address = 0;
    e.twoHanding_address = 0;
    e.staminaRecoveryRate_address = FindPointerAddr(processHandle, *eba, Enemy_stamRecovery_offsets_length, Enemy_stamRecovery_offsets);
    e.poise_address = FindPointerAddr(processHandle, *eba, Enemy_Poise_offsets_length, Enemy_Poise_offsets);
    e.bleedStatus_address = 0;

    let mut p = Player.lock().unwrap(); // TODO: error handling
    p.location_x_address = FindPointerAddr(processHandle, *pba, Player_loc_x_offsets_length, Player_loc_x_offsets);
    p.location_y_address = FindPointerAddr(processHandle, *pba, Player_loc_y_offsets_length, Player_loc_y_offsets);
    p.rotation_address = FindPointerAddr(processHandle, *pba, Player_rotation_offsets_length, Player_rotation_offsets);
    p.animationType_address = FindPointerAddr(processHandle, *pba, Player_animationType_offsets_length, Player_animationType_offsets);
    p.hp_address = FindPointerAddr(processHandle, *pba, Player_hp_offsets_length, Player_hp_offsets);
    p.stamina_address = FindPointerAddr(processHandle, *pba, Player_stamina_offsets_length, Player_stamina_offsets);
    p.r_weapon_address = FindPointerAddr(processHandle, *pba, Player_r_weapon_offsets_length, Player_r_weapon_offsets);
    p.l_weapon_address = FindPointerAddr(processHandle, *pba, Player_l_weapon_offsets_length, Player_l_weapon_offsets);
    p.animationTimer_address = FindPointerAddr(processHandle, *pba, Player_animationTimer_offsets_length, Player_animationTimer_offsets);
    p.animationTimer2_address = FindPointerAddr(processHandle, *pba, Player_animationTimer2_offsets_length, Player_animationTimer2_offsets);
    p.animationId_address = FindPointerAddr(processHandle, *pba, Player_animationID_offsets_length, Player_animationID_offsets);
    p.animationId2_address = FindPointerAddr(processHandle, *pba, Player_animationID2_offsets_length, Player_animationID2_offsets);
	p.animationId3_address = FindPointerAddr(processHandle, *pba, Player_animationID3_offsets_length, Player_animationID3_offsets);
    p.hurtboxActive_address = 0;
    p.readyState_address = FindPointerAddr(processHandle, *pba, Player_readyState_offsets_length, Player_readyState_offsets);
    p.velocity_address = 0;
    p.locked_on_address = FindPointerAddr(processHandle, *pba, Player_Lock_on_offsets_length, Player_Lock_on_offsets);
    p.twoHanding_address = FindPointerAddr(processHandle, *pba, Player_twohanding_offsets_length, Player_twohanding_offsets);
    p.staminaRecoveryRate_address = 0;
    p.poise_address = FindPointerAddr(processHandle, *pba, Player_Poise_offsets_length, Player_Poise_offsets);
    p.bleedStatus_address = FindPointerAddr(processHandle, *pba, Player_BleedStatus_offsets_length, Player_BleedStatus_offsets);
}