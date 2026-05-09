SECTIONS
{
    /* Placed at the 'heap' location, after bss */
    /* . = ORIGIN(ram) + LENGTH(ram); */

    .ulp ALIGN(4):
    {
        *(.ulp)
        *(.ulp*)
    }

    /* Fix the stack location */
    __stack_top = .;
}
