FROM quay.io/pypa/manylinux2014_x86_64 AS manylinux

WORKDIR /
RUN ln -s libgcc_s.so.1 usr/lib64/libgcc_s.so
RUN tar cf /manylinux.tar --exclude=/proc --exclude=/dev --exclude=/sys /

FROM debian:trixie AS build

# NOTE: gcc is required for building native build scripts etc
RUN apt update && apt -y install gcc llvm curl zip busybox-static
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add i686-pc-windows-gnullvm
RUN rustup target add x86_64-pc-windows-gnullvm
RUN rustup target add x86_64-unknown-linux-gnu

# Grab mingw-w64 libs
WORKDIR /build/mingw-w64-minimal
RUN curl -OL https://github.com/ArcaneNibble/mingw-w64-libs-only/releases/download/v14.0.0/mingw-w64-v14.0.0.tar.gz
RUN tar xzf mingw-w64-v14.0.0.tar.gz
RUN rm -f mingw-w64-v14.0.0.tar.gz

# Grab manylinux sysroot
WORKDIR /build/manylinux2014
COPY --from=manylinux /manylinux.tar .
RUN tar xf manylinux.tar
RUN rm -f manylinux.tar

# Run the build
COPY . /build/tolk2spd
RUN mkdir -p /build/package
# Linux-side
WORKDIR /build/tolk2spd/tolk2spd_unix
RUN rm -rf target/
RUN cargo build --release --target x86_64-unknown-linux-gnu
RUN cp target/x86_64-unknown-linux-gnu/release/libtolk2spd_unix.so /build/package/tolk.so
# Win32
WORKDIR /build/tolk2spd/tolk2spd_win32
RUN rm -rf target/
RUN cargo build --release --target i686-pc-windows-gnullvm
RUN /usr/bin/printf 'Wine builtin DLL\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00' | dd of=target/i686-pc-windows-gnullvm/release/tolk.dll seek=64 bs=1 conv=notrunc
RUN cp target/i686-pc-windows-gnullvm/release/tolk.dll /build/package/tolk-win32.dll
# Win64
RUN cargo build --release --target x86_64-pc-windows-gnullvm
RUN /usr/bin/printf 'Wine builtin DLL\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00' | dd of=target/x86_64-pc-windows-gnullvm/release/tolk.dll seek=64 bs=1 conv=notrunc
RUN cp target/x86_64-pc-windows-gnullvm/release/tolk.dll /build/package/tolk-win64.dll

# Build an archive
WORKDIR /build/package
RUN zip -r /build/tolk2spd.zip .

FROM scratch
COPY --from=build /build/tolk2spd.zip /
COPY --from=build /usr/bin/busybox /

# Copies output for Github Actions
ENTRYPOINT [ "/busybox", "cp", "/tolk2spd.zip", "/github/workspace" ]
