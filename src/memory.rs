use std::fmt;

pub const MEMORY_SIZE: usize = 0x1000;

//Hold the memory
pub struct Memory {
    data: [u8; MEMORY_SIZE]
}

//Manage memory
impl Memory{
    //Return a new memory with all null data
    pub fn new() -> Memory {
        Memory {
            data: [0; MEMORY_SIZE]
        }
    }
    //Translate a virtual adress to local
    //Basically just remove 0x8000_0000
    pub fn virt_to_local(virt: usize) -> usize{
        virt & 0xFF_FFFF
    }

    pub fn read(&self, at: usize, buf: &mut [u8]) {
        let at = Memory::virt_to_local(at);
        buf.copy_from_slice(&self.data[at..at + buf.len()]);
    }

    pub fn write(&mut self, at: usize, buf: &[u8]){
        let at = Memory::virt_to_local(at);
        self.data[at..at + buf.len()].copy_from_slice(buf);
    }

    pub fn allocate(&mut self, data: &[u8]){
        self.data[0..data.len()].copy_from_slice(data);
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
        writeln!(f)*/
        writeln!(f)
    }
}