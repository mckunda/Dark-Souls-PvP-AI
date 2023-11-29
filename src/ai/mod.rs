pub mod animation_mappings;
pub mod sub_routines;
pub mod weapon_data;
pub mod helper_util;
pub mod character;
pub mod memory_edits;
pub mod memory;
pub mod settings;
pub mod gui;
pub mod vjoyhelper;
mod ai_decisions;
mod ai_methods;
pub mod initalize_fann;
mod mind_routines;
pub mod handler;
pub mod source;
mod test_space;

pub use std::println as guiPrint;

extern crate libc;

pub mod ffi {
    extern {
        pub fn clock() -> i64;
    }
}

// https://stackoverflow.com/a/57578431/10447751
macro_rules! back_to_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<u8> for $name {
            type Error = ();

            fn try_from(v: u8) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as u8 => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
        impl std::convert::TryFrom<u16> for $name {
            type Error = ();

            fn try_from(v: u16) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as u16 => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}

pub(crate) use back_to_enum;