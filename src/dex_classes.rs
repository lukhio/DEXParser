use std::io::{Seek, SeekFrom};

use crate::dex_reader::DexReader;
use crate::dex_types::DexTypes;
use crate::dex_fields::DexFields;
use crate::dex_methods::DexMethods;
use crate::dex_strings::DexStrings;
use crate::access_flags::AccessFlag;

const NO_INDEX: u32 = 0xffffffff;

#[derive(Debug)]
pub struct ClassDefItem {
    class_idx: u32,
    access_flags: u32,
    superclass_idx: u32,
    interfaces_off: u32,
    source_file_idx: u32,
    annotations_off: u32,
    class_data_off: u32,
    static_value_off: u32,
    class_data: Option<ClassDataItem>
}

#[derive(Debug)]
pub struct EncodedField {
    field: String,
    access_flags: Vec<AccessFlag>,
}

#[derive(Debug)]
pub struct EncodedMethod {
    method_idx_diff: u32,
    access_flags: u32,
    code_off: u32,
}

#[derive(Debug)]
pub struct ClassDataItem {
    static_fields_size: u32,
    instance_fields_size: u32,
    direct_methods_size: u32,
    virtual_methods_size: u32,
    static_fields: Vec<EncodedField>,
    instance_fields: Vec<EncodedField>,
    direct_methods: Vec<EncodedMethod>,
    virtual_methods: Vec<EncodedMethod>,
}

#[derive(Debug)]
pub struct DexClasses {
    pub items: Vec<ClassDefItem>
}

impl DexClasses {
    pub fn build(dex_reader: &mut DexReader,
                 offset: u32,
                 size: u32,
                 types_list: &DexTypes,
                 fields_list: &DexFields,
                 methods_list: &DexMethods,
                 strings_list: &DexStrings) -> Self {
        dex_reader.bytes.seek(SeekFrom::Start(offset.into())).unwrap();

        let mut methods = Vec::new();

        for _ in 0..size {
            let class_idx = dex_reader.read_u32().unwrap();
            let access_flags = dex_reader.read_u32().unwrap();
            let superclass_idx = dex_reader.read_u32().unwrap();
            let interfaces_off = dex_reader.read_u32().unwrap();
            let source_file_idx = dex_reader.read_u32().unwrap();
            let annotations_off = dex_reader.read_u32().unwrap();
            let class_data_off = dex_reader.read_u32().unwrap();
            let static_value_off = dex_reader.read_u32().unwrap();

            // If class_data_off == 0 then we have no class data
            if class_data_off != 0 {
                // Start parse class data

                // Keep track of current stream position
                let current_offset = dex_reader.bytes.position();

                // Go to class data offset
                dex_reader.bytes.seek(SeekFrom::Start(class_data_off.into())).unwrap();

                let (static_fields_size, _)   = dex_reader.read_uleb128().unwrap();
                let (instance_fields_size, _) = dex_reader.read_uleb128().unwrap();
                let (direct_methods_size, _)  = dex_reader.read_uleb128().unwrap();
                let (virtual_methods_size, _) = dex_reader.read_uleb128().unwrap();

                let mut static_fields   = Vec::<EncodedField>::with_capacity(static_fields_size as usize);
                let mut instance_fields = Vec::<EncodedField>::with_capacity(instance_fields_size as usize);
                let mut direct_methods  = Vec::<String>::with_capacity(direct_methods_size as usize);
                let mut virtual_methods = Vec::<String>::with_capacity(virtual_methods_size as usize);

                // Encoded fields
                let mut field_idx = 0;
                for _ in 0..static_fields_size {
                    let (idx, read) = dex_reader.read_uleb128().unwrap();
                    let (access_flags, _) = dex_reader.read_uleb128().unwrap();

                    field_idx += idx;

                    let decoded_field = fields_list.items.get(field_idx as usize)
                                                         .unwrap()
                                                         .to_string();
                    let decoded_flags = AccessFlag::parse(access_flags, true);
                    static_fields.push(EncodedField {
                        field: decoded_field,
                        access_flags: decoded_flags
                    });
                }

                let mut field_idx = 0;
                for _ in 0..instance_fields_size {
                    let (idx, _) = dex_reader.read_uleb128().unwrap();
                    let (access_flags, _) = dex_reader.read_uleb128().unwrap();

                    field_idx += idx;

                    let decoded_field = fields_list.items.get(field_idx as usize)
                                                         .unwrap()
                                                         .to_string();
                    let decoded_flags = AccessFlag::parse(access_flags, true);
                    instance_fields.push(EncodedField {
                        field: decoded_field,
                        access_flags: decoded_flags
                    });
                }

                // Encoded methods
                let mut method_idx = 0;
                for _ in 0..direct_methods_size {
                    let (idx, _) = dex_reader.read_uleb128().unwrap();
                    let (access_flags, _) = dex_reader.read_uleb128().unwrap();
                    let (code_offset, _) = dex_reader.read_uleb128().unwrap();

                    method_idx += idx;

                    direct_methods.push(methods_list.items.get(method_idx as usize)
                                                        .unwrap()
                                                        .to_string());
                }

                let mut method_idx = 0;
                for _ in 0..virtual_methods_size {
                    let (idx, _) = dex_reader.read_uleb128().unwrap();
                    let (access_flags, _) = dex_reader.read_uleb128().unwrap();
                    let (code_offset, _) = dex_reader.read_uleb128().unwrap();

                    method_idx += idx;

                    virtual_methods.push(methods_list.items.get(method_idx as usize)
                                                        .unwrap()
                                                        .to_string());
                }

                // Go back to the previous offset
                dex_reader.bytes.seek(SeekFrom::Start(current_offset)).unwrap();
            }

            methods.push(ClassDefItem { 
                class_idx,
                access_flags,
                superclass_idx,
                interfaces_off,
                source_file_idx,
                annotations_off,
                class_data_off,
                static_value_off,
                class_data: None
            });
        }

        DexClasses { items: methods }
    }
}
