use std::fmt;

pub const STACK_SIZE: usize = 0x10;

#[derive(Debug)]
struct MemoryRegion{
    data: Vec<u8>,
    virt_addr: u32,
    size: usize,
}

//Hold the memory
#[derive(Debug)]
pub struct Memory {
    data: [u8; STACK_SIZE],
    allocated: Vec<MemoryRegion>,
}

//Manage memory
impl Memory{
    //Return a new memory with all null data
    pub fn new() -> Memory {
        Memory {
            data: [0; STACK_SIZE],
            allocated: Vec::new()
        }
    }
    //Translate a virtual adress to local
    //Basically just remove 0x8000_0000
    pub fn virt_to_local(virt: usize) -> usize{
        virt & 0xFF_FFFF
    }
    
    pub fn read(&self, at: u32, buf: &mut [u8]) {
        for m in &self.allocated{
            if at >= m.virt_addr && at <= (m.virt_addr + m.size as u32){
                buf.copy_from_slice(&m.data[(at - m.virt_addr) as usize..(at - m.virt_addr) as usize + buf.len()]);
            }
        }

    }

    pub fn write(&mut self, at: u32, buf: &[u8]){
        for m in &mut self.allocated{
            if at >= m.virt_addr && at <= (m.virt_addr + m.size as u32){
                m.data[(at - m.virt_addr) as usize..(at - m.virt_addr) as usize + buf.len()].copy_from_slice(buf);
            }
        }
        panic!("Try to get memory unmapped at: {:#8X}", at);
    }

    pub fn allocate(&mut self, at: u32, size: usize, data: &[u8]){
        self.allocated.push(
            MemoryRegion{
                data: data.to_vec(),
                virt_addr: at,
                size: size
            }
        )
    }
}

/*
impl fmt::Debug for Memory{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "---------------------------------------------------Memory--------------------------------------------------");
        
        let mut i = 0;
        while i < self.data.len(){
            if (i % 0x10) == 0{
                if i != 0 {writeln!(f);}
                write!(f, "{:#08X}: ", i);
            }
            if (i % 2) == 0{
                write!(f, " ");
            }
            write!(f, "{:02X}", self.data[i]);
            i += 1;
        }
        writeln!(f)
        writeln!(f)
    }
}
*/