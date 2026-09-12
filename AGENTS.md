# AGENTS.md

CarbonOS is a build-from-scratch immutable Linux distro (LLVM/Clang + musl + Rust `uutils`). The deliverable is a bootable initramfs/EROFS image, not a normal application.

## Build

- Autoconf-based: `./configure [--with-target=ARCH] [--disable-erofs]` then `make`.
- `configure` and `Makefile.in` are tracked; `Makefile` is generated and gitignored. Edit `configure.ac` and run `autoconf` to regenerate the committed `configure`; edit `Makefile.in` and re-run `./configure` (or `./config.status Makefile`) to regenerate `Makefile`. Editing `Makefile` directly is lost.
- `make` = `kernel userland rootfs`; `userland` depends on `musl`, so the musl static sysroot (`build/sysroot`) is built automatically before the Rust userland.
- `make run-qemu` boots `build/Image` + `build/rootfs.cpio.gz` in QEMU (`virt`, riscv64, `-nographic`).
- No tests, lint, or CI exist in this repo.

## Requirements

- Host tools checked by `configure` (it fails fast): `clang`, `ld.lld`, `llvm-ar`, `llvm-ranlib`, `cpio`, `cargo`/`rustup`, `mkfs.erofs` (unless `--disable-erofs`); `qemu-system-riscv64` only warns.
- The Rust musl target (e.g. `riscv64gc-unknown-linux-musl`) is auto-installed via `rustup` during `configure`.

## Gotchas

- riscv64 is the only target actually wired up: the kernel target hardcodes `ARCH=riscv` and musl hardcodes `--target=riscv64-linux-musl`. `--with-target` only changes `RUST_TARGET` and the rustup target.
- `make musl` configures musl with `--disable-shared` into `build/sysroot/usr`. The host LLVM has no riscv64 compiler-rt, so shared musl libs and `libc.so` cannot link; the static sysroot is enough for the Rust userland, which links against Rust's bundled musl.
- `uutils` needs the musl headers in `build/sysroot` for its Oniguruma C dependency (`expr`); without `--sysroot` clang falls back to host `/usr/include` and fails with `__float128 is not supported on this target`.
- `run-qemu` passes `rdinit=/sbin/init`, but the rootfs only installs `/etc/init.d/rcS` (no `/sbin/init`, and no `sh`/`mount`), so the current image will not reach `rcS` on boot.
- `rootfs` runs `coreutils --install -s .` in `/bin`, but the binary is riscv64 and cannot execute on an x86_64 host; the `|| true` masks the failure, so no applet symlinks are created during a normal cross-build.
- `vendor/{linux,musl,uutils}` are pinned submodules (linux v6.6.21, musl v1.2.5, uutils 0.11.0). `configure` shallow-inits them if absent. Never edit code under `vendor/`.
- `packages/` and `scripts/` are empty placeholders.
- `overlay/` is copied verbatim into the rootfs.
