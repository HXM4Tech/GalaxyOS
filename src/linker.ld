ENTRY(start)

SECTIONS {
  . = 1M;

  .rodata :
  {
    KEEP(*(.multiboot_header))
    KEEP(*(.eh_frame))
    *(.rodata .rodata.*)
    . = ALIGN(4K);
  }
  
  .text :
  {
    *(.text .text.*)
    . = ALIGN(4K);
  }

  .data :
  {
    *(.data .data.*)
    . = ALIGN(4K);
  }

  .bss :
  {
    *(.bss .bss.*)
    . = ALIGN(4K);
  }

  .got :
  {
    *(.got)
    . = ALIGN(4K);
  }

  .got.plt :
  {
    *(.got.plt)
    . = ALIGN(4K);
  }

  .data.rel.ro : ALIGN(4K) {
    *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    . = ALIGN(4K);
  }

  .gcc_except_table : ALIGN(4K) {
    *(.gcc_except_table)
    . = ALIGN(4K);
  }

  .debug :
  {
    *(.debug*)
    . = ALIGN(4K);
  }
}
