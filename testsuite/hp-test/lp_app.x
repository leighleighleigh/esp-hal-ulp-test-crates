SECTIONS {
  .ulp.ULP_LOOP_COUNTER 0x500002f0 (NOLOAD):
  {
    *(.ulp.ULP_LOOP_COUNTER)
  }
  .ulp.ULP_COMMAND 0x500002f4 (NOLOAD):
  {
    *(.ulp.ULP_COMMAND)
  }
  .ulp.ULP_REPLY 0x500002f8 (NOLOAD):
  {
    *(.ulp.ULP_REPLY)
  }
  .ulp.__stack_top 0x500002fc (NOLOAD):
  {
    *(.ulp.__stack_top)
  }
}
