__BOOTLOADER_LOAD_ADDR = 32K;
__WORD_ALIGN =4;
__STACK_ALIGN = 8;
__STACK_SIZE = 32K;

ENTRY(_start)

SECTIONS {
    .text __BOOTLOADER_LOAD_ADDR : {
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
        . = ALIGN(__WORD_ALIGN);
        *(.data*)
        . = ALIGN(__WORD_ALIGN);
    }
    .bss : {
        . = ALIGN(__WORD_ALIGN);
        PROVIDE(__bss_start = .);
        *(.bss*)
        *(COMMON)
        . = ALIGN(__WORD_ALIGN);
        PROVIDE(__bss_end = .);
    }
    .stack (COPY) : {
        . = ALIGN(__STACK_ALIGN);
        PROVIDE(__stack_start = .);
        . = . + __STACK_SIZE;
        . = ALIGN(__STACK_ALIGN);
        PROVIDE(__stack_init = .);
        PROVIDE(__stack_end = .);
        PROVIDE(__exec_end = .);
    }
}