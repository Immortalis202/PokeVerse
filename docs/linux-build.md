# Linux x86-64 build

PokeVerse 0.30.0 uses the same engine 0.23.0-core, protocol 31 and schema 4 on Windows and Linux. The Linux bundle is intended for the user's Intel/AMD x86-64 Ubuntu 26.10 computer. It is built with an Ubuntu 26.04 x86-64 toolchain; native operation on the other computer and Windows/Linux Tailscale acceptance remain separate checks.

## Install and connect

Copy the Linux delivery files to the Ubuntu PC. In the directory containing the Debian package:

```bash
sudo apt install ./PokeVerse_0.30.0_amd64.deb
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


## Verified build — 23 October 2026

The Ubuntu 26.04 WSL2 x86-64 release build produced both bundles successfully, using Rust 1.93.1 and checksum-verified Node 24.22.1. All **651 Rust tests passed on Linux** (`cargo test --release --locked --lib -- --test-threads=4`), including localhost room/GM/owner, retry, reconnect, rollback and persistence scenarios. The Rust dependency lockfile matches Windows after normalizing line endings. The frontend production build passed.

The Debian package declares `amd64`, version 0.30.0 and its GTK/WebKit runtime dependencies. Its ELF is x86-64 and all linked libraries resolved on the build system. Both the extracted Debian package and extracted AppImage contain the exact Windows-release catalog (SHA256 `284344d2c3b6c40e658e6d3d9387cedf068ff464d9da19e866f2513af3c5f7f4`). The GUI and installers were not launched; Ubuntu 26.10 installation, display behavior and physical Windows/Linux Tailscale remain unverified.

Delivery folder: `src-tauri/target/linux-bundle/`. The ZIP contains the `.deb`, `.AppImage`, `SHA256SUMS`, `INSTALL.txt` and `BUILD-INFO.json` with exact package hashes and verification boundaries. Windows executable/installer files are unchanged.

| Package | SHA256 |
| --- | --- |
| PokeVerse_0.30.0_amd64.deb | `90fb202fa5602789126cc7e6c3888b9f2fa794decaf6ecb19aa7c050a1c76678` |
| PokeVerse_0.30.0_amd64.AppImage | `fb9fccec2677f3731c90c8693f97dc7cf7fb2c40ee0d933f14ffa918b33eb964` |
