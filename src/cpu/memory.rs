use std::fmt;

pub const STACK_SIZE: usize = 0x1000;

// No idea of what would be a good value 
pub const BITMAP_SIZE: u64 = 0x10;

#[derive(Debug, Clone)]
struct MemoryRegion{
    data: Vec<u8>,
    virt_addr: u64,
    size: u64,

    dirty_bitmap: Vec<u8>,
}

//Hold the memory
#[derive(Clone)]
pub struct Memory {
    stack: [u8; STACK_SIZE],
    allocated: Vec<MemoryRegion>,

    saved_state: Option<Vec<MemoryRegion>>,
}

//Manage memory
impl Memory{
    //Return a new memory with all null data
    pub fn new() -> Memory {
        Memory {
            stack: [0; STACK_SIZE],
            allocated: Vec::new(),
            saved_state: None,
        }
    }
    
    pub fn read(&self, at: u64, buf: &mut [u8]) {
        if at < STACK_SIZE as u64{
            buf.copy_from_slice(&self.stack[at as usize .. at as usize + buf.len()]);
            return
        }

        for m in &self.allocated{
            if at >= m.virt_addr && at < (m.virt_addr + m.size){
                buf.copy_from_slice(&m.data[(at - m.virt_addr) as usize..(at - m.virt_addr) as usize + buf.len()]);
                return
            }
        }
    }

    pub fn write(&mut self, at: u64, buf: &[u8]){
        if at < STACK_SIZE as u64{
            self.stack[at as usize.. at as usize + buf.len()].copy_from_slice(buf);
            return
        }

        for m in &mut self.allocated{
            if at >= m.virt_addr && at < (m.virt_addr + m.size){
                /*println!("at:{:08X} virt_addr:{:08X} end_section{:08X}, section_buff_size:{:08X} buf len {:08X}",
                    at,
                    m.virt_addr,
                    (at - m.virt_addr) as usize + buf.len(),
                    m.data.len(),
                    buf.len()
                );*/
                let relative_addr = (at - m.virt_addr) as usize;
                m.data[relative_addr..relative_addr + buf.len()].copy_from_slice(buf);
                
                //Set first byte to 1 when data has been changed
                m.dirty_bitmap[relative_addr / BITMAP_SIZE as usize] = 0x1;

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
                size: size,

                dirty_bitmap: vec![0; (size  / BITMAP_SIZE) as usize],
            }
        )
    }
    
    pub fn save_state(&mut self){
        // Clone the current memory state
        self.saved_state = Some(self.allocated.clone());

        //Reset the dirty bytes bitmap
        for m in &mut self.allocated{
            m.dirty_bitmap = vec![0; (m.size / BITMAP_SIZE) as usize];
        }
    }

    pub fn reset_to_saved_state(&mut self){
        let saved_state = self.saved_state.as_mut()
            .expect("Trying to reset but no initial state has been saved");

        let mut i: usize = 0;
        let mut nb_chunks = 0;
        let mut nb_chunks_reseted = 0;

        for m in &mut self.allocated{
            let mut j: usize = 0;

            for dirty in &m.dirty_bitmap{
                if *dirty == 1{
                    let begin_block = j * BITMAP_SIZE as usize;
                    let end_block = ((j + 1) * BITMAP_SIZE as usize) - 1;

                    m.data[begin_block..end_block].copy_from_slice(&saved_state[i].data[begin_block..end_block]);
                    nb_chunks_reseted = nb_chunks_reseted + 1;
                }

                nb_chunks = nb_chunks + 1;
                j += 1;
            }
            i += 1;
        }

        println!("Reseted {} memory chunks (size {} bytes), total: {}", nb_chunks_reseted, BITMAP_SIZE, nb_chunks);
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