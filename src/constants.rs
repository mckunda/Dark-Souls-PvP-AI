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
