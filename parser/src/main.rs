use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::ops::Div;
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;
use regex::Regex;
use std::fmt;

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssemblerError::IoError(e) => write!(f, "IO error: {}", e),
            AssemblerError::ParseIntError => write!(f, "Error parsing integer"),
            AssemblerError::UnknownInstruction(s) => write!(f, "Unknown instruction: {}", s),
            AssemblerError::LabelNotFound(s) => write!(f, "Label not found: {}", s),
        }
    }
}

#[derive(Debug)]
enum AssemblerError {
    IoError(io::Error),
    ParseIntError,
    UnknownInstruction(String),
    LabelNotFound(String),
}

impl From<io::Error> for AssemblerError {
    fn from(error: io::Error) -> Self {
        AssemblerError::IoError(error)
    }
}

type Result<T> = std::result::Result<T, AssemblerError>;

struct Assembler {
    labels: HashMap<String, i32>,
    pc: i32,
}

impl Assembler {
    fn new() -> Self {
        Assembler {
            labels: HashMap::new(),
            pc: 0,
        }
    }

    fn replace_all(s: &mut String, what: &str, with: &str) {
        while let Some(pos) = s.find(what) {
            s.replace_range(pos..pos + what.len(), with);
        }
    }

    fn parse_register(s: &str) -> Result<i32> {
        let mut s = s.to_string();
        Self::replace_all(&mut s, ",", "");
        let s_upper = s.to_uppercase();
        Ok(match s_upper.as_str() {
            "R0" => 0b000,
            "R1" => 0b001,
            "R2" => 0b010,
            "R3" => 0b011,
            "R4" => 0b100,
            "R5" => 0b101,
            "R6" => 0b110,
            "R7" => 0b111,
            "R8" => 0b1000,
            "R9" => 0b1001,
            "R10" => 0b1010,
            "R11" => 0b1011,
            "R12" => 0b1100,
            "R13" => 0b1101,
            "R14" => 0b1110,
            "R15" => 0b1111,
            "SP" => 0b1101,
            "LR" => 0b1110,
            "PC" => 0b1111,
            _ => return Err(AssemblerError::UnknownInstruction(format!("register not recognized: {}", s))),
        })
    }

    fn parse_imm<T: FromStr + std::ops::Div<Output=T> + From<i32>>(s: &str, sp: bool) -> Result<T> {
        let mut s = s.to_string();
        Self::replace_all(&mut s, "]", "");
        if !s.is_empty() && s.starts_with('#') {
            let s = s[1..].parse::<i32>().map_err(|_| AssemblerError::ParseIntError)?;
            Ok(s.div(if sp { 4 } else { 1 }).into())
        } else {
            Err(AssemblerError::UnknownInstruction("immediate not parsed".to_string()))
        }
    }

