// ActionScript decompiler module
// TODO: Implement AS3, AS1/2 decompilation

pub mod as3;
pub mod as1_2;

use crate::core::types::ScriptType;

/// Decompile bytecode to readable code
pub fn decompile_script(bytecode: &[u8], script_type: ScriptType) -> Result<String, String> {
    match script_type {
        ScriptType::AS3 => as3::decompile(bytecode),
        ScriptType::AS1 | ScriptType::AS2 => as1_2::decompile(bytecode),
    }
}
