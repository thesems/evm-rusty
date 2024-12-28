use alloy_primitives::U256;

// Gas cost calculation helper structs
pub struct GasCost {
    base: u64,               // Base cost of operation
    dynamic_multiplier: u64, // For operations with dynamic costs
}

// Stack requirements for operation
pub struct StackReq {
    pub min_stack_height: u32, // Required minimum stack height
    pub stack_inputs: u32,     // Number of items taken from stack
    pub stack_outputs: u32,    // Number of items pushed to stack
}

#[derive(Debug, Clone)]
pub enum Operation {
    // 0x00 - 0x0f: Stop and Arithmetic
    Stop,
    Add,
    Mul,
    Sub,
    Div,
    SDiv,
    Mod,
    SMod,
    AddMod,
    MulMod,
    Exp,
    SignExtend,

    // 0x10 - 0x1f: Comparison & Bitwise Logic
    Lt,
    Gt,
    Slt,
    Sgt,
    Eq,
    IsZero,
    And,
    Or,
    Xor,
    Not,
    Byte,
    Shl,
    Shr,
    Sar,

    // 0x30 - 0x3f: Environmental Information
    Address,
    Balance,
    Origin,
    Caller,
    CallValue,
    CallDataLoad,
    CallDataSize,
    CallDataCopy,
    CodeSize,
    CodeCopy,
    GasPrice,
    ExtCodeSize,
    ExtCodeCopy,
    ReturnDataSize,
    ReturnDataCopy,
    ExtCodeHash,

    // 0x40 - 0x4f: Block Information
    BlockHash,
    Coinbase,
    Timestamp,
    Number,
    Difficulty,
    GasLimit,
    ChainId,
    SelfBalance,
    BaseFee,

    // 0x50 - 0x5f: Stack, Memory, Storage and Flow
    Pop,
    MLoad,
    MStore,
    MStore8,
    SLoad,
    SStore,
    Jump,
    JumpI,
    PC,
    MSize,
    Gas,
    JumpDest,

    // 0x60 - 0x7f: Push Operations
    Push1(U256),
    Push2(U256),
    Push3(U256),
    Push4(U256),
    Push5(U256),
    Push6(U256),
    Push7(U256),
    Push8(U256),
    Push9(U256),
    Push10(U256),
    Push11(U256),
    Push12(U256),
    Push13(U256),
    Push14(U256),
    Push15(U256),
    Push16(U256),
    Push17(U256),
    Push18(U256),
    Push19(U256),
    Push20(U256),
    Push21(U256),
    Push22(U256),
    Push23(U256),
    Push24(U256),
    Push25(U256),
    Push26(U256),
    Push27(U256),
    Push28(U256),
    Push29(U256),
    Push30(U256),
    Push31(U256),
    Push32(U256),

    // 0x80 - 0x8f: Duplication Operations 1-16
    Dup(u8),

    // 0x90 - 0x9f: Exchange Operations
    Swap1,
    Swap2,
    Swap3,
    Swap4,
    Swap5,
    Swap6,
    Swap7,
    Swap8,
    Swap9,
    Swap10,
    Swap11,
    Swap12,
    Swap13,
    Swap14,
    Swap15,
    Swap16,

    // 0xa0 - 0xaf: Logging Operations
    Log0,
    Log1,
    Log2,
    Log3,
    Log4,

    // 0xf0 - 0xff: System Operations
    Create,
    Call,
    CallCode,
    Return,
    DelegateCall,
    Create2,
    StaticCall,
    Revert,
    Invalid,
    SelfDestruct,
}

