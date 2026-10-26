# Linux x86-64 build

PokeVerse 0.32.0 uses the same engine 0.25.0-core, protocol 33 and schema 4 on Windows and Linux. The Linux bundle is intended for the user's Intel/AMD x86-64 Ubuntu 26.10 computer. It is built with an Ubuntu 26.04 x86-64 toolchain; native operation on the other computer and Windows/Linux Tailscale acceptance remain separate checks.

## Install and connect

Copy the Linux delivery files to the Ubuntu PC. In the directory containing the Debian package:

```bash
sudo apt install ./PokeVerse_0.32.0_amd64.deb
```

Open PokeVerse from the application menu. The package manager installs required system libraries; the app includes its reference catalog and sprites. Create a fresh Linux identity rather than copying a Windows profile to create a second client. Join the matching Windows host with `ws://WINDOWS-TAILSCALE-IP:39231/room`, using its displayed port and access token. Installing/configuring Tailscale and firewall changes are not performed by the build.

If using the AppImage instead, make it executable and run it from its own folder. It does not support every Linux distribution merely because it is an AppImage; the host still needs compatible core libraries. Keep install/startup results separate from an automated build result.

## Rebuild

Install Node.js 24, Rust and the Linux system dependencies listed in the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/). The build used Ubuntu packages `build-essential`, `pkg-config`, `curl`, `ca-certificates`, `xz-utils`, `rustc`, `cargo`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libxdo-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf` and `file`.

With `src-tauri/resources/catalog.db` and `public/sprites/` already generated from the matching source checkout:

```bash
npm ci
npm run package:linux -- --ci
```

`tauri.linux.conf.json` selects Debian/AppImage targets and compiles the frontend using those prepared assets. It does not regenerate the source/PDF catalog during packaging. If building from a fresh source checkout, generate the catalog first with `npm run catalog` using the documented sibling `Pokerole-Data` directory. Do not copy Windows `node_modules` or Rust target files into a Linux build tree.

Outputs are under `src-tauri/target/release/bundle/deb/` and `src-tauri/target/release/bundle/appimage/`. For headless AppImage packaging, `APPIMAGE_EXTRACT_AND_RUN=1` allows build helpers to run without a FUSE mount. The normal Windows configuration retains its NSIS target.

See [Debian packaging](https://v2.tauri.app/distribute/debian/) and [AppImage limitations](https://v2.tauri.app/distribute/appimage/) for platform packaging requirements.


## Current v0.32.0 verification — 26 October 2026

The GM can select [starting multiplayer battlefield conditions](battle-starting-environment.md) before battle. Wild/training encounters remain separate. All **672 Rust tests passed on Windows and Linux** (release mode on Linux), plus 63 frontend tests, 21 catalog/script checks, TypeScript, Clippy and formatting/diff checks. Windows NSIS and Linux Debian/AppImage production builds passed.

The Debian package declares version 0.32.0, architecture `amd64`, and GTK3/WebKit4.1 dependencies. Its ELF is x86-64 and all linked libraries resolve on the build system. Both extracted Linux packages contain catalog SHA256 `e8096209cee398d7e891b39b7d7c5f2018064e1f120634e230a0c79299cd4a6e`, matching Windows. Native installation, GUI and physical Tailscale acceptance remain unverified.

Delivery: `src-tauri/target/linux-bundle/0.32.0/PokeVerse_0.32.0_linux_x64.zip`; matching Windows installer: `src-tauri/target/release/bundle/nsis/PokeVerse_0.32.0_x64-setup.exe`. Use matching v0.32.0/protocol 33 clients, and upgrade the campaign to engine 0.25.0-core between battles to enable the new setup. Schema 4 is retained.

| Package | SHA256 |
| --- | --- |
| PokeVerse_0.32.0_amd64.deb | `46298921dbd928aa921977bf65d06d4260b146ecf8c53fa06677b30f48ed4860` |
| PokeVerse_0.32.0_amd64.AppImage | `f0cab19ccf8227f8963b139b0420a63a7f161080175281c28299d4964b29bb8a` |
| PokeVerse_0.32.0_x64-setup.exe | `77c6df1afd585fd58b76ef863e1416f7ae49318962e3baeee800bae8c0222320` |

## Previous v0.31.1 verification — 26 October 2026

Mirror Coat is corrected to the supplied Special-move-only Late Reaction 5 wording; see [the correction and compatibility](battle-group-rules.md#mirror-coat-correction--v0311). All **665 Rust tests passed on Windows and Linux** (release mode on Linux), plus 60 frontend tests, 21 script/catalog checks, TypeScript, Clippy and formatting checks. Windows NSIS and Linux Debian/AppImage production builds passed.

The Debian package declares version 0.31.1, architecture `amd64`, and GTK3/WebKit4.1 dependencies. The ELF is x86-64 and all linked libraries resolve on the build system. Both extracted Linux packages contain catalog SHA256 `4c0e39b7d090bc7731a180359981b341bc012477fc5913e53adb7b577bb9faac`, matching Windows. Native installation, GUI and physical Tailscale acceptance remain unverified.

Delivery: `src-tauri/target/linux-bundle/0.31.1/PokeVerse_0.31.1_linux_x64.zip`; matching Windows installer: `src-tauri/target/release/bundle/nsis/PokeVerse_0.31.1_x64-setup.exe`. Use matching builds, and upgrade between battles.

| Package | SHA256 |
| --- | --- |
| PokeVerse_0.31.1_amd64.deb | `e007d30f6565db7161d76f4df994577db841b979d8d7a6afe82fd508d48cb9a2` |
| PokeVerse_0.31.1_amd64.AppImage | `eedf06327034b8baf200ed4638d639ae7832cba2ba2418a9fc2517d045457d5c` |
| PokeVerse_0.31.1_x64-setup.exe | `20b94f94b434e27de975181108be6b70da81f601cf5b3970919e09d960510391` |

## Previous v0.31.0 verification — 26 October 2026

Windows NSIS and Linux Debian/AppImage production builds passed. All **662 Rust tests passed on Windows and on Linux** (Linux release mode), alongside 60 frontend tests, 21 script/catalog checks, TypeScript, Rust formatting and Clippy checks. The selected group rules and historical encounter behavior are covered in [battle-group-rules.md](battle-group-rules.md).

The Debian package declares version 0.31.0, architecture `amd64`, and GTK3/WebKit4.1 dependencies. Its ELF is x86-64; all linked libraries resolve on the Ubuntu build system. Both extracted Linux packages contain the Windows catalog with SHA256 `f2793511fef9bbf3c24908698fbf4b5833499a29b11359cddd17646e96a1b29e`. Native installation, GUI and physical Tailscale testing remain unverified.

Delivery: `src-tauri/target/linux-bundle/0.31.0/PokeVerse_0.31.0_linux_x64.zip`, containing both packages, installation instructions, checksums and build metadata. The matching Windows installer is `src-tauri/target/release/bundle/nsis/PokeVerse_0.31.0_x64-setup.exe`. Upgrade both clients together; protocol 32 requires matching builds, and the new group rules apply to new current-engine encounters.

| Package | SHA256 |
| --- | --- |
| PokeVerse_0.31.0_amd64.deb | `84c68d0b04277ff6b930fb46678c514d38182aa74849dc902d30c3863386dc7d` |
| PokeVerse_0.31.0_amd64.AppImage | `d5c3229003c343ae215002d9a5bc90c60b06ae5e3ce7fdd93bcf9f72dc9947b5` |
| PokeVerse_0.31.0_x64-setup.exe | `90844d19b25c1f5cadb816d25b902d191bdba4f4572b8738e4088ad51134f1f0` |

## Previous v0.30.0 verification — 23 October 2026

The Ubuntu 26.04 WSL2 x86-64 release build produced both bundles successfully, using Rust 1.93.1 and checksum-verified Node 24.22.1. All **651 Rust tests passed on Linux** (`cargo test --release --locked --lib -- --test-threads=4`), including localhost room/GM/owner, retry, reconnect, rollback and persistence scenarios. The Rust dependency lockfile matches Windows after normalizing line endings. The frontend production build passed.

The Debian package declares `amd64`, version 0.30.0 and its GTK/WebKit runtime dependencies. Its ELF is x86-64 and all linked libraries resolved on the build system. Both the extracted Debian package and extracted AppImage contain the exact Windows-release catalog (SHA256 `284344d2c3b6c40e658e6d3d9387cedf068ff464d9da19e866f2513af3c5f7f4`). The GUI and installers were not launched; Ubuntu 26.10 installation, display behavior and physical Windows/Linux Tailscale remain unverified.

Delivery folder: `src-tauri/target/linux-bundle/`. The ZIP contains the `.deb`, `.AppImage`, `SHA256SUMS`, `INSTALL.txt` and `BUILD-INFO.json` with exact package hashes and verification boundaries. Windows executable/installer files are unchanged.

| Package | SHA256 |
| --- | --- |
| PokeVerse_0.30.0_amd64.deb | `90fb202fa5602789126cc7e6c3888b9f2fa794decaf6ecb19aa7c050a1c76678` |
| PokeVerse_0.30.0_amd64.AppImage | `fb9fccec2677f3731c90c8693f97dc7cf7fb2c40ee0d933f14ffa918b33eb964` |
