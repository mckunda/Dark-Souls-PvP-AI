use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::ai::ffi::clock;

pub const last_subroutine_states_self_LENGTH: usize = 20;

lazy_static!(
    pub static ref last_subroutine_states_self: Mutex<[u8;last_subroutine_states_self_LENGTH]> = Mutex::new([0;last_subroutine_states_self_LENGTH]);
);


pub const last_animation_ids_enemy_LENGTH: usize = 20;
lazy_static!(
    pub static ref last_animation_ids_enemy: Mutex<[i32;last_animation_ids_enemy_LENGTH]> = Mutex::new([0;last_animation_ids_enemy_LENGTH]);
);

//update every 100 ms. LENGTH*100 = time of memory
pub const last_animation_types_enemy_LENGTH: usize = 100;
lazy_static!(
    pub static ref last_animation_types_enemy: Mutex<[u16;last_animation_types_enemy_LENGTH]> = Mutex::new([0;last_animation_types_enemy_LENGTH]);
);

//update every 100 ms.
pub const DistanceMemoryLENGTH: usize = 50;
lazy_static!(
    pub static ref DistanceMemory: Mutex<[f32;DistanceMemoryLENGTH]> = Mutex::new([0f32;DistanceMemoryLENGTH]);
);

//update every 500 ms
pub const AIHPMemoryLENGTH: usize = 20;
lazy_static!(
    pub static ref AIHPMemory: Mutex<[u32;AIHPMemoryLENGTH]> = Mutex::new([0;AIHPMemoryLENGTH]);
);


pub fn AppendLastSubroutineSelf(subroutineId: u8) {
    let mut s = last_subroutine_states_self.lock().unwrap(); // TODO: handle error
    for i in (0..last_subroutine_states_self_LENGTH).rev() {
        s[i] = s[i - 1];
    }
    s[0] = subroutineId;
}

//handles check that the new aid to add isnt the same as the most recent old one. This can't happen from attacks, because -1 it always between two attacks fo the same aid.
pub fn AppendLastAnimationIdEnemy(aid: i32) -> bool {
    let mut l = last_animation_ids_enemy.lock().unwrap(); // TODO: handle error

    if aid != l[0] {
        for i in (0..last_animation_ids_enemy_LENGTH).rev() {
            l[i] = l[i - 1];
        }
        l[0] = aid;
        return true;
    }

    false
}

lazy_static!(
    static ref lastAnimationTypeUpdateTime: Mutex<i64> = Mutex::new(0);
);

//stores enemy's last animation types. Does not duplicate if its the same as the last stored one, except if it is 0.
pub unsafe fn AppendAnimationTypeEnemy(animationType_id: u16) {
    let mut l = lastAnimationTypeUpdateTime.lock().unwrap(); // TODO: handle error
    let mut t = last_animation_types_enemy.lock().unwrap(); // TODO: handle error
    if clock() - *l >= 100 && (animationType_id != t[0] || animationType_id == 0)
    {
        for i in (0..last_animation_types_enemy_LENGTH).rev() {
            t[i] = t[i - 1];
        }
        t[0] = animationType_id;

        *l = clock();
    }
}

lazy_static!(
    static ref LastDistanceUpdateTime: Mutex<i64> = Mutex::new(0);
);

//store distance between AI and enemy over time
pub unsafe fn AppendDistance(distance: f32) {
    let mut l = LastDistanceUpdateTime.lock().unwrap(); // TODO: handle error
    let mut d = DistanceMemory.lock().unwrap(); // TODO: handle error
    if clock() - *l >= 100 {
        for i in (0..DistanceMemoryLENGTH).rev() {
            d[i] = d[i - 1];
        }
        d[0] = distance;

        *l = clock();
    }
}

lazy_static!(
    static ref LastAIHPMemoryUpdateTime: Mutex<i64> = Mutex::new(0);
);

pub unsafe fn AppendAIHP(hp: u32) {
    let mut l = LastAIHPMemoryUpdateTime.lock().unwrap(); // TODO: handle error
    let mut a = AIHPMemory.lock().unwrap(); // TODO: handle error
    if clock() - *l >= 500 {
        for i in (0..AIHPMemoryLENGTH).rev() {
            a[i] = a[i - 1];
        }
        a[0] = hp;

        *l = clock();
    }
}
