#!/usr/bin/env python3

### BUILD SCRIPT FOR GalaxyOS ###

import os
import sys
import shutil

# CHANGE FOLLOWING TO SPECIFY LOCATIONS OF NEEDED BINARIES
_ASM_BINARY   = "nasm"
_LD_BINARY    = "ld"
_GRUB_BINARY  = "grub-mkrescue"
_CARGO_BANARY = "cargo"
_STRIP_BINARY = "strip"

SELFPATH = os.path.dirname(os.path.realpath(__file__))
BOOTCODE = os.path.join(SELFPATH, "src/boot/")
ISOFILES = os.path.join(SELFPATH, "src/isofiles/")
KERNEL   = os.path.join(SELFPATH, "src/kernel/")

OUTPUTS  = os.path.join(os.getcwd(), "build/")

try:
    shutil.rmtree(OUTPUTS)
except Exception:
    pass

try:
    if not os.path.exists(OUTPUTS):
        os.mkdir(OUTPUTS)
except Exception:
    print(f"\nBUILDER: ERROR: The build directory ({OUTPUTS}) cannot be created: os.mkdir() raised exception.", file=sys.stderr)
    raise SystemExit(1)

class BuildError(Exception):
    """Raised when build failed"""
    def __str__(self):
        return f"BUILDER: ERROR: ^^^ Command returned non-zero exit code. ^^^"


def make_bootcode():
    for asm_file in [x for x in os.listdir(BOOTCODE) if x.endswith(".asm")]:
        inp = f'"{os.path.join(BOOTCODE, asm_file)}"'
        of = f'"{os.path.join(OUTPUTS, ".".join(asm_file.split(".")[:-1]) + ".out")}"'

        cmd = f"{_ASM_BINARY} -f elf64 -o {of} {inp}"
        print(f"$ {cmd}")
        exit_code = os.system(cmd)
        try:
            assert exit_code == 0
        except AssertionError:
            raise BuildError(exit_code)

def make_kernel():
    cmd = f"RUST_TARGET_PATH=\"{SELFPATH}\" {_CARGO_BANARY} build --manifest-path {os.path.join(SELFPATH, 'Cargo.toml')} --target-dir \"{OUTPUTS}\" -r"
    print(f"$ {cmd}")
    exit_code = os.system(cmd)
    try:
        assert exit_code == 0
    except AssertionError:
        raise BuildError(exit_code)

def link():
    ifs = " ".join([f'"{os.path.join(OUTPUTS, f)}"' for f in os.listdir(OUTPUTS) if f.endswith(".out")])
    ifs += f" \"{os.path.join(OUTPUTS, 'x86_64-galaxyos', 'release', 'libgalaxyos.a')}\""
    of = f'"{os.path.join(OUTPUTS, "galaxyos")}"'

    cmds = [
        f"{_LD_BINARY} -n -o {of} -T {os.path.join(SELFPATH, 'src/linker.ld')} {ifs}",
        f"{_STRIP_BINARY} -v -s {of}"
    ]
    for cmd in cmds:
        print(f"$ {cmd}")
        exit_code = os.system(cmd)
        try:
            assert exit_code == 0
        except AssertionError:
            raise BuildError(exit_code)

def make_iso():
    cmds = [
        f"cp -R \"{ISOFILES}\" \"{os.path.join(OUTPUTS, 'iso')}\"",
        f"cp \"{os.path.join(OUTPUTS, 'galaxyos')}\" \"{os.path.join(OUTPUTS, 'iso', 'boot', 'galaxyos')}\"",
        f"{_GRUB_BINARY} -o \"{os.path.join(OUTPUTS, 'galaxyos.iso')}\" \"{os.path.join(OUTPUTS, 'iso')}\""
    ]

    for cmd in cmds:
        print(f"$ {cmd}")
        exit_code = os.system(cmd)
        try:
            assert exit_code == 0
        except AssertionError:
            raise BuildError(exit_code)

def clean():
    # Leaves only kernel binary and iso image
    cmds = [
        f"rm -R \"{os.path.join(OUTPUTS, 'iso')}\"",
        f"rm \"{OUTPUTS}\"*out",
        f"rm -R \"{os.path.join(OUTPUTS, 'release')}\"",
        f"rm -R \"{os.path.join(OUTPUTS, 'x86_64-galaxyos')}\"",
        f"rm \"{OUTPUTS}\".*.json"
    ]

    for cmd in cmds:
        print(f"(ignore errors) $ {cmd}")
        os.system(cmd)


if __name__ == "__main__":
    try:
        make_bootcode()
        make_kernel()
        link()
        make_iso()
        clean()
    except BuildError as e:
        print(str(e))
    