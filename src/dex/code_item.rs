//! DEX bytecode
//!
//! Code item elements contain the actual bytecode of the app.
//! Each code item represent a function and the associated bytecode,
//! along with some metadata such as the number of registers, try/catch
//! offsets, etc.

use std::io::{Seek, SeekFrom};

use crate::dex::reader::DexReader;
use crate::dex::{
    instructions,
    instructions::Instructions
};

use crate::dex::types::DexTypes;

/// A `try` statement with offset to the `catch` part
#[derive(Clone, Debug)]
pub struct TryItem {
    start_addr : u32,
    insn_count : u16,
    handler_off: u16
}

/// A `catch` statement
#[derive(Clone, Debug)]
pub struct EncodedCatchHandler {
    size          : i32,
    handlers      : Vec<EncodedTypeAddrPair>,
    catch_all_addr: Option<u32>,
}

/// Addresses of the handler for an exception of the given type
#[derive(Clone, Debug)]
pub struct EncodedTypeAddrPair {
    decoded_type: String,
    addr        : u32,
}

/// Code structure for a method
#[derive(Debug)]
pub struct CodeItem {
    registers_size: u16,
    ins_size      : u16,
    outs_size     : u16,
    debug_info_off: u32,
    pub insns         : Option<Vec<Instructions>>,
    tries         : Option<Vec<TryItem>>,
    handlers      : Option<Vec<EncodedCatchHandler>>
}

impl CodeItem {
    /// Build a `CodeItem` struct from the reader
    pub fn build(dex_reader: &mut DexReader,
                 offset: u32,
                 types_list: &DexTypes) -> Self {

        /* Go to start of code item */
        dex_reader.bytes.seek(SeekFrom::Start(offset.into())).unwrap();

        /* Get the metadata */
        let registers_size = dex_reader.read_u16().unwrap();
        let ins_size       = dex_reader.read_u16().unwrap();
        let outs_size      = dex_reader.read_u16().unwrap();
        let tries_size     = dex_reader.read_u16().unwrap();
        let debug_info_off = dex_reader.read_u32().unwrap();
        let insns_size     = dex_reader.read_u32().unwrap();

        /* Get the actual bytecode */
        let mut insns = Vec::with_capacity(insns_size as usize);
        let end_offset = dex_reader.bytes.position() + (insns_size * 2) as u64;

        while dex_reader.bytes.position() < end_offset {
            instructions::parse_read(dex_reader, &mut insns);
        }

        /* Check if there is some padding */
        if tries_size != 0 && insns_size % 2 == 1 {
            _ = dex_reader.read_u16().unwrap();
        }

        let mut tries = Vec::<TryItem>::new();
        let mut handlers = Vec::<EncodedCatchHandler>::new();

        if tries_size != 0 {
            tries = Vec::with_capacity(tries_size as usize);
            for _ in 0..tries_size {
                let start_addr = dex_reader.read_u32().unwrap();
                let insn_count = dex_reader.read_u16().unwrap();
                let handler_off = dex_reader.read_u16().unwrap();

                tries.push(TryItem {
                    start_addr,
                    insn_count,
                    handler_off
                });
            }

            let (handlers_list_size, _) = dex_reader.read_uleb128().unwrap();
            handlers = Vec::with_capacity(handlers_list_size as usize);

            for _ in 0..handlers_list_size {
                let (handler_size, _) = dex_reader.read_sleb128().unwrap();
                let mut type_add_pairs = Vec::with_capacity(handler_size.unsigned_abs() as usize);

                for _ in 0..handler_size.abs() {
                    let (type_idx, _) = dex_reader.read_uleb128().unwrap();
                    let decoded_type = types_list.items.get(type_idx as usize)
                                                       .unwrap_or(&String::from("MISSINGTYPE"))  // FIXME
                                                       .to_string();
                    let (addr, _) = dex_reader.read_uleb128().unwrap();

                    type_add_pairs.push(EncodedTypeAddrPair {
                        decoded_type,
                        addr
                    });

                }

                if handler_size <= 0 {
                    let (catch_all_addr, _) = dex_reader.read_uleb128().unwrap();
                    handlers.push(EncodedCatchHandler {
                        size: handler_size,
                        handlers: type_add_pairs,
                        catch_all_addr: Some(catch_all_addr)
                    });
                } else {
                    handlers.push(EncodedCatchHandler {
                        size: handler_size,
                        handlers: type_add_pairs,
                        catch_all_addr: None
                    });
                }
            }
        }

        if tries_size != 0 {
            CodeItem {
                registers_size,
                ins_size,
                outs_size,
                debug_info_off,
                // insns: parsed_ins,
                insns: Some(insns),
                tries: Some(tries),
                handlers: Some(handlers)
            }
        } else {
            CodeItem {
                registers_size,
                ins_size,
                outs_size,
                debug_info_off,
                // insns: parsed_ins,
                insns: Some(insns),
                tries: None,
                handlers: None
            }
        }
    }
}
