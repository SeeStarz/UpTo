mod manual;
mod parser;
mod profile;

pub use parser::*;

pub mod export {
    #![allow(unused_imports)]
    pub use crate::parser::Cli;
    pub use crate::parser::manual::{
        BeginCommand as ManualBeginCommand, EndCommand as ManualEndCommand, ManualArgs,
        ManualCommand, ManualCommandType,
    };
    pub use crate::parser::profile::{
        EditCommand as ProfileEditCommand, NewCommand as ProfileNewCommand, ProfileCommand,
    };
    pub use clap::Parser;
}
