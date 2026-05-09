SECTIONS {
  .ulp.ULP_COMMAND 0x500000a8 (NOLOAD):
  {
    *(.ulp.ULP_COMMAND)
  }
  .ulp.ULP_REPLY 0x500000ac (NOLOAD):
  {
    *(.ulp.ULP_REPLY)
  }
  .ulp.ULP_LOOP_COUNTER 0x500000b0 (NOLOAD):
  {
    *(.ulp.ULP_LOOP_COUNTER)
  }
  .ulp.__stack_top 0x500000b4 (NOLOAD):
  {
    *(.ulp.__stack_top)
  }
}
