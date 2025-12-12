__DRAM_START = 0x40000000;
/* OrangePi One / Lite */
__DRAM_SIZE = 512M;
__WORD_ALIGN = 8;
__STACK_ALIGN = 16;
__STACK_SIZE = 32K;

SECTIONS {
    ENTRY(_start)
    .text __DRAM_START : {
        PROVIDE(__exec_start = .);
        KEEP(*(.text.start))
        *(.text*)
        . = ALIGN(__WORD_ALIGN);
    }
    .rodata : {
        . = ALIGN(__WORD_ALIGN);
        *(.rodata*)
        . = ALIGN(__WORD_ALIGN);
    }
    .data : {
        /* __global_pointer$ = . + 0x800; */
        . = ALIGN(__WORD_ALIGN);
        *(.data*)
        . = ALIGN(__WORD_ALIGN);
    }
    .bss : {
        . = ALIGN(__WORD_ALIGN);
        PROVIDE(__bss_start = .);
        *(.bss*)
        /* anything that wasn't explicitly assigned a section */
        *(COMMON)
        . = ALIGN(__WORD_ALIGN);
        PROVIDE(__bss_end = .);
    }
    .stack : {
        . = ALIGN(__STACK_ALIGN);
        PROVIDE(__stack_start = .);
        . = . + __STACK_SIZE;
        . = ALIGN(__STACK_ALIGN);
        PROVIDE(__stack_init = .);
        PROVIDE(__stack_end = .);
        PROVIDE(__exec_end = .);
    }

    PROVIDE(__memory_end = __DRAM_START + __DRAM_SIZE);
}
