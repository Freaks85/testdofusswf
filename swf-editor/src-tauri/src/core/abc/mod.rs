/// ActionScript 3 ABC (ActionScript ByteCode) disassembler
///
/// This module provides basic disassembly of AS3 bytecode to human-readable format.
/// Based on AVM2 specification.

use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Clone)]
pub struct ABCFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub methods: Vec<MethodInfo>,
    pub metadata: Vec<MetadataInfo>,
    pub classes: Vec<ClassInfo>,
    pub scripts: Vec<ScriptInfo>,
    pub method_bodies: Vec<MethodBodyInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct ConstantPool {
    pub integers: Vec<i32>,
    pub uintegers: Vec<u32>,
    pub doubles: Vec<f64>,
    pub strings: Vec<String>,
    pub namespaces: Vec<NamespaceInfo>,
    pub namespace_sets: Vec<Vec<u32>>,
    pub multinames: Vec<MultinameInfo>,
}

#[derive(Debug, Clone)]
pub struct NamespaceInfo {
    pub kind: u8,
    pub name: u32,
}

#[derive(Debug, Clone)]
pub struct MultinameInfo {
    pub kind: u8,
    pub data: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: u32,
    pub param_count: u32,
    pub return_type: u32,
    pub param_types: Vec<u32>,
    pub flags: u8,
}

#[derive(Debug, Clone)]
pub struct MetadataInfo {
    pub name: u32,
    pub items: Vec<(u32, u32)>,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: u32,
    pub super_name: u32,
    pub flags: u8,
    pub traits: Vec<TraitInfo>,
}

#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub name: u32,
    pub kind: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ScriptInfo {
    pub init: u32,
    pub traits: Vec<TraitInfo>,
}

#[derive(Debug, Clone)]
pub struct MethodBodyInfo {
    pub method: u32,
    pub max_stack: u32,
    pub local_count: u32,
    pub init_scope_depth: u32,
    pub max_scope_depth: u32,
    pub code: Vec<u8>,
}

/// Parse ABC bytecode
pub fn parse_abc(data: &[u8]) -> Result<ABCFile, String> {
    let mut cursor = Cursor::new(data);

    // Read version
    let minor_version = cursor.read_u16::<LittleEndian>().map_err(|e| e.to_string())?;
    let major_version = cursor.read_u16::<LittleEndian>().map_err(|e| e.to_string())?;

    // Parse constant pool
    let constant_pool = parse_constant_pool(&mut cursor)?;

    // Parse methods
    let method_count = read_u30(&mut cursor)?;
    let mut methods = Vec::new();
    for _ in 0..method_count {
        methods.push(parse_method_info(&mut cursor)?);
    }

    // Parse metadata
    let metadata_count = read_u30(&mut cursor)?;
    let mut metadata = Vec::new();
    for _ in 0..metadata_count {
        metadata.push(parse_metadata_info(&mut cursor)?);
    }

    // Parse classes
    let class_count = read_u30(&mut cursor)?;
    let mut classes = Vec::new();
    for _ in 0..class_count {
        classes.push(parse_class_info(&mut cursor)?);
    }

    // Parse scripts
    let script_count = read_u30(&mut cursor)?;
    let mut scripts = Vec::new();
    for _ in 0..script_count {
        scripts.push(parse_script_info(&mut cursor)?);
    }

    // Parse method bodies
    let body_count = read_u30(&mut cursor)?;
    let mut method_bodies = Vec::new();
    for _ in 0..body_count {
        method_bodies.push(parse_method_body(&mut cursor)?);
    }

    Ok(ABCFile {
        minor_version,
        major_version,
        constant_pool,
        methods,
        metadata,
        classes,
        scripts,
        method_bodies,
    })
}

