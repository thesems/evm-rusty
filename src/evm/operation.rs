use alloy_primitives::U256;
use strum_macros::FromRepr;

#[derive(Debug)]
pub enum OperationError {
    InvalidOpcodeFormat,
    InvalidPush,
    PushOpNeedsData,
    UnknownOpcode,
}

// Gas cost calculation helper structs
pub struct GasCost {
    pub base: u64,               // Base cost of operation
    pub dynamic_multiplier: u64, // For operations with dynamic costs
}

// Stack requirements for operation
pub struct StackReq {
    pub min_stack_height: u32, // Required minimum stack height
    pub stack_inputs: u32,     // Number of items taken from stack
    pub stack_outputs: u32,    // Number of items pushed to stack
}

#[repr(u8)]
#[derive(FromRepr, Debug, Clone)]
pub enum Operation {
    // 0x00 - 0x0f: Stop and Arithmetic
    Stop = 0x0,
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
    Lt = 0x10,
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

    // 0x20
    Keccak256 = 0x20,

    // 0x30 - 0x3f: Environmental Information
    Address = 0x30,
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
    BlockHash = 0x40,
    Coinbase,
    Timestamp,
    Number,
    Difficulty,
    GasLimit,
    ChainId,
    SelfBalance,
    BaseFee,

    // 0x50 - 0x5f: Stack, Memory, Storage and Flow
    Pop = 0x50,
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
    Push0 = 0x5F,
    Push1(U256) = 0x60,
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
    Dup(u8) = 0x80,

    // 0x90 - 0x9f: Exchange Operations
    Swap1 = 0x90,
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
    Log0 = 0xa0,
    Log1,
    Log2,
    Log3,
    Log4,

    // 0xf0 - 0xff: System Operations
    Create = 0xf0,
    Call,
    CallCode,
    Return,
    DelegateCall,
    Create2,
    StaticCall = 0xfa,
    Revert = 0xfd,
    Invalid = 0xfe,
    SelfDestruct = 0xff,
}

impl Operation {
    // Convert a byte to Operation
    pub fn from_byte(byte: u8, data: Option<U256>) -> Result<Self, OperationError> {
        match byte {
            // 0x00 - 0x0f: Stop and Arithmetic
            // 0x10 - 0x1f: Comparison & Bitwise Logic
            // 0x30 - 0x3f: Environmental Information
            // 0x40 - 0x4f: Block Information
            // 0x50 - 0x5f: Stack, Memory, Storage and Flow
            // Swap operations (0x90 - 0x9f)
            // 0xa0 - 0xa4: Logging
            // 0xf0 - 0xff: System
            0x00..0x5f | 0x90..=0x9f | 0xa0..=0xff => {
                Ok(Operation::from_repr(byte).ok_or(OperationError::InvalidOpcodeFormat)?)
            }

            0x5f => Ok(Operation::Push0),

            // Push operations (0x60 - 0x7f)
            0x60..=0x7f => {
                // Check if we have data for push operations
                match data {
                    Some(value) => Ok(match byte - 0x60 {
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
                        _ => return Err(OperationError::InvalidPush),
                    }),
                    None => Err(OperationError::PushOpNeedsData),
                }
            }

            // Dup operations (0x80 - 0x8f)
            0x80..=0x8f => Ok(Operation::Dup((byte - 0x80 + 1) as u8)),

            _ => Err(OperationError::UnknownOpcode),
        }
    }

    // Get the number of additional bytes needed for this operation
    pub fn additional_bytes(&self) -> usize {
        match self {
            Operation::Push1(_) => 1,
            Operation::Push2(_) => 2,
            Operation::Push3(_) => 3,
            Operation::Push4(_) => 4,
            Operation::Push5(_) => 5,
            Operation::Push6(_) => 6,
            Operation::Push7(_) => 7,
            Operation::Push8(_) => 8,
            Operation::Push9(_) => 9,
            Operation::Push10(_) => 10,
            Operation::Push11(_) => 11,
            Operation::Push12(_) => 12,
            Operation::Push13(_) => 13,
            Operation::Push14(_) => 14,
            Operation::Push15(_) => 15,
            Operation::Push16(_) => 16,
            Operation::Push17(_) => 17,
            Operation::Push18(_) => 18,
            Operation::Push19(_) => 19,
            Operation::Push20(_) => 20,
            Operation::Push21(_) => 21,
            Operation::Push22(_) => 22,
            Operation::Push23(_) => 23,
            Operation::Push24(_) => 24,
            Operation::Push25(_) => 25,
            Operation::Push26(_) => 26,
            Operation::Push27(_) => 27,
            Operation::Push28(_) => 28,
            Operation::Push29(_) => 29,
            Operation::Push30(_) => 30,
            Operation::Push31(_) => 31,
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
            Operation::JumpDest => StackReq {
                min_stack_height: 0,
                stack_inputs: 0,
                stack_outputs: 0,
            },

            Operation::Push0
            | Operation::Push1(_)
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
            | Operation::Push32(_)
            | Operation::CallDataSize
            | Operation::CodeSize
            | Operation::CallValue
            | Operation::Origin
            | Operation::Address => StackReq {
                min_stack_height: 0,
                stack_inputs: 0,
                stack_outputs: 1,
            },

            Operation::CallDataLoad => StackReq {
                min_stack_height: 0,
                stack_inputs: 1,
                stack_outputs: 1,
            },


            Operation::Pop | Operation::Jump => StackReq {
                min_stack_height: 1,
                stack_inputs: 1,
                stack_outputs: 0,
            },

            Operation::IsZero | Operation::SLoad => StackReq {
                min_stack_height: 1,
                stack_inputs: 1,
                stack_outputs: 1,
            },

            Operation::MStore
            | Operation::MStore8
            | Operation::SStore
            | Operation::JumpI
            | Operation::Revert => StackReq {
                min_stack_height: 2,
                stack_inputs: 2,
                stack_outputs: 0,
            },

            Operation::Add | Operation::Sub | Operation::Mul | Operation::Div => StackReq {
                min_stack_height: 2,
                stack_inputs: 2,
                stack_outputs: 1,
            },

            Operation::CodeCopy => StackReq {
                min_stack_height: 3,
                stack_inputs: 3,
                stack_outputs: 0,
            },

            Operation::Dup(n) if *n >= 1 && *n <= 16 => StackReq {
                min_stack_height: *n as u32,
                stack_inputs: 0,
                stack_outputs: 1,
            },

            // Default conservative requirements
            _ => panic!("Stack requirements not implemented for {:?}", self),
        }
    }

    pub fn opcode(&self) -> u8 {
        // SAFETY: This is safe because:
        // 1. The enum is #[repr(u8)]
        // 2. We're only reading the discriminant, not the associated data
        // 3. The size of u8 matches the repr
        unsafe { *(self as *const Operation as *const u8) }
    }
}
