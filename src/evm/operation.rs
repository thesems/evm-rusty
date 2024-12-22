use alloy_primitives::{Address, B256, U256};

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
