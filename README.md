# GalaxyOS
Linux ABI-compatible kernel written in Rust

## Compiling
To compile use `build.py` Python (3.6+) script
You can change `_*_BINARY` variales inside the script to specify non-standard executable locations
```sh
./build.py
# or
python3 build.py
```

## Running
The kernel binary and basic OS iso are located in `build/` directory after compiling
You can run basic OS in the qemu virtual machine with:
```sh
qemu-system-x86_64 -cdrom build/galaxyos.iso
```

Copyright (c) 2022 Krzysztof Stefańczyk