/// Disassemble ABC bytecode to readable text
pub fn disassemble_abc(abc: &ABCFile) -> String {
    let mut output = String::new();

    output.push_str(&format!("// ABC Version: {}.{}\n\n", abc.major_version, abc.minor_version));

    // Disassemble constant pool
    output.push_str("// ===== CONSTANT POOL =====\n");
    output.push_str(&format!("// {} strings, {} integers, {} doubles\n",
                            abc.constant_pool.strings.len(),
                            abc.constant_pool.integers.len(),
                            abc.constant_pool.doubles.len()));

    // Show some strings
    output.push_str("\n// Strings (first 20):\n");
    for (i, s) in abc.constant_pool.strings.iter().take(20).enumerate() {
        output.push_str(&format!("//   [{}] \"{}\"\n", i, escape_string(s)));
    }
    if abc.constant_pool.strings.len() > 20 {
        output.push_str(&format!("//   ... {} more strings\n", abc.constant_pool.strings.len() - 20));
    }

    // Disassemble classes
    output.push_str("\n// ===== CLASSES =====\n");
    for (i, class) in abc.classes.iter().enumerate() {
        let class_name = get_string(&abc.constant_pool, class.name);
        let super_name = get_string(&abc.constant_pool, class.super_name);
        output.push_str(&format!("\nclass {} extends {} {{\n", class_name, super_name));
        output.push_str(&format!("  // {} traits\n", class.traits.len()));
        for trait_info in &class.traits {
            let trait_name = get_string(&abc.constant_pool, trait_info.name);
            output.push_str(&format!("  // trait: {} (kind: {})\n", trait_name, trait_info.kind));
        }
        output.push_str("}\n");
    }

    // Disassemble methods
    output.push_str("\n// ===== METHODS =====\n");
    for (i, method) in abc.methods.iter().enumerate() {
        let method_name = get_string(&abc.constant_pool, method.name);
        output.push_str(&format!("\nmethod[{}] {} (", i, method_name));
        for (j, param) in method.param_types.iter().enumerate() {
            if j > 0 { output.push_str(", "); }
            output.push_str(&get_string(&abc.constant_pool, *param));
        }
        output.push_str(&format!(") : {}\n", get_string(&abc.constant_pool, method.return_type)));
    }

    // Disassemble method bodies
    output.push_str("\n// ===== METHOD BODIES =====\n");
    for (i, body) in abc.method_bodies.iter().enumerate() {
        output.push_str(&format!("\nmethod_body[{}] (method {})\n", i, body.method));
        output.push_str(&format!("  maxStack: {}, locals: {}, scopeDepth: {}-{}\n",
                                body.max_stack, body.local_count,
                                body.init_scope_depth, body.max_scope_depth));
        output.push_str("  code:\n");
        output.push_str(&disassemble_bytecode(&body.code, &abc.constant_pool));
    }

    output
}

/// Disassemble AVM2 bytecode instructions
fn disassemble_bytecode(code: &[u8], pool: &ConstantPool) -> String {
    let mut output = String::new();
    let mut pos = 0;

    while pos < code.len() {
        let opcode = code[pos];
        output.push_str(&format!("    {:04x}  ", pos));

        let (instruction, size) = disassemble_instruction(opcode, &code[pos..], pool);
        output.push_str(&instruction);
        output.push('\n');

        pos += size;
    }

    output
}

