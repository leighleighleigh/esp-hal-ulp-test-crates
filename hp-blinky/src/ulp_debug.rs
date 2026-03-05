// Non-public debugging interfaces for the RISCV ULP,
// figured out by inspection of PicoRV32 source code,
// and reading forums - Leigh Oliver
// Originally written for stompy-ulp/hp-core project,
// on the 2nd of January 2026.
use log::info;
use esp_hal::peripherals;

#[derive(Debug)]
#[allow(unused)]
pub struct SarCocpuState {
    clk_en_st: bool,
    reset_n: bool,
    eoi: bool,
    trap: bool,
    ebreak: bool
}

#[derive(Debug)]
#[allow(unused)]
pub struct CocpuDebug {
    pc: u16,
    mem_valid: bool,
    mem_ready: bool,
    write_enable: u8,
    mem_address: u16,
    state : SarCocpuState
}

fn read_coproc_state() -> SarCocpuState {
    // 1. Trigger the debug prompt
    let r = unsafe { &*peripherals::SENS::PTR }.sar_cocpu_state().read();
    SarCocpuState {
        clk_en_st: r.sar_cocpu_clk_en_st().bit_is_set(),
        reset_n: r.sar_cocpu_reset_n().bit_is_set(),
        eoi: r.sar_cocpu_eoi().bit_is_set(),
        trap: r.sar_cocpu_trap().bit_is_set(),
        ebreak: r.sar_cocpu_ebreak().bit_is_set(),
    }
}

pub fn read_coproc_debug() -> CocpuDebug {
    // Forum quote
    // "SENS_SAR_COCPU_DEBUG_REG might be helpful. Set SENS_SAR_COCPU_STATE_REG[SENS_COCPU_DBG_TRIGGER] to update (I'm not sure it's immediate though?)."

    // 1. Trigger the debug prompt
    unsafe {
        { &*peripherals::SENS::PTR }
            .sar_cocpu_state()
            .write(|w| w.sar_cocpu_dbg_trigger().set_bit());
    };

    // 2. Read the debug register
    let r = unsafe { &*peripherals::SENS::PTR }.sar_cocpu_debug().read();
    CocpuDebug {
        pc: r.sar_cocpu_pc().bits(),
        mem_valid: r.sar_cocpu_mem_vld().bit_is_set(),
        mem_ready: r.sar_cocpu_mem_rdy().bit_is_set(),
        write_enable: r.sar_cocpu_mem_wen().bits(),
        mem_address: r.sar_cocpu_mem_addr().bits(),
        state: read_coproc_state()
    }
}

pub fn dump_coproc_pc_instructions(dbg: CocpuDebug) {
    // Using the 'pc' field of CocpuDebug,
    // reads the data from RTC_SLOW_MEM and prints it as hex.
    // Will print an instruction before and after this too.
    let pc = (dbg.pc as u32 + 0x50000000) as *mut u32;
    let instr = unsafe { pc.read_unaligned() };
    info!("*PC({:04x}): {:08x}", dbg.pc, instr);
}