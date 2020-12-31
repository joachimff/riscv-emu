use std::fmt;

pub const STACK_SIZE: usize = 0x10;

#[derive(Debug)]
struct MemoryRegion{
    data: Vec<u8>,
    virt_addr: u64,
    size: u64,
}

//Hold the memory
pub struct Memory {
    //data: [u8; STACK_SIZE],
    allocated: Vec<MemoryRegion>,
}

//Manage memory
impl Memory{
    //Return a new memory with all null data
    pub fn new() -> Memory {
        Memory {
            //data: [0; STACK_SIZE],
            allocated: Vec::new()
        }
    }
    
    pub fn read(&self, at: u64, buf: &mut [u8]) {
        for m in &self.allocated{
            if at >= m.virt_addr && at <= (m.virt_addr + m.size){
                buf.copy_from_slice(&m.data[(at - m.virt_addr) as usize..(at - m.virt_addr) as usize + buf.len()]);
                return
            }
        }
    }

    pub fn write(&mut self, at: u64, buf: &[u8]){
        for m in &mut self.allocated{
            if at >= m.virt_addr && at <= (m.virt_addr + m.size){
                m.data[(at - m.virt_addr) as usize..(at - m.virt_addr) as usize + buf.len()].copy_from_slice(buf);
                return;
            }
        }
        panic!("Try to get memory unmapped at: {:#8X}", at);
    }

    pub fn allocate(&mut self, at: u64, size: u64, data: &[u8]){
        self.allocated.push(
            MemoryRegion{
                data: data.to_vec(),
                virt_addr: at,
                size: size
            }
        )
    }
}

impl fmt::Debug for Memory{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /*
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
        */
        writeln!(f)
    }
}