/// Disassemble a single AVM2 instruction
fn disassemble_instruction(opcode: u8, data: &[u8], pool: &ConstantPool) -> (String, usize) {
    let name = get_opcode_name(opcode);

    match opcode {
        // Simple instructions (no operands)
        0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 | 0x08 | 0x09 |
        0x0A | 0x20..=0x27 | 0x29 | 0x2A | 0x2B | 0x2C | 0x35 | 0x37 |
        0x38 | 0x39 | 0x3A | 0x48 | 0x49 | 0x4A | 0x4B | 0x4C | 0x4D |
        0x4E | 0x4F | 0x50 | 0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x56 |
        0x57 | 0x58 | 0x59 | 0x5A | 0x5B | 0x5C | 0x5D | 0x5E | 0x70 |
        0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x76 | 0x77 | 0x78 | 0x79 |
        0x7A | 0x80 | 0x81 | 0x82 | 0x85 | 0x86 | 0x87 | 0x90 | 0x91 |
        0x92 | 0x93 | 0x94 | 0x95 | 0x96 | 0x97 | 0xA0 | 0xA1 | 0xA2 |
        0xA3 | 0xA4 | 0xA5 | 0xA6 | 0xA7 | 0xA8 | 0xA9 | 0xAA | 0xAB |
        0xAC | 0xAD | 0xAE | 0xAF | 0xB0 | 0xB1 | 0xB2 | 0xB3 | 0xB4 |
        0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC4 | 0xC5 | 0xC6 | 0xC7 | 0xD0 |
        0xD1 | 0xD2 | 0xD3 | 0xD4 | 0xD5 | 0xD6 | 0xD7 | 0xEF | 0xF0 |
        0xF1 | 0xF2 | 0xF3 => {
            (name.to_string(), 1)
        }

        // pushbyte
        0x24 => {
            if data.len() >= 2 {
                (format!("{} {}", name, data[1] as i8), 2)
            } else {
                (name.to_string(), 1)
            }
        }

        // pushstring
        0x2C => {
            let (index, size) = read_u30_from_bytes(&data[1..]);
            let string = get_string(pool, index);
            (format!("{} \"{}\"", name, escape_string(&string)), size + 1)
        }

        // pushint, pushuint, pushdouble
        0x2D | 0x2E | 0x2F => {
            let (index, size) = read_u30_from_bytes(&data[1..]);
            (format!("{} {}", name, index), size + 1)
        }

        // getlocal, setlocal
        0x62 | 0x63 => {
            let (index, size) = read_u30_from_bytes(&data[1..]);
            (format!("{} {}", name, index), size + 1)
        }

        // jump, iftrue, iffalse, etc
        0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16 | 0x17 | 0x18 | 0x19 => {
            if data.len() >= 4 {
                let offset = i32::from_le_bytes([data[1], data[2], data[3], 0]);
                (format!("{} {}", name, offset), 4)
            } else {
                (name.to_string(), 1)
            }
        }

        // Default: unknown instruction
        _ => {
            (format!("{} ; unknown", name), 1)
        }
    }
}

