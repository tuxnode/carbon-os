# AGENTS.md

CarbonOS is a build-from-scratch immutable Linux distro (LLVM/Clang + musl + Rust `uutils`). The deliverable is a bootable initramfs/EROFS image, not a normal application.

## Build

- Autoconf-based: `./configure [--with-target=ARCH] [--disable-erofs]` then `make`.
- `configure` and `Makefile.in` are tracked; `Makefile` is generated and gitignored. Edit `configure.ac` and run `autoconf` to regenerate the committed `configure`; edit `Makefile.in` and re-run `./configure` (or `./config.status Makefile`) to regenerate `Makefile`. Editing `Makefile` directly is lost.
- `make` = `kernel userland brush init rootfs`; `userland`/`brush` depend on `musl`, so the musl static sysroot (`build/sysroot`) is built automatically before the Rust userland. `init` is CarbonOS' own crate in `init/` (PID 1).
- `make run-qemu` boots `build/Image` + `build/rootfs.cpio.gz` in QEMU (`virt`, riscv64, `-nographic`).
- No tests, lint, or CI exist in this repo.

## Requirements

- Host tools checked by `configure` (it fails fast): `clang`, `ld.lld`, `llvm-ar`, `llvm-ranlib`, `cpio`, `cargo`/`rustup`, `mkfs.erofs` (unless `--disable-erofs`); `qemu-system-riscv64` only warns.
- The Rust musl target (e.g. `riscv64gc-unknown-linux-musl`) is auto-installed via `rustup` during `configure`.

## Gotchas

- riscv64 is the only target actually wired up: the kernel target hardcodes `ARCH=riscv` and musl hardcodes `--target=riscv64-linux-musl`. `--with-target` only changes `RUST_TARGET` and the rustup target.
- `make musl` configures musl with `--disable-shared` into `build/sysroot/usr`. The host LLVM has no riscv64 compiler-rt, so shared musl libs and `libc.so` cannot link; the static sysroot is enough for the Rust userland, which links against Rust's bundled musl.
- `uutils` needs the musl headers in `build/sysroot` for its Oniguruma C dependency (`expr`); without `--sysroot` clang falls back to host `/usr/include` and fails with `__float128 is not supported on this target`.
- The `brush` shell (`vendor/brush`) is installed as `/bin/brush` and symlinked to `/bin/sh`. `/sbin/init` is the `init/` Rust binary: it calls `mount(2)` directly for `/proc`,`/sys`,`/dev`,`/dev/pts`,`/tmp`,`/run`, sets the hostname, prints the banner, then supervises `/bin/sh` on `/dev/console` (fork + setsid + `TIOCSCTTY`) and restarts it on exit so PID 1 never dies. `/etc/init.d/rcS` is legacy and no longer used. uutils has no `mount`, so filesystems are mounted by init itself.
- The `init` crate deliberately avoids `unwrap`/`expect` (a PID 1 panic is fatal) and uses only `std` plus a few `extern "C"` syscalls (`mount`, `setsid`, `ioctl`, `sethostname`); it is built with `+crt-static` like brush.
- brush does not ship uutils' `.cargo/config.toml`, so the Makefile passes `CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static"`; without it the link fails with `unable to find library -lgcc_s`/`-lc`.
- Applet symlinks are derived from uutils' generated `target/$(RUST_TARGET)/release/build/coreutils-*/out/uutils_map.rs` (the same list as `coreutils --list`) rather than GNU's `--install`, which uutils does not implement. The riscv64 `coreutils` binary cannot be executed on an x86_64 host; `qemu-riscv64` is only needed if you want to run it manually.
- `userland` builds uutils with `--features feat_Tier1,feat_require_unix_musl`; the default `feat_common_core` omits `uname`/`hostname`/`kill`/`arch`/`nproc`/`timeout`/`uptime`/`whoami` and `id`/`groups`/`chmod`/`chown`/`stat`/`stty`/`mknod`. Use `feat_require_unix_musl` rather than `feat_os_unix`, which pulls in the `stdbuf` cdylib that musl cannot link. `mount`, `ps`, and `free` are not part of coreutils at all.
- `vendor/{linux,musl,uutils,brush}` are pinned submodules (linux v6.6.21, musl v1.2.5, uutils 0.11.0, brush v0.4.0). `configure` shallow-inits them if absent. Never edit code under `vendor/`.
- `packages/` and `scripts/` are empty placeholders.
- `overlay/` is copied verbatim into the rootfs.
