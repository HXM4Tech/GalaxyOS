# ![GalaxyOS-logo](https://user-images.githubusercontent.com/68482946/201522597-a5579bed-eb51-4139-ba85-2843a59ec2d1.svg)
**Linux ABI-compatible** kernel written in Rust
<img src="https://user-images.githubusercontent.com/68482946/201523093-e398ba2f-62c1-4700-a412-191dcd1bd3f9.png" alt="rust-logo" height="24px" align="center"/>

#### 🖼️ Screenshot (v0.1.0-alpha.1)

<img src="https://user-images.githubusercontent.com/68482946/201498195-18769c05-db98-4e94-ba9a-368a4e3f848d.png" alt="screenshot" width="45%"/>

## 📦 Build dependencies
To compile GalaxyOS kernel and create basic OS ISO image you need to install following dependencies:
- python3 (to run `build.py` script)
- nasm
- rust nightly toolchain (with rust-src component)
- grub
- xorriso
- GNU mtools

#### You can install them with following commands:

Debian-based distributions:
```sh
apt install python3 nasm grub-common xorriso mtools curl   # as root
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
rustup toolchain install nightly
rustup component add rust-src
```

Arch-based distributions:
```sh
pacman -S python3 nasm rustup grub xorriso mtools   # as root
rustup toolchain install nightly
rustup component add rust-src
```

## ⚙️ Compiling
Clone GalaxyOS repository and change working directory to its location.

To compile use `build.py` Python (3.6+) script.
You can change `_*_BINARY` variales inside the script to specify non-standard executable locations.
```sh
chmod +x build.py
./build.py
# or
python3 build.py
```

## ▶️ Running
The kernel binary and basic OS iso are located in `build/` directory after compiling.

If you have qemu installed, you can run basic OS in the qemu virtual machine with:
```sh
qemu-system-x86_64 -cdrom build/galaxyos.iso
```

## 📃 License
This project is distributed under the [MIT License](https://en.wikipedia.org/wiki/MIT_License), see [LICENSE](https://github.com/HXM4Tech/GalaxyOS/blob/master/LICENSE)

*Copyright (c) 2022 Krzysztof Stefańczyk*
