#!/bin/bash

nice_path() {
  perl -le "use File::Spec;print File::Spec->abs2rel(@ARGV)" "$1" "$(pwd)"
}

elf_path=$1
target_dir=$(nice_path "$(dirname "$elf_path")")
base_name=$(basename "$elf_path")
bin_path="${target_dir}/${base_name/.elf/}.bin"

load_addr=0x40000000

riscv64-elf-objcopy "$elf_path" -O binary "$bin_path"
echo "Created $bin_path from $elf_path"
xfel ddr d1
xfel write $load_addr "$bin_path"
xfel exec $load_addr
