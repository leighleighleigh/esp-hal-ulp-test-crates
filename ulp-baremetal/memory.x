/* CONFIG_ULP_COPROC_RESERVE_MEM = 8 * 1024; */

MEMORY
{
    /* RAM(RW) : ORIGIN = 0, LENGTH = CONFIG_ULP_COPROC_RESERVE_MEM */
    RAM : ORIGIN = 0x0, LENGTH = 8K 
}

REGION_ALIAS("REGION_TEXT", RAM);
REGION_ALIAS("REGION_RODATA", RAM);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);

_stext = ORIGIN(REGION_TEXT) + 0x0;             /* Load .text region at 0x0 */
_heap_size = 0;                                 /* Disable heap */
_max_hart_id = 0;                               /* One harts present */
_hart_stack_size = SIZEOF(.stack);              
_stack_start = ORIGIN(REGION_STACK) + LENGTH(REGION_STACK);

/* Sections from riscv-rt */
/* Put reset handler first in .text section so it ends up as the entry */
/* point of the program. */
PROVIDE(_start_trap=ulp_irq);

/*
KEEP(*(.init));
. = ALIGN(4);
KEEP(*(.trap.vector));   
KEEP(*(.trap.start));    
KEEP(*(.trap.start.*));  
KEEP(*(.trap.continue)); 
KEEP(*(.trap.rust));     
KEEP(*(.trap .trap.*));  
*(.text.abort);
*/