/// Get opcode name
fn get_opcode_name(opcode: u8) -> &'static str {
    match opcode {
        0x01 => "bkpt",
        0x02 => "nop",
        0x03 => "throw",
        0x04 => "getsuper",
        0x05 => "setsuper",
        0x06 => "dxns",
        0x07 => "dxnslate",
        0x08 => "kill",
        0x09 => "label",
        0x0C => "ifnlt",
        0x0D => "ifnle",
        0x0E => "ifngt",
        0x0F => "ifnge",
        0x10 => "jump",
        0x11 => "iftrue",
        0x12 => "iffalse",
        0x13 => "ifeq",
        0x14 => "ifne",
        0x15 => "iflt",
        0x16 => "ifle",
        0x17 => "ifgt",
        0x18 => "ifge",
        0x19 => "ifstricteq",
        0x1A => "ifstrictne",
        0x1B => "lookupswitch",
        0x1C => "pushwith",
        0x1D => "popscope",
        0x1E => "nextname",
        0x1F => "hasnext",
        0x20 => "pushnull",
        0x21 => "pushundefined",
        0x23 => "nextvalue",
        0x24 => "pushbyte",
        0x25 => "pushshort",
        0x26 => "pushtrue",
        0x27 => "pushfalse",
        0x28 => "pushnan",
        0x29 => "pop",
        0x2A => "dup",
        0x2B => "swap",
        0x2C => "pushstring",
        0x2D => "pushint",
        0x2E => "pushuint",
        0x2F => "pushdouble",
        0x30 => "pushscope",
        0x40 => "newfunction",
        0x41 => "call",
        0x42 => "construct",
        0x46 => "callproperty",
        0x47 => "returnvoid",
        0x48 => "returnvalue",
        0x49 => "constructsuper",
        0x4A => "constructprop",
        0x4F => "callpropvoid",
        0x55 => "newobject",
        0x56 => "newarray",
        0x57 => "newactivation",
        0x58 => "newclass",
        0x59 => "getdescendants",
        0x5A => "newcatch",
        0x5D => "findpropstrict",
        0x5E => "findproperty",
        0x60 => "getlex",
        0x61 => "setproperty",
        0x62 => "getlocal",
        0x63 => "setlocal",
        0x64 => "getglobalscope",
        0x65 => "getscopeobject",
        0x66 => "getproperty",
        0x68 => "initproperty",
        0x6A => "deleteproperty",
        0x6C => "getslot",
        0x6D => "setslot",
        0x70 => "convert_s",
        0x71 => "esc_xelem",
        0x72 => "esc_xattr",
        0x73 => "convert_i",
        0x74 => "convert_u",
        0x75 => "convert_d",
        0x76 => "convert_b",
        0x77 => "convert_o",
        0x78 => "checkfilter",
        0x80 => "coerce",
        0x82 => "coerce_a",
        0x85 => "coerce_s",
        0x86 => "astype",
        0x87 => "astypelate",
        0x90 => "negate",
        0x91 => "increment",
        0x92 => "inclocal",
        0x93 => "decrement",
        0x94 => "declocal",
        0x95 => "typeof",
        0x96 => "not",
        0x97 => "bitnot",
        0xA0 => "add",
        0xA1 => "subtract",
        0xA2 => "multiply",
        0xA3 => "divide",
        0xA4 => "modulo",
        0xA5 => "lshift",
        0xA6 => "rshift",
        0xA7 => "urshift",
        0xA8 => "bitand",
        0xA9 => "bitor",
        0xAA => "bitxor",
        0xAB => "equals",
        0xAC => "strictequals",
        0xAD => "lessthan",
        0xAE => "lessequals",
        0xAF => "greaterthan",
        0xB0 => "greaterequals",
        0xB1 => "instanceof",
        0xB2 => "istype",
        0xB3 => "istypelate",
        0xB4 => "in",
        0xC0 => "increment_i",
        0xC1 => "decrement_i",
        0xC2 => "inclocal_i",
        0xC3 => "declocal_i",
        0xC4 => "negate_i",
        0xC5 => "add_i",
        0xC6 => "subtract_i",
        0xC7 => "multiply_i",
        0xD0 => "getlocal_0",
        0xD1 => "getlocal_1",
        0xD2 => "getlocal_2",
        0xD3 => "getlocal_3",
        0xD4 => "setlocal_0",
        0xD5 => "setlocal_1",
        0xD6 => "setlocal_2",
        0xD7 => "setlocal_3",
        0xEF => "debug",
        0xF0 => "debugline",
        0xF1 => "debugfile",
        _ => "unknown",
    }
}

// Helper functions

fn read_u30(cursor: &mut Cursor<&[u8]>) -> Result<u32, String> {
    let mut result = 0u32;
    for i in 0..5 {
        let byte = cursor.read_u8().map_err(|e| e.to_string())?;
        result |= ((byte & 0x7F) as u32) << (i * 7);
        if (byte & 0x80) == 0 {
            break;
        }
    }
    Ok(result)
}

fn read_u30_from_bytes(data: &[u8]) -> (u32, usize) {
    let mut result = 0u32;
    let mut size = 0;
    for i in 0..5 {
        if i >= data.len() {
            break;
        }
        let byte = data[i];
        result |= ((byte & 0x7F) as u32) << (i * 7);
        size += 1;
        if (byte & 0x80) == 0 {
            break;
        }
    }
    (result, size)
}

