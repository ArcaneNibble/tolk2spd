# Tolk to Speech Dispatcher bridge

This is a compatible replacemet implementation of the [Tolk](https://github.com/dkager/tolk) library designed exclusively for [Wine](https://www.winehq.org/), which bridges emulated Windows software to the Speech Dispatcher ([[1]](https://freebsoft.org/speechd) [[2]](https://github.com/brailcom/speechd) [[3]](https://wiki.archlinux.org/title/Speech_dispatcher)) socket interface.

This is simpler than setting up TTS _within_ Wine, and this also preserves all of the local speech synthesis settings that might have been configured on the native Linux host.

Tolk is often used as an abstraction layer by [audio games](https://en.wikipedia.org/wiki/Audio_game).

## Feature support

x86_64 Windows binaries are fully supported. x86 (32-bit) binaries only work when using the "new" Wine WOW64 implementation. Arm64 is not supported yet (but you can run a complete x86_64 Wine under emulation, and it will be able to talk to an aarch64 `speechd`).

Steam (Deck) + Proton does not work yet. This needs to be investigated.

The Tolk C API for speech synthesis is fully implemented. Most software uses this interface.

The JNI interface has not been implemented yet, so Java applications running under Wine will not work.

Braille output is not supported, since Speech Dispatcher doesn't support that.

There is no option to configure which voices are used, because the Tolk API doesn't expose that.

## Installation

To use this, you need to:

1. Patch your game (or other application)
2. Set the `WINEDLLPATH` environment variable _before_ launching

You can download pre-built releases from the Github "Releases" section. These are compiled for the Python `manylinux2014` build target (there is no Python code involved here, but this is being used as a well-specified Linux platform ABI target).

### Patching the application

To patch the application, you need to:

1. _Replace_ its `Tolk.dll` with the appropriate DLL from the pre-built package.
2. Place the Linux `.so` in the right place.

For a 32-bit program, use `tolk-win32.dll`. For a 64-bit program, use `tolk-win64.dll`. Both of these _must be renamed_ to `tolk.dll`, and this name _must be all lowercase_.

For example, games built using [NVGT](https://github.com/samtupy/nvgt) will have a `Tolk.dll` under the `lib` folder. Delete that `Tolk.dll` file, and then copy `tolk-win64.dll` as `lib/tolk.dll`.

The `.so` file would normally be extracted to `lib/tolk.so` (next to the DLL). See the next step for details.

### Setting the environment variable

The `WINEDLLPATH` environment variable needs to be set to include the (_Linux_) path to the directory containing `tolk.so`. This needs to be set _before_ launching the program.

For example, to launch a NVGT game with both files under the `lib` directory, run:

```shell
WINEDLLPATH=$(pwd)/lib wine the-game.exe
```

### Building from source

Building this code from source is somewhat tricky because Cargo [doesn't currently stably support different targets per package](https://doc.rust-lang.org/cargo/reference/unstable.html#per-package-target). Therefore, each component has its own Cargo settings.

#### Using the `Dockerfile`

To build a complete release package, use:

```shell
docker build . --output=.
```

This will produce a `tolk2spd.zip` in the current directory (as well as a `busybox` binary which is useless for developing but is necessary for the Github CI/CD pipeline).

#### Manually building Windows component

The Windows component is in `tolk2spd_win32` and generally should be built using the `build-debug.sh` script when testing. This is because the DLL files need post-build patching.

This only builds an x86_64 DLL. A 32-bit DLL can be build by passing `--target` to `cargo build`, but there's no script to automate the patching.

The default and recommended build target is `x86_64-pc-windows-gnullvm` because it does not require any external compiler tools, especially Microsoft proprietary tools. (Only external libraries are required.)

The default `.cargo/config.toml` is set up to expect mingw-w64 libraries in parallel to the `tolk2spd` directory. For example, if you have a git checkout at `~/code/tolk2spd`, it will expect `~/code/mingw-w64-minimal/{i686,x86_64}-w64-mingw32/{include,lib}`. Appropriate libraries can be obtained from [here](https://github.com/ArcaneNibble/mingw-w64-libs-only/).

#### Manually building Linux component

The Linux component is in `tolk2spd_unix`. It can be built normally using `cargo`.

This component should always be 64-bit (a 32-bit _Linux host_ can theoretically work but is _very_ untested). The default settings force a build for x86_64 Linux glibc using `manylinux2014` as a sysroot, which must also be extracted in parallel to the `tolk2spd` directory.

If you are developing _on_ Linux, you can clear `RUSTFLAGS`
and use your default build/linker settings. This avoids having to download a sysroot but no longer guarantees ABI portability.

After building, the resulting `.so` needs to be renamed to `tolk.so` and placed in the correct location.
