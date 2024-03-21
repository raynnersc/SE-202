use std::io::{self, Write};

pub const MEMORY_SIZE: usize = 4096;
const NREGS: usize = 16;

const IP: usize = 0;

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Machine {
    mem: [u8; MEMORY_SIZE],
    regs: [u32; NREGS],
}

#[derive(Debug)]
pub enum Error {
    /// Attempt to create a machine with too large a memory
    MemoryOverflow,
    InvalidRegister,
    InvalidInstruction,
    InvalidMemoryAccess,
    OutputError,
    TESTE,
    // Add some more entries to represent different errors
}

impl Machine {
    /// Create a new machine in its reset state. The `memory` parameter will
    /// be copied at the beginning of the machine memory.
    ///
    /// # Errors
    /// This function returns an error when the memory exceeds `MEMORY_SIZE`.
    pub fn new(memory: &[u8]) -> Result<Self> {
        if memory.len() > MEMORY_SIZE {
            Err(Error::MemoryOverflow)
        } else {
            let mut machine = Self {
                mem: [0; MEMORY_SIZE],
                regs: [0; NREGS],
            };
            machine.mem[..memory.len()].copy_from_slice(memory);
            log::info!("Machine created with memory: {:?}", machine.mem);
            Ok(machine)
        }
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on `fd`.
    pub fn run_on<T: Write>(&mut self, fd: &mut T) -> Result<()> {
        while !self.step_on(fd)? {}
        Ok(())
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on standard output.
    pub fn run(&mut self) -> Result<()> {
        self.run_on(&mut io::stdout().lock())
    }

    /// Execute the next instruction by doing the following steps:
    ///   - decode the instruction located at IP (register 0)
    ///   - increment the IP by the size of the instruction
    ///   - execute the decoded instruction
    ///
    /// If output instructions are run, they print on `fd`.
    /// If an error happens at either of those steps, an error is
    /// returned.
    ///
    /// In case of success, `true` is returned if the program is
    /// terminated (upon encountering an exit instruction), or
    /// `false` if the execution must continue.
    pub fn step_on<T: Write>(&mut self, fd: &mut T) -> Result<bool> {
        let address = self.regs[IP] as usize;
        if address >= MEMORY_SIZE {
            return Err(Error::InvalidMemoryAccess);
        }
        let instruction = self.mem[address];
        let offset = match instruction {
            1 | 4 | 5 => 4,
            2 | 3 => 3,
            6 | 8 => 2,
            7 => 1,
            _ => return Err(Error::InvalidInstruction),
        } as u32;
        self.regs[IP] += offset;

        log::info!(
            "Executing instruction: {:?} at address: {}",
            instruction,
            address
        );

        match instruction {
            1 => {
                if address + 3 >= MEMORY_SIZE {
                    return Err(Error::InvalidMemoryAccess);
                }
                let index_regs: [u8; 3] = self.mem[address + 1..=address + 3].try_into().unwrap();
                let (ri, rj, rk) = (
                    index_regs[0] as usize,
                    index_regs[1] as usize,
                    index_regs[2] as usize,
                );
                if ri >= NREGS || rj >= NREGS || rk >= NREGS {
                    return Err(Error::InvalidRegister);
                }
                if self.regs[rk] != 0 {
                    self.regs[ri] = self.regs[rj];
                }
            }
            2 => {
                if address + 2 >= MEMORY_SIZE {
                    return Err(Error::InvalidMemoryAccess);
                }
                let index_regs: [u8; 2] = self.mem[address + 1..=address + 2].try_into().unwrap();
                let (ri, rj) = (index_regs[0] as usize, index_regs[1] as usize);
                if ri >= NREGS || rj >= NREGS {
                    return Err(Error::InvalidRegister);
                }
                let dest_addr = self.regs[ri] as usize;
                if dest_addr + 3 >= MEMORY_SIZE {
                    return Err(Error::InvalidMemoryAccess);
                }
                self.mem[dest_addr..=dest_addr + 3].copy_from_slice(&self.regs[rj].to_le_bytes());
            }
            3 => {
                if address + 2 >= MEMORY_SIZE {
                    return Err(Error::InvalidMemoryAccess);
                }
                let index_regs: [u8; 2] = self.mem[address + 1..=address + 2].try_into().unwrap();
                let (ri, rj) = (index_regs[0] as usize, index_regs[1] as usize);
                if ri >= NREGS || rj >= NREGS {
                    return Err(Error::InvalidRegister);
                }
                let src_addr = self.regs[rj] as usize;
                if src_addr + 3 >= MEMORY_SIZE {
                    return Err(Error::InvalidMemoryAccess);
                }
                self.regs[ri] =
                    u32::from_le_bytes(self.mem[src_addr..=src_addr + 3].try_into().unwrap());
            }
            4 => {
                if address + 3 >= MEMORY_SIZE {
                    return Err(Error::InvalidMemoryAccess);
                }
                let index_regs: [u8; 3] = self.mem[address + 1..=address + 3].try_into().unwrap();
                let (ri, imm) = (
                    index_regs[0] as usize,
                    i16::from_le_bytes(index_regs[1..].try_into().unwrap()),
                );
                if ri >= NREGS {
                    return Err(Error::InvalidRegister);
                }
                self.regs[ri] = imm as u32;
            }
            5 => {
                if address + 3 >= MEMORY_SIZE {
                    return Err(Error::InvalidMemoryAccess);
                }
                let index_regs: [u8; 3] = self.mem[address + 1..=address + 3].try_into().unwrap();
                let (ri, rj, rk) = (
                    index_regs[0] as usize,
                    index_regs[1] as usize,
                    index_regs[2] as usize,
                );
                if ri >= NREGS || rj >= NREGS || rk >= NREGS {
                    return Err(Error::InvalidRegister);
                }
                self.regs[ri] = self.regs[rj].wrapping_sub(self.regs[rk]);
            }
            6 => {
                if address + 1 >= MEMORY_SIZE {
                    return Err(Error::InvalidMemoryAccess);
                }
                let ri: usize = self.mem[address + 1].into();
                if ri >= NREGS {
                    return Err(Error::InvalidRegister);
                }
                let data = self.regs[ri] as u8 as char;
                write!(fd, "{}", data).map_err(|_| Error::OutputError)?;
            }
            7 => {
                return Ok(true);
            }
            8 => {
                if address + 1 >= MEMORY_SIZE {
                    return Err(Error::InvalidMemoryAccess);
                }
                let ri: usize = self.mem[address + 1].into();
                if ri >= NREGS {
                    return Err(Error::InvalidRegister);
                }
                let data = self.regs[ri] as i32;
                write!(fd, "{}", data).map_err(|_| Error::OutputError)?;
            }
            _ => {
                return Err(Error::InvalidInstruction);
            }
        }
        Ok(false)
    }

    /// Similar to [`step_on`](Machine::step_on).
    /// If output instructions are run, they print on standard output.
    pub fn step(&mut self) -> Result<bool> {
        self.step_on(&mut io::stdout().lock())
    }

    /// Reference onto the machine current set of registers.
    #[must_use]
    pub fn regs(&self) -> &[u32] {
        &self.regs
    }

    /// Sets a register to the given value.
    pub fn set_reg(&mut self, reg: usize, value: u32) -> Result<()> {
        if reg < 16 {
            self.regs[reg] = value;
            Ok(())
        } else {
            Err(Error::InvalidRegister)
        }
    }

    /// Reference onto the machine current memory.
    #[must_use]
    pub fn memory(&self) -> &[u8] {
        &self.mem
    }
}