fn parse_constant_pool(cursor: &mut Cursor<&[u8]>) -> Result<ConstantPool, String> {
    let mut pool = ConstantPool::default();

    // Integers
    let int_count = read_u30(cursor)?;
    for _ in 1..int_count {
        let val = read_u30(cursor)? as i32;
        pool.integers.push(val);
    }

    // Unsigned integers
    let uint_count = read_u30(cursor)?;
    for _ in 1..uint_count {
        let val = read_u30(cursor)?;
        pool.uintegers.push(val);
    }

    // Doubles
    let double_count = read_u30(cursor)?;
    for _ in 1..double_count {
        let val = cursor.read_f64::<LittleEndian>().map_err(|e| e.to_string())?;
        pool.doubles.push(val);
    }

    // Strings
    let string_count = read_u30(cursor)?;
    pool.strings.push(String::from("")); // index 0 is always empty
    for _ in 1..string_count {
        let len = read_u30(cursor)?;
        let mut bytes = vec![0u8; len as usize];
        cursor.read_exact(&mut bytes).map_err(|e| e.to_string())?;
        let s = String::from_utf8_lossy(&bytes).to_string();
        pool.strings.push(s);
    }

    // Namespaces (simplified)
    let ns_count = read_u30(cursor)?;
    for _ in 1..ns_count {
        let kind = cursor.read_u8().map_err(|e| e.to_string())?;
        let name = read_u30(cursor)?;
        pool.namespaces.push(NamespaceInfo { kind, name });
    }

    // Namespace sets
    let ns_set_count = read_u30(cursor)?;
    for _ in 1..ns_set_count {
        let count = read_u30(cursor)?;
        let mut ns_set = Vec::new();
        for _ in 0..count {
            ns_set.push(read_u30(cursor)?);
        }
        pool.namespace_sets.push(ns_set);
    }

    // Multinames (simplified)
    let mn_count = read_u30(cursor)?;
    for _ in 1..mn_count {
        let kind = cursor.read_u8().map_err(|e| e.to_string())?;
        let mut data = Vec::new();

        match kind {
            0x07 | 0x0D => { // QName, QNameA
                data.push(read_u30(cursor)?);
                data.push(read_u30(cursor)?);
            }
            0x0F | 0x10 => { // RTQName, RTQNameA
                data.push(read_u30(cursor)?);
            }
            0x11 | 0x12 => { // RTQNameL, RTQNameLA
                // No data
            }
            0x09 | 0x0E => { // Multiname, MultinameA
                data.push(read_u30(cursor)?);
                data.push(read_u30(cursor)?);
            }
            0x1B | 0x1C => { // MultinameL, MultinameLA
                data.push(read_u30(cursor)?);
            }
            _ => {}
        }

        pool.multinames.push(MultinameInfo { kind, data });
    }

    Ok(pool)
}

fn parse_method_info(cursor: &mut Cursor<&[u8]>) -> Result<MethodInfo, String> {
    let param_count = read_u30(cursor)?;
    let return_type = read_u30(cursor)?;

    let mut param_types = Vec::new();
    for _ in 0..param_count {
        param_types.push(read_u30(cursor)?);
    }

    let name = read_u30(cursor)?;
    let flags = cursor.read_u8().map_err(|e| e.to_string())?;

    // Skip optional parameters and param names for simplicity

    Ok(MethodInfo {
        name,
        param_count,
        return_type,
        param_types,
        flags,
    })
}

fn parse_metadata_info(cursor: &mut Cursor<&[u8]>) -> Result<MetadataInfo, String> {
    let name = read_u30(cursor)?;
    let item_count = read_u30(cursor)?;

    let mut items = Vec::new();
    for _ in 0..item_count {
        let key = read_u30(cursor)?;
        let value = read_u30(cursor)?;
        items.push((key, value));
    }

    Ok(MetadataInfo { name, items })
}

fn parse_class_info(cursor: &mut Cursor<&[u8]>) -> Result<ClassInfo, String> {
    let name = read_u30(cursor)?;
    let super_name = read_u30(cursor)?;
    let flags = cursor.read_u8().map_err(|e| e.to_string())?;

    // Skip protectedNs, interfaces
    if (flags & 0x08) != 0 {
        read_u30(cursor)?;
    }

    let interface_count = read_u30(cursor)?;
    for _ in 0..interface_count {
        read_u30(cursor)?;
    }

    read_u30(cursor)?; // iinit

    let trait_count = read_u30(cursor)?;
    let mut traits = Vec::new();
    for _ in 0..trait_count {
        traits.push(parse_trait_info(cursor)?);
    }

    Ok(ClassInfo {
        name,
        super_name,
        flags,
        traits,
    })
}

fn parse_trait_info(cursor: &mut Cursor<&[u8]>) -> Result<TraitInfo, String> {
    let name = read_u30(cursor)?;
    let kind_byte = cursor.read_u8().map_err(|e| e.to_string())?;
    let kind = kind_byte & 0x0F;

    // Read trait data (simplified - just skip it)
    let mut data = Vec::new();
    match kind {
        0 | 6 => { // Slot, Const
            read_u30(cursor)?;
            read_u30(cursor)?;
            let vindex = read_u30(cursor)?;
            if vindex != 0 {
                cursor.read_u8().map_err(|e| e.to_string())?;
            }
        }
        1 | 2 | 3 => { // Method, Getter, Setter
            read_u30(cursor)?;
            read_u30(cursor)?;
        }
        4 => { // Class
            read_u30(cursor)?;
            read_u30(cursor)?;
        }
        5 => { // Function
            read_u30(cursor)?;
            read_u30(cursor)?;
        }
        _ => {}
    }

    // Skip metadata
    if (kind_byte & 0x40) != 0 {
        let metadata_count = read_u30(cursor)?;
        for _ in 0..metadata_count {
            read_u30(cursor)?;
        }
    }

    Ok(TraitInfo { name, kind, data })
}

fn parse_script_info(cursor: &mut Cursor<&[u8]>) -> Result<ScriptInfo, String> {
    let init = read_u30(cursor)?;

    let trait_count = read_u30(cursor)?;
    let mut traits = Vec::new();
    for _ in 0..trait_count {
        traits.push(parse_trait_info(cursor)?);
    }

    Ok(ScriptInfo { init, traits })
}

fn parse_method_body(cursor: &mut Cursor<&[u8]>) -> Result<MethodBodyInfo, String> {
    let method = read_u30(cursor)?;
    let max_stack = read_u30(cursor)?;
    let local_count = read_u30(cursor)?;
    let init_scope_depth = read_u30(cursor)?;
    let max_scope_depth = read_u30(cursor)?;

    let code_length = read_u30(cursor)?;
    let mut code = vec![0u8; code_length as usize];
    cursor.read_exact(&mut code).map_err(|e| e.to_string())?;

    // Skip exception info and traits
    let exception_count = read_u30(cursor)?;
    for _ in 0..exception_count {
        read_u30(cursor)?; // from
        read_u30(cursor)?; // to
        read_u30(cursor)?; // target
        read_u30(cursor)?; // exc_type
        read_u30(cursor)?; // var_name
    }

    let trait_count = read_u30(cursor)?;
    for _ in 0..trait_count {
        parse_trait_info(cursor)?;
    }

    Ok(MethodBodyInfo {
        method,
        max_stack,
        local_count,
        init_scope_depth,
        max_scope_depth,
        code,
    })
}

fn get_string(pool: &ConstantPool, index: u32) -> String {
    if index == 0 || index as usize >= pool.strings.len() {
        return format!("?{}", index);
    }
    pool.strings[index as usize].clone()
}

fn escape_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            c if c.is_control() => format!("\\x{:02x}", c as u8),
            c => c.to_string(),
        })
        .collect()
}
