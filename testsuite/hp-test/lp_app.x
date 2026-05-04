SECTIONS {
  .ulp.ULP_SHARED_DATA 0x50000640 (NOLOAD):
  {
    *(.ulp.ULP_SHARED_DATA)
  }
  .ulp.__stack_top 0x50000644 (NOLOAD):
  {
    *(.ulp.__stack_top)
  }
}