    fn parse_condition(s: &str) -> Result<i32> {
        let s_upper = s.to_uppercase();
        let result = match s_upper.as_str() {
            "EQ" => 0b0000,
            "NE" => 0b0001,
            "CS" => 0b0010,
            "CC" | "LO" => 0b0011,
            "MI" => 0b0100,
            "PL" => 0b0101,
            "VS" => 0b0110,
            "VC" => 0b0111,
            "HI" => 0b1000,
            "LS" => 0b1001,
            "GE" => 0b1010,
            "LT" => 0b1011,
            "GT" => 0b1100,
            "LE" => 0b1101,
            "AL" => 0b1110,
            _ => return Err(AssemblerError::UnknownInstruction(format!("condition not recognized: {}", s))),
        };
        Ok(result.into())
    }
    fn convert_instruction(instruction: &str, args: &[&str], pc: i32,labels: HashMap<String, i32>) -> Result<i32> {
        match instruction {
            "lsls" | "lsrs" => {
                if args.len() == 3 {
                    // Immediate (5bits)
                    // LSLS <Rd > , <Rm> ,# <imm5>
                    // 15 14 13 12 11 | 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  0  0  0  0 |       imm5 |    Rm |    Rd
                    let imm5 = Self::parse_imm::<i32>(&args[2], false)?;
                    let rm = Self::parse_register(&args[1])?;
                    let rd = Self::parse_register(&args[0])?;
                    Ok(0b00000_00000_000_000 | (imm5 << 6) | (rm << 3) | rd)
                } else if args.len() == 2 {
                    // Registers
                    // LSLS <Rdn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
                    //  0  1  0  0  0  0 0 0 1 0    Rm   Rdn
                    let rm = Self::parse_register(&args[1])?;
                    let rdn = Self::parse_register(&args[0])?;
                    Ok(0b0100_0000_10_000_000 | (rm << 3) | rdn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "adds" => {
                if args[0] != "sp" && args[1] != "sp" {
                    if args.len() == 3 && args[2].starts_with('#') {
                        // Immediate (3bits)
                        // ADDS <Rd > , < Rn > , <#imm3>
                        // 15 14 13 12 11 10 9 | 8 7 6 | 5 4 3 | 2 1 0
                        //  0  0  0  1  1  1 0 |  Imm3 |    Rn |    Rd
                        let imm3 = Self::parse_imm::<i32>(&args[2], false)?;
                        let rn = Self::parse_register(&args[1])?;
                        let rd = Self::parse_register(&args[0])?;
                        Ok(0b0001110_000_000_000 | (imm3 << 6) | (rn << 3) | rd)
                    } else if args.len() == 3 && args[2].starts_with('r') {
                        // Registers
                        // ADDS <Rd > , < Rn > , <Rm>
                        // 15 14 13 12 11 10 9 | 8 7 6 | 5 4 3 | 2 1 0
                        //  0  0  0  1  1  0 0 |    Rm |    Rn |    Rd
                        let rm = Self::parse_register(&args[2])?;
                        let rn = Self::parse_register(&args[1])?;
                        let rd = Self::parse_register(&args[0])?;
                        Ok(0b0001100_000_000_000 | (rm << 6) | (rn << 3) | rd)
                    } else {
                        // Immediate (8bits)
                        // ADDS <Rdn > , [ < Rdn > , ] #<imm8>
                        // 15 14 13 12 11 | 10 9 8 | 7 6 5 4 3 2 1 0
                        //  0  0  1 1  0 |     Rd |            imm8
                        let imm8 = Self::parse_imm::<i32>(&args[1], false)?;
                        let rdn = Self::parse_register(&args[0])?;
                        Ok(0b00110_000_00000000 | (rdn << 8) | imm8)
                    }
                } else if args[0] == "sp" {
                    // SP + Immediate (7bits)
                    // ADD [ SP , ] SP,# <offset>
                    // 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
                    //  1  0  1  1  0  0 0 0 0 |          imm7
                    let imm7 = Self::parse_imm::<i32>(&args[1], true)?;
                    Ok(0b101100000_0000000 | imm7)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "sub" => {
                if args.len() == 3 {
                    if args[2].starts_with('#') {
                        // Immediate (3bits)
                        // SUBS <Rd > , < Rn> ,# <imm3>
                        // 15 14 13 12 11 10 9 | 8 7 6 | 5 4 3 | 2 1 0
                        //  0  0  0  1  1  1 1 |  Imm3 |    Rn |    Rd
                        let imm3 = Self::parse_imm::<i32>(&args[2], false)?;
                        let rn = Self::parse_register(&args[1])?;
                        let rd = Self::parse_register(&args[0])?;
                        Ok(0b0001111_000_000_000 | (imm3 << 6) | (rn << 3) | rd)
                    } else {
                        // Registers
                        // SUBS <Rd > , < Rn > , <Rm>
                        // 15 14 13 12 11 10 9 | 8 7 6 | 5 4 3 | 2 1 0
                        //  0  0  0  1  1  0 1 |    Rm |    Rn |    Rd
                        let rm = Self::parse_register(&args[2])?;
                        let rn = Self::parse_register(&args[1])?;
                        let rd = Self::parse_register(&args[0])?;
                        Ok(0b0001101_000_000_000 | (rm << 6) | (rn << 3) | rd)
                    }
                } else if args.len() == 2 {
                    if args[0] != "sp" {
                        // Immediate (8bits)
                        // SUBS <Rdn > , [ < Rdn > , ] #<imm8>
                        // 15 14 13 12 11 | 10 9 8 | 7 6 5 4 3 2 1 0
                        //  0  0  1 1  1 |     Rd |            imm8
                        let imm8 = Self::parse_imm::<i32>(&args[1], false)?;
                        let rdn = Self::parse_register(&args[0])?;
                        Ok(0b00111_000_00000000 | (rdn << 8) | imm8)
                    } else {
                        // Immediate (7bits)
                        // SUB [ SP , ] SP,# <offset>
                        // 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
                        //  1  0  1  1  0  0 0 0 1 |          imm7
                        let imm7 = Self::parse_imm::<i32>(&args[1], true)?;
                        Ok(0b101100001_0000000 | imm7)
                    }
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "movs" => {
                if args.len() == 2 {
                    if args[1].starts_with('#') {
                        // Immediate (8bits)
                        // MOVS <Rd> ,# <imm8>
                        // 15 14 13 12 11 | 10 9 8 | 7 6 5 4 3 2 1 0
                        //  0  0  1  0  0 |     Rd |            imm8
                        let imm8 = Self::parse_imm::<i32>(&args[1], false)?;
                        let rd = Self::parse_register(&args[0])?;
                        Ok(0b00100_000_00000000 | (rd << 8) | imm8)
                    } else {
                        // Registers
                        // MOVS <Rd> , <Rn>
                        // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                        //  0  0  0  0  0  0 0 0 0 0 |    Rd |    Rn
                        let rn = Self::parse_register(&args[1])?;
                        let rd = Self::parse_register(&args[0])?;
                        Ok(0b0000000000_000_000 | (rd << 3) | rn)
                    }
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "cmp" => {
                if args.len() == 2 {
                    if args[1].starts_with('#') {
                        // Immediate (8bits)
                        // CMP <Rd> ,# <imm8>
                        // 15 14 13 12 11 | 10 9 8 | 7 6 5 4 3 2 1 0
                        //  0  0  1  0  1 |     Rd |            imm8
                        let imm8 = Self::parse_imm::<i32>(&args[1], false)?;
                        let rd = Self::parse_register(&args[0])?;
                        Ok(0b00101_000_00000000 | (rd << 8) | imm8)
                    } else {
                        // Registers
                        // CMP <Rn > , <Rm>
                        // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                        //  0  0  0  0  0  0 1 0 1 0 |    Rm |    Rn
                        let rm = Self::parse_register(&args[1])?;
                        let rn = Self::parse_register(&args[0])?;
                        Ok(0b0100001010_000_000 | (rm << 3) | rn)
                    }
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "and" => {
                if args.len() == 2 {
                    // AND <Rdn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  1  0  0  0  0 0 0 0 0 |    Rm |    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b0100000000_000_000 | (rm << 3) | rn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "eors" => {
                if args.len() == 2 {
                    // EORS <Rdn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  1  0  0  0  0 0 0 0 1 |    Rm |    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b0100000001_000_000 | (rm << 3) | rn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "adcs" => {
                if args.len() == 2 {
                    // ADC <Rdn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  1  0  0  0  0 0 1 0 1 |    Rm |    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b0100000101_000_000 | (rm << 3) | rn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "asrs" => {
                if args.len() == 2 {
                    // ASRS <Rdn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  1  0  0  0  0 0 1 0 0 |    Rm |    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b01000000100_000_000 | (rm << 3) | rn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "sbcs" => {
                if args.len() == 2 {
                    // SBCS <Rdn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  1  0  0  0  0 0 1 1 0 |    Rm |    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b0100000110_000_000 | (rm << 3) | rn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "rors" => {
                if args.len() == 2 {
                    // RORS <Rdn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  1  0  0  0  0 0 1 1 1 |    Rm |    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b0100000111_000_000 | (rm << 3) | rn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "tst" => {
                if args.len() == 2 {
                    // TST <Rn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  1  0  0  0  0 1 0 0 0 |    Rm |    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b0100001000_000_000 | (rm << 3) | rn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "rsbs" => {
                if args.len() == 2 {
                    // RSBS <Rd > , <Rn> ,#0
                    // 15 14 13 12 11 10 9 8 7 6 5 | 4 3 | 2 1 0
                    //  0  1  0  0  0  0 1 0 0 1   |  Rn |    Rd
                    let rn = Self::parse_register(&args[1])?;
                    let rd = Self::parse_register(&args[0])?;
                    Ok(0b0100001001_000_000 | (rn << 3) | rd)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "cmn" => {
                if args.len() == 2 {
                    // CMN <Rn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  1  0  0  0  0 1 0 1 1 |    Rm |    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b0100001011_000_000 | (rm << 3) | rn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "orrs" => {
                if args.len() == 2 {
                    // ORRS <Rdn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  1  0  0  0  0 1 1 0 0 |    Rm |    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b0100001100_000_000 | (rm << 3) | rn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "muls" => {
                if args.len() == 3 {
                    // MULS <Rdm> , < Rn > , <Rdm>
                    // 15 14 13 12 11 10 9 8 7 6 5 | 4 3 | 2 1 0
                    //  0  1  0  0  0  0 1 1 0 1   |  Rn |    Rd
                    let rn = Self::parse_register(&args[1])?;
                    let rd = Self::parse_register(&args[0])?;
                    let rm = Self::parse_register(&args[2])?;
                    Ok(0b0100001101_000_000 | (rn << 3) | rd)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "bics" => {
                if args.len() == 2 {
                    // BICS <Rdn > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                    //  0  1  0  0  0  0 1 1 1 0 |    Rm |    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b0100001110_000_000 | (rm << 3) | rn)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "mvns" => {
                if args.len() == 2 {
                    // MVNS <Rd > , <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 5 | 4 3 | 2 1 0
                    //  0  1  0  0  0  0 1 1 1 1   |  Rn |    Rd
                    let rd = Self::parse_register(&args[0])?;
                    let rm = Self::parse_register(&args[1])?;
                    Ok(0b0100001111_000_000 | (rm << 3) | rd)
                } else {
                    Err(AssemblerError::UnknownInstruction("Invalid number of arguments".to_string()))
                }
            }
            "str" => {
                // STR <Rt > , [ SP,# <offset> ]
                // 15 14 13 12 11 10 9 8 7 6 5 | 4 3 | 2 1 0
                //  1  0  0  1  0     Rt            imm8
                let bits = 0b10010_000_00000000;
                if args.len() >= 3 {
                    let imm8 = Self::parse_imm::<i32>(&args[2], false)?/4;
                    let rt = Self::parse_register(&args[0])?;
                    Ok(bits | (rt << 8) | imm8)
                } else {
                    Ok(bits | (Self::parse_register(&args[0])? << 8))
                }
            }
            "ldr" => {
                // LDR <Rt > , [ SP{ , # <offset> } ]
                // 15 14 13 12 11 10 9 8 7 6 5 | 4 3 | 2 1 0
                //  1  0  0  1  1     Rt            imm8
                let bits = 0b10011_000_00000000;
                if args.len() >= 3 {
                    let imm8 = Self::parse_imm::<i32>(&args[2], false)?/4;
                    let rt = Self::parse_register(&args[0])?;
                    Ok(bits | (rt << 8) | imm8)
                } else {
                    Ok(bits | (Self::parse_register(&args[0])? << 8))
                }
            }
            "cpm" => {
                if args[1].starts_with("#") {
                    // CPM <Rd> ,# <imm8>
                    // 15 14 13 12 11 10 9 8 7 6 5 | 4 3 | 2 1 0
                    // 0 0 1 0 1  Rd            imm8
                    let imm8 = Self::parse_imm::<i32>(&args[1], false)?;
                    let rd = Self::parse_register(&args[0])?;
                    Ok(0b00101_000_00000000 | (rd << 8) | imm8)
                } else {
                    // CPM <Rn>, <Rm>
                    // 15 14 13 12 11 10 9 8 7 6 5 | 4 3 | 2 1 0
                    // 0 1 0 0 0 0 1 0 1 0   Rm    Rn
                    let rm = Self::parse_register(&args[1])?;
                    let rn = Self::parse_register(&args[0])?;
                    Ok(0b0100001010_000_000 | (rm << 3) | rn)
                }
            }

            "b" | "bx" => {
                //jumps
                // B <label>
                // 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
                //  1  1  0  1  0                  imm11
                let bitset = 0b11100_00000000000;
                let label = Self::parse_label( &args[0], pc,labels)?;
                Ok(bitset | label)
            }
            "bl" => {
                // BL <label>
                // 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
                //  1  1  1  0  1                  imm11
                let bitset = 0b11010_00000000000;
                let condition = Self::parse_condition(&args[0])?;
                let label = Self::parse_label( &args[0], pc,labels)?;

                Ok(bitset | condition | label)
            }
            "bic" => {
                // BIC <Rdn > , <Rm>
                // 15 14 13 12 11 10 9 8 7 6 | 5 4 3 | 2 1 0
                //  0  1  0  0  0  0 1 1 0 0 |    Rm |    Rn
                let bitset = 0b11010_00000000000;
                let condition = Self::parse_condition(&args[0])?;
                let label = Self::parse_label(&args[0], pc,labels)?;

                Ok(bitset | condition | label)
            }
            "bne" => {
                // BNE <label>
                // 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
                //  1  1  0  1  1                  imm11
                let imm11 = Self::parse_imm::<i32>(&args[0], false)?;
                let bitset = 0b11010_00000000000;
                let condition = Self::parse_condition(&args[0])?;
                let label = Self::parse_label( &args[0], pc,labels)?;

                Ok(bitset | condition | label)
            }

            // Handle other instructions similarly
            _ => Err(AssemblerError::UnknownInstruction(format!("unknown instruction {}", instruction))),
        }
    }

    fn parse_label(s: &str, source: i32,labels: HashMap<String, i32>) -> Result<i32> {
        let label = labels.get(s).ok_or(AssemblerError::LabelNotFound(s.to_string()))?;
        Ok(label - source - 3)
    }
}

fn main() -> io::Result<()> {
    let start_time = Instant::now();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "No input file provided"));
    }

    let input_path = Path::new(&args[1]);
    if !input_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("File {} does not exist", input_path.display())));
    }

    let output_path = input_path.with_extension("bin");
    let mut output_file = File::create(&output_path)?;

    let label_regex = Regex::new(r"^.*:$").unwrap();
    let instruction_regex = Regex::new(r"^\t?[^.@\t].*[^:]$").unwrap();

    let mut pc = 0;
    let mut labels = std::collections::HashMap::new();

    let input_file = File::open(&input_path)?;
    let reader = BufReader::new(input_file);

    for line in reader.lines() {
        let line = line?;
        if label_regex.is_match(&line) {
            let label = line.trim_end_matches(':');
            labels.insert(label.to_string(), pc);
        } else if instruction_regex.is_match(&line) {
            pc += 1;
        }
    }
    let input_file = File::open(&input_path)?;
    let reader = BufReader::new(input_file);
    for line in reader.lines() {
        let line = line?;
        let mut line = line.replace("[", "");
        line = line.replace("]", "");

        println!("{}", line);
        if line.starts_with("run") {
            continue;
        } else if line.starts_with("\t.") || line.starts_with(".") {
            continue;
        } else if line.is_empty() {
            continue;
        } else if line.starts_with("\t@") {
            continue;
        } else if instruction_regex.is_match(&line) {
            let words: Vec<&str> = line.split_whitespace().collect();
            let instruction = words[0];
            let args = &words[1..];
            //pass labels as a copy to avoid borrowing issues
            let instruction = Assembler::convert_instruction(instruction, args, pc, labels.clone())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            //in hexa
            let hexa = format!("{:04X}", instruction);
            output_file.write_all(hexa.as_bytes())?;
            output_file.write_all(b" ")?;
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid line: {}", line)));
        }
    }


    let duration = start_time.elapsed();
    println!("Assembled in {} seconds", duration.as_secs());

    Ok(())
}
