#![allow(dead_code)]

use clap::Parser;

pub mod logging;
pub mod dex_header;
pub mod dex_file;
pub mod map_list;
pub mod error;
pub mod dex_reader;
pub mod adler32;
pub mod constants;
pub mod dex_strings;
pub mod dex_types;
pub mod dex_protos;
pub mod dex_fields;
pub mod dex_methods;
pub mod dex_classes;
pub mod access_flags;
pub mod method_handle;
pub mod code_item;
pub mod opcodes;
pub mod instructions;

/* Actually unused for now but there should
 * be more options as things progress */
pub struct Config {
    pub log_level: u8,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            log_level: 0    // only show error messages
        }
    }
}

/// CLI args
#[derive(Parser)]
pub struct CliArgs {
    /// The path to the file to read
    #[arg(short, long)]
    pub apk: String,

    /// Loglevel
    #[arg(short, long, default_value_t = 0)]
    pub log_level: u8,
}