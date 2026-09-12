AC_INIT([CarbonOS], [0.1.0], [tuxnode@outlook.com])
AC_PREREQ([2.69])

AC_CONFIG_SRCDIR([overlay/etc/init.d/rcS])

echo "===================================================="
echo "       CarbonOS Autoconf Configuration              "
echo "===================================================="

# command line config
AC_ARG_WITH([target],
  [AS_HELP_STRING([--with-target=ARCH], [Specify target architecture (x86_64, aarch64, riscv64) @<:@default=riscv64@:>@])],
  [TARGET_ARCH="$withval"],
  [TARGET_ARCH="riscv64"])

AC_ARG_ENABLE([erofs],
  [AS_HELP_STRING([--disable-erofs], [Disable EROFS image generation])],
  [ENABLE_EROFS="$enableval"],
  [ENABLE_EROFS="yes"])

AC_SUBST([TARGET_ARCH])
AC_SUBST([ENABLE_EROFS])

# git submodules
AC_MSG_CHECKING([Git Submodules status])
if test -d "${srcdir}/.git"; then
    if test ! -f "${srcdir}/vendor/linux/Makefile" || test ! -f "${srcdir}/vendor/musl/configure"; then
        AC_MSG_RESULT([Incomplete, fetching now...])
        git submodule update --init --recursive --depth 1 || AC_MSG_ERROR([Failed to fetch git submodules])
    else
        AC_MSG_RESULT([OK])
    fi
else
    AC_MSG_RESULT([Not a git repo, skipping])
fi

# LLVM and build-essential tools
AC_CHECK_PROGS([CC], [clang], [none])
if test "$CC" = "none"; then
    AC_MSG_ERROR([clang is required but was not found on your host.])
fi

AC_CHECK_PROGS([LLD], [ld.lld], [none])
if test "$LLD" = "none"; then
    AC_MSG_ERROR([ld.lld (LLVM linker) is required but was not found.])
fi

AC_CHECK_PROG([MAKE_CHECK], [make], [yes], [no])
if test "$MAKE_CHECK" = "no"; then
    AC_MSG_ERROR([GNU Make is required.])
fi

AC_CHECK_PROG([CPIO_CHECK], [cpio], [yes], [no])
if test "$CPIO_CHECK" = "no"; then
    AC_MSG_ERROR([cpio is required to build initramfs.])
fi

# qemu riscv64
if test "$TARGET_ARCH" = "riscv64"; then
    AC_CHECK_PROGS([QEMU_RISCV], [qemu-system-riscv64], [none])
    if test "$QEMU_RISCV" = "none"; then
        AC_MSG_WARN([qemu-system-riscv64 not found. You will not be able to run QEMU target.])
    fi
fi

# Rust tool chain
AC_CHECK_PROGS([CARGO], [cargo], [none])
if test "$CARGO" = "none"; then
    AC_MSG_ERROR([Rust/Cargo is required to build uutils coreutils.])
fi

if test "$TARGET_ARCH" = "riscv64"; then
    RUST_TARGET="riscv64gc-unknown-linux-musl"
elif test "$TARGET_ARCH" = "aarch64"; then
    RUST_TARGET="aarch64-unknown-linux-musl"
else
    RUST_TARGET="${TARGET_ARCH}-unknown-linux-musl"
fi
AC_SUBST([RUST_TARGET])

AC_CHECK_PROGS([RUSTUP], [rustup], [none])
if test "$RUSTUP" = "none"; then
    AC_MSG_WARN([rustup not found, skipping automatic musl target check.])
else
    AC_MSG_CHECKING([Rust target $RUST_TARGET])
    if rustup target list | grep -q "${RUST_TARGET} (installed)"; then
        AC_MSG_RESULT([installed])
    else
        AC_MSG_RESULT([not installed. Installing...])
        rustup target add "${RUST_TARGET}" || AC_MSG_ERROR([Failed to install Rust target ${RUST_TARGET}])
    fi
fi

# EROFS tools
if test "$ENABLE_EROFS" = "yes"; then
    AC_CHECK_PROGS([MKFS_EROFS], [mkfs.erofs], [none])
    if test "$MKFS_EROFS" = "none"; then
        AC_MSG_ERROR([mkfs.erofs is required when EROFS is enabled. Pass --disable-erofs to skip.])
    fi
fi

AC_CONFIG_FILES([Makefile])
AC_OUTPUT

echo "----------------------------------------------------"
echo " Configuration Successful!"
echo " Target Arch : ${TARGET_ARCH}"
echo " Rust Target : ${RUST_TARGET}"
echo " EROFS       : ${ENABLE_EROFS}"
echo " Now run 'make' to start building CarbonOS."
echo "----------------------------------------------------"
