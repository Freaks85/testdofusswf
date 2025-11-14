// AS3 (AVM2) decompiler
use crate::core::abc::{parse_abc, disassemble_abc};

/// Decompile AS3 bytecode (ABC format)
pub fn decompile(bytecode: &[u8]) -> Result<String, String> {
    if bytecode.is_empty() {
        return Ok("// Empty bytecode".to_string());
    }

    // Try to parse as ABC
    match parse_abc(bytecode) {
        Ok(abc) => {
            // Successfully parsed - disassemble it
            Ok(disassemble_abc(&abc))
        }
        Err(e) => {
            // Parsing failed - show hex dump as fallback
            let hex_dump = bytecode
                .iter()
                .enumerate()
                .map(|(i, b)| {
                    if i % 16 == 0 {
                        format!("\n{:04x}: {:02x}", i, b)
                    } else {
                        format!(" {:02x}", b)
                    }
                })
                .collect::<String>();

            Ok(format!(
                "// Failed to parse ABC: {}\n// Showing raw bytecode instead\n// Length: {} bytes\n{}",
                e,
                bytecode.len(),
                hex_dump
            ))
        }
    }
}
