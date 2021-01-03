#[derive(Debug)]
pub struct RType{
    pub funct7: u8,
    pub rs2: usize,
    pub rs1: usize,
    pub funct3: u8,
    pub rd: usize,
    pub opcode: u8
}

impl From<u32> for RType{
    fn from(instruction:u32) -> Self{
        RType{
            funct7: (instruction >> 25) as u8,
            rs2:    ((instruction >> 20) & 0b11111) as usize,
            rs1:    ((instruction >> 15) & 0b11111) as usize,
            funct3: ((instruction >> 12) & 0b111) as u8,
            rd:     ((instruction >> 7) & 0b11111) as usize,
            opcode: (instruction & 0b111_1111) as u8
        }
    }
}

#[derive(Debug)]
pub struct IType{
    pub imm: i32,
    pub rs1: usize,
    pub funct3: u8,
    pub rd: usize,
    pub opcode: u8
}

impl From<u32> for IType {
    fn from(instruction: u32) -> Self {
        IType{
            imm:    (instruction as i32) >> 20,
            rs1:    ((instruction >> 15)  & 0b11111) as usize,
            funct3: ((instruction >> 12)  & 0b111) as u8,
            rd:     ((instruction >> 7)   & 0b11111) as usize,
            opcode: (instruction          & 0b1111_111) as u8,
        }
    }
}

#[derive(Debug)]
pub struct UType{
    pub imm: u32,
    pub rd: usize,
    pub opcode: u8
}

impl From<u32> for UType{
    fn from(instruction:u32) -> Self{
        UType{
            imm: ((instruction >> 12) & 0b1111_1111_1111_1111_1111) as u32,
            rd:     ((instruction >> 7) & 0b11111) as usize,
            opcode: (instruction & 0b111_1111) as u8
        }
    }
}

#[derive(Debug)]
pub struct JType{
    pub imm: i32,
    pub rd: usize,
    pub opcode: u8
}

impl From<u32> for JType{
    fn from(instruction:u32) -> Self{
        JType{
            imm: (((instruction >> 21) & 0b11_1111_1111) << 1 |
                ((instruction >> 20) & 0b1) << 11 |
                ((instruction >> 12) & 0b1111_1111) << 12) as i32 |
                //Sign extended
                (((instruction as i32) >> 31) << 20) as i32,
            rd:     ((instruction >> 7) & 0b1_1111) as usize,
            opcode: (instruction & 0b111_1111) as u8
        }
    }
}

#[derive(Debug)]
pub struct BType{
    pub imm: i32,
    pub rs1: usize,
    pub rs2: usize,
    pub func3: u8,
    pub opcode: u8
}

impl From<u32> for BType{
    fn from(instruction:u32) -> Self{
        BType{
            imm: (((instruction >> 8) & 0b1111) << 1 |
                ((instruction >> 25) & 0b11_1111) << 5 |
                ((instruction >> 7) & 0b1) << 11) as i32 |
                //Sign extended
                (((instruction as i32) >> 31) << 12),
            rs1: ((instruction >> 15) & 0b11111) as usize,
            rs2: ((instruction >> 20) & 0b11111) as usize,
            func3: ((instruction >> 12) & 0b111) as u8,
            opcode: (instruction  & 0b111_1111) as u8,
        }
    }
}

#[derive(Debug)]
pub struct SType{
    pub imm: i32,
    pub rs1: usize,
    pub rs2: usize,
    pub funct3: u8,
    pub opcode: u8
}

impl From<u32> for SType{
    fn from(instruction:u32) -> Self{
        SType{
            imm: ((instruction >> 7) & 0b1_1111) as i32|
                //Sign extended
                (((instruction as i32) >> 25) << 5),
            rs1: ((instruction >> 15) & 0b11111) as usize,
            rs2: ((instruction >> 20) & 0b11111) as usize,
            funct3: ((instruction >> 12) & 0b111) as u8,
            opcode: (instruction  & 0b111_1111) as u8,
        }
    }
}