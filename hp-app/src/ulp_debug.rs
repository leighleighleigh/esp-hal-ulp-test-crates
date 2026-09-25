// Non-public debugging interfaces for the RISCV ULP,
// figured out by inspection of PicoRV32 source code,
// and reading forums - Leigh Oliver
// Originally written for stompy-ulp/hp-core project,
// on the 2nd of January 2026.
#![allow(dead_code)]
use core::fmt::Display;

use esp_hal::peripherals;
use riscv_decode::{self, DecodingError, Instruction};

const COPROC_ADDRESS_BASE: usize = 0x5000_0000;

pub trait FromRegister {
    fn read() -> Self;
}

#[derive(defmt::Format, Debug, Clone, Copy, PartialEq)]
#[allow(unused)]
pub struct SarCocpuState {
    clk_en_st: bool,
    reset_n: bool,
    eoi: bool,
    trap: bool,
    ebreak: bool,
}

impl FromRegister for SarCocpuState {
    #[cfg(esp32s3)]
    fn read() -> Self {
        let r = unsafe { &*peripherals::SENS::PTR }.sar_cocpu_state().read();

        SarCocpuState {
            clk_en_st: r.sar_cocpu_clk_en_st().bit_is_set(),
            reset_n: r.sar_cocpu_reset_n().bit_is_set(),
            eoi: r.sar_cocpu_eoi().bit_is_set(),
            trap: r.sar_cocpu_trap().bit_is_set(),
            ebreak: r.sar_cocpu_ebreak().bit_is_set(),
        }
    }
    #[cfg(esp32s2)]
    fn read() -> Self {
        let r = unsafe { &*peripherals::SENS::PTR }.sar_cocpu_state().read();
        SarCocpuState {
            clk_en_st: r.cocpu_clk_en().bit_is_set(),
            reset_n: r.cocpu_reset_n().bit_is_set(),
            eoi: r.cocpu_eoi().bit_is_set(),
            trap: r.cocpu_trap().bit_is_set(),
            ebreak: r.cocpu_ebreak().bit_is_set(),
        }
    }
}

#[derive(defmt::Format, Debug, Clone, Copy, PartialEq)]
#[allow(unused)]
pub struct CocpuDebug {
    pc: u16,
    mem_valid: bool,
    mem_ready: bool,
    write_enable: u8,
    mem_address: u16,
    state: SarCocpuState,
}

impl CocpuDebug {
    fn trigger_debug() {
        #[cfg(esp32s3)]
        unsafe {
            { &*peripherals::SENS::PTR }
                .sar_cocpu_state()
                .write(|w| w.sar_cocpu_dbg_trigger().set_bit())
        };
        #[cfg(esp32s2)]
        unsafe {
            { &*peripherals::SENS::PTR }
                .sar_cocpu_state()
                .write(|w| w.cocpu_dbg_trigger().set_bit())
        };
    }
    fn read_debug() -> Self {
        let r = unsafe { &*peripherals::SENS::PTR }.sar_cocpu_debug().read();

        #[cfg(esp32s3)]
        return CocpuDebug {
            pc: r.sar_cocpu_pc().bits(),
            mem_valid: r.sar_cocpu_mem_vld().bit_is_set(),
            mem_ready: r.sar_cocpu_mem_rdy().bit_is_set(),
            write_enable: r.sar_cocpu_mem_wen().bits(),
            mem_address: r.sar_cocpu_mem_addr().bits(),
            state: SarCocpuState::read(),
        };

        #[cfg(esp32s2)]
        return CocpuDebug {
            pc: r.cocpu_pc().bits(),
            mem_valid: r.cocpu_mem_vld().bit_is_set(),
            mem_ready: r.cocpu_mem_rdy().bit_is_set(),
            write_enable: r.cocpu_mem_wen().bits(),
            mem_address: r.cocpu_mem_addr().bits(),
            state: SarCocpuState::read(),
        };
    }

    /// Returns the program counter in the HP-core memory space
    pub fn pc_address(&self) -> u32 {
        self.pc as u32 + COPROC_ADDRESS_BASE as u32
    }

    /// Get the instruction code pointed at by the program counter
    pub fn pc_instruction(&self) -> u32 {
        let pc = self.pc_address() as *mut u32;
        unsafe { pc.read_unaligned() }
    }

    /// Decodes the RISCV instruction at the PC
    pub fn decode_instruction(&self) -> Result<Instruction, DecodingError> {
        let instr = self.pc_instruction();
        riscv_decode::decode(instr)
    }
}

impl FromRegister for CocpuDebug {
    fn read() -> Self {
        // Forum quote:
        // "SENS_SAR_COCPU_DEBUG_REG might be helpful. Set
        // SENS_SAR_COCPU_STATE_REG[SENS_COCPU_DBG_TRIGGER] to update (I'm not sure it's immediate
        // though?)."
        // 1. Trigger the debug prompt
        CocpuDebug::trigger_debug();
        // 2. Read the debug register
        CocpuDebug::read_debug()
    }
}

impl Display for CocpuDebug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CocpuDebug")
            .field("pc", &format_args!("0x{:04X}", &self.pc))
            .field("*pc", &format_args!("0x{:08X}", &self.pc_instruction()))
            .field("instr", &self.decode_instruction())
            .field("state", &self.state)
            .finish()
    }
}

// pub fn get_cocpu_pc_instr(dbg: &CocpuDebug) -> (u32, u32) {
//     // Using the 'pc' field of CocpuDebug,
//     // calculates the HP-core-relative PC address,
//     // reads the data from RTC_SLOW_MEM.
//     let pc = (dbg.pc as u32 + COPROC_ADDRESS_BASE as u32) as *mut u32;
//     let instr = unsafe { pc.read_unaligned() };
//     (dbg.pc as u32, instr)
// }

// pub fn dump_coproc_pc_instructions(dbg: &CocpuDebug) {
//     let (pc, instr) = get_cocpu_pc_instr(dbg);
//     info!("*PC(0x{pc:x}): {instr:08x}");
// }