impl Operation {
    // Convert a byte to Operation
    pub fn from_byte(byte: u8, data: Option<U256>) -> Result<Self, String> {
        match byte {
            // 0x00 - 0x0f: Stop and Arithmetic
            0x00 => Ok(Operation::Stop),
            0x01 => Ok(Operation::Add),
            0x02 => Ok(Operation::Mul),
            0x03 => Ok(Operation::Sub),
            0x04 => Ok(Operation::Div),
            0x05 => Ok(Operation::SDiv),
            0x06 => Ok(Operation::Mod),
            0x07 => Ok(Operation::SMod),
            0x08 => Ok(Operation::AddMod),
            0x09 => Ok(Operation::MulMod),
            0x0a => Ok(Operation::Exp),
            0x0b => Ok(Operation::SignExtend),

            // 0x10 - 0x1f: Comparison & Bitwise Logic
            0x10 => Ok(Operation::Lt),
            0x11 => Ok(Operation::Gt),
            0x12 => Ok(Operation::Slt),
            0x13 => Ok(Operation::Sgt),
            0x14 => Ok(Operation::Eq),
            0x15 => Ok(Operation::IsZero),
            0x16 => Ok(Operation::And),
            0x17 => Ok(Operation::Or),
            0x18 => Ok(Operation::Xor),
            0x19 => Ok(Operation::Not),
            0x1a => Ok(Operation::Byte),
            0x1b => Ok(Operation::Shl),
            0x1c => Ok(Operation::Shr),
            0x1d => Ok(Operation::Sar),

            // 0x30 - 0x3f: Environmental Information
            0x30 => Ok(Operation::Address),
            0x31 => Ok(Operation::Balance),
            0x32 => Ok(Operation::Origin),
            0x33 => Ok(Operation::Caller),
            0x34 => Ok(Operation::CallValue),
            0x35 => Ok(Operation::CallDataLoad),
            0x36 => Ok(Operation::CallDataSize),
            0x37 => Ok(Operation::CallDataCopy),
            0x38 => Ok(Operation::CodeSize),
            0x39 => Ok(Operation::CodeCopy),
            0x3a => Ok(Operation::GasPrice),
            0x3b => Ok(Operation::ExtCodeSize),
            0x3c => Ok(Operation::ExtCodeCopy),
            0x3d => Ok(Operation::ReturnDataSize),
            0x3e => Ok(Operation::ReturnDataCopy),
            0x3f => Ok(Operation::ExtCodeHash),

            // 0x40 - 0x4f: Block Information
            0x40 => Ok(Operation::BlockHash),
            0x41 => Ok(Operation::Coinbase),
            0x42 => Ok(Operation::Timestamp),
            0x43 => Ok(Operation::Number),
            0x44 => Ok(Operation::Difficulty),
            0x45 => Ok(Operation::GasLimit),
            0x46 => Ok(Operation::ChainId),
            0x47 => Ok(Operation::SelfBalance),
            0x48 => Ok(Operation::BaseFee),

            // 0x50 - 0x5f: Stack, Memory, Storage and Flow
            0x50 => Ok(Operation::Pop),
            0x51 => Ok(Operation::MLoad),
            0x52 => Ok(Operation::MStore),
            0x53 => Ok(Operation::MStore8),
            0x54 => Ok(Operation::SLoad),
            0x55 => Ok(Operation::SStore),
            0x56 => Ok(Operation::Jump),
            0x57 => Ok(Operation::JumpI),
            0x58 => Ok(Operation::PC),
            0x59 => Ok(Operation::MSize),
            0x5a => Ok(Operation::Gas),
            0x5b => Ok(Operation::JumpDest),

            // Push operations (0x60 - 0x7f)
            n @ 0x60..=0x7f => {
                // Check if we have data for push operations
                match data {
                    Some(value) => Ok(match n - 0x60 {
                        0 => Operation::Push1(value),
                        1 => Operation::Push2(value),
                        2 => Operation::Push3(value),
                        3 => Operation::Push4(value),
                        4 => Operation::Push5(value),
                        5 => Operation::Push6(value),
                        6 => Operation::Push7(value),
                        7 => Operation::Push8(value),
                        8 => Operation::Push9(value),
                        9 => Operation::Push10(value),
                        10 => Operation::Push11(value),
                        11 => Operation::Push12(value),
                        12 => Operation::Push13(value),
                        13 => Operation::Push14(value),
                        14 => Operation::Push15(value),
                        15 => Operation::Push16(value),
                        16 => Operation::Push17(value),
                        17 => Operation::Push18(value),
                        18 => Operation::Push19(value),
                        19 => Operation::Push20(value),
                        20 => Operation::Push21(value),
                        21 => Operation::Push22(value),
                        22 => Operation::Push23(value),
                        23 => Operation::Push24(value),
                        24 => Operation::Push25(value),
                        25 => Operation::Push26(value),
                        26 => Operation::Push27(value),
                        27 => Operation::Push28(value),
                        28 => Operation::Push29(value),
                        29 => Operation::Push30(value),
                        30 => Operation::Push31(value),
                        31 => Operation::Push32(value),
                        _ => return Err("Invalid push operation".to_string()),
                    }),
                    None => Err("Push operation requires data".to_string()),
                }
            }

            // Dup operations (0x80 - 0x8f)
            n @ 0x80..=0x8f => Ok(Operation::Dup((n - 0x80 + 1) as u8)),

            // Swap operations (0x90 - 0x9f)
            0x90 => Ok(Operation::Swap1),
            0x91 => Ok(Operation::Swap2),
            // ... add all swap operations
            0x9f => Ok(Operation::Swap16),

            // Add remaining operations...
            _ => Err(format!("Unknown opcode: 0x{:02x}", byte)),
        }
    }

    // Get the number of additional bytes needed for this operation
    pub fn additional_bytes(&self) -> usize {
        match self {
            Operation::Push1(_) => 1,
            Operation::Push2(_) => 2,
            Operation::Push3(_) => 3,
            // ... and so on for all push operations
            Operation::Push32(_) => 32,
            _ => 0,
        }
    }

    // Get the gas cost for this operation
    pub fn gas_cost(&self) -> GasCost {
        match self {
            // Zero gas operations
            Operation::Stop | Operation::Return => GasCost {
                base: 0,
                dynamic_multiplier: 0,
            },

            // Very low gas operations (3 gas)
            Operation::Add
            | Operation::Sub
            | Operation::Not
            | Operation::Lt
            | Operation::Gt
            | Operation::Slt
            | Operation::Sgt
            | Operation::Eq
            | Operation::IsZero
            | Operation::And
            | Operation::Or
            | Operation::Xor
            | Operation::Pop => GasCost {
                base: 3,
                dynamic_multiplier: 0,
            },

            // Low gas operations (5 gas)
            Operation::Mul
            | Operation::Div
            | Operation::SDiv
            | Operation::Mod
            | Operation::SMod
            | Operation::SignExtend => GasCost {
                base: 5,
                dynamic_multiplier: 0,
            },

            // Mid gas operations
            Operation::AddMod | Operation::MulMod => GasCost {
                base: 8,
                dynamic_multiplier: 0,
            },

            // Storage operations
            Operation::SLoad => GasCost {
                base: 800,
                dynamic_multiplier: 0,
            },
            Operation::SStore => GasCost {
                base: 5000,
                dynamic_multiplier: 15000,
            }, // Can be 20k for new value

            // Memory operations have dynamic costs based on size
            Operation::MLoad | Operation::MStore => GasCost {
                base: 3,
                dynamic_multiplier: 3,
            },

            // Push operations
            Operation::Push1(_)
            | Operation::Push2(_)
            | Operation::Push3(_)
            | Operation::Push4(_)
            | Operation::Push5(_)
            | Operation::Push6(_)
            | Operation::Push7(_)
            | Operation::Push8(_)
            | Operation::Push9(_)
            | Operation::Push10(_)
            | Operation::Push11(_)
            | Operation::Push12(_)
            | Operation::Push13(_)
            | Operation::Push14(_)
            | Operation::Push15(_)
            | Operation::Push16(_)
            | Operation::Push17(_)
            | Operation::Push18(_)
            | Operation::Push19(_)
            | Operation::Push20(_)
            | Operation::Push21(_)
            | Operation::Push22(_)
            | Operation::Push23(_)
            | Operation::Push24(_)
            | Operation::Push25(_)
            | Operation::Push26(_)
            | Operation::Push27(_)
            | Operation::Push28(_)
            | Operation::Push29(_)
            | Operation::Push30(_)
            | Operation::Push31(_)
            | Operation::Push32(_) => GasCost {
                base: 3,
                dynamic_multiplier: 0,
            },

            // System operations
            Operation::Create => GasCost {
                base: 32000,
                dynamic_multiplier: 200,
            },
            Operation::Call => GasCost {
                base: 700,
                dynamic_multiplier: 9000,
            },
            Operation::SelfDestruct => GasCost {
                base: 5000,
                dynamic_multiplier: 25000,
            },

            // Default for others
            _ => GasCost {
                base: 3,
                dynamic_multiplier: 0,
            },
        }
    }

    // Get stack requirements for this operation
    pub fn stack_req(&self) -> StackReq {
        match self {
            Operation::Push1(_)
            | Operation::Push2(_)
            | Operation::Push3(_)
            | Operation::Push4(_)
            | Operation::Push5(_)
            | Operation::Push6(_)
            | Operation::Push7(_)
            | Operation::Push8(_)
            | Operation::Push9(_)
            | Operation::Push10(_)
            | Operation::Push11(_)
            | Operation::Push12(_)
            | Operation::Push13(_)
            | Operation::Push14(_)
            | Operation::Push15(_)
            | Operation::Push16(_)
            | Operation::Push17(_)
            | Operation::Push18(_)
            | Operation::Push19(_)
            | Operation::Push20(_)
            | Operation::Push21(_)
            | Operation::Push22(_)
            | Operation::Push23(_)
            | Operation::Push24(_)
            | Operation::Push25(_)
            | Operation::Push26(_)
            | Operation::Push27(_)
            | Operation::Push28(_)
            | Operation::Push29(_)
            | Operation::Push30(_)
            | Operation::Push31(_)
            | Operation::Push32(_) => StackReq {
                min_stack_height: 0,
                stack_inputs: 0,
                stack_outputs: 1,
            },

            Operation::Add | Operation::Sub | Operation::Mul | Operation::Div => StackReq {
                min_stack_height: 2,
                stack_inputs: 2,
                stack_outputs: 1,
            },

            Operation::Pop => StackReq {
                min_stack_height: 1,
                stack_inputs: 1,
                stack_outputs: 0,
            },

            Operation::Dup(n) if *n >= 1 && *n <= 16 => StackReq {
                min_stack_height: *n as u32,
                stack_inputs: 0,
                stack_outputs: 1,
            },

            // Default conservative requirements
            _ => StackReq {
                min_stack_height: 1,
                stack_inputs: 1,
                stack_outputs: 1,
            },
        }
    }

    // Get opcode (byte representation)
    pub fn opcode(&self) -> u8 {
        match self {
            Operation::Stop => 0x00,
            Operation::Add => 0x01,
            Operation::Mul => 0x02,
            Operation::Sub => 0x03,
            Operation::Div => 0x04,
            Operation::SStore => 0x55,
            Operation::SLoad => 0x54,
            // ... Add more as needed
            _ => 0x00, // Default, should implement for all opcodes
        }
    }
}
