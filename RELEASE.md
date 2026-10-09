# EEG Lab Release Notes

## Overview

EEG Lab is a full-stack neuroscience SaaS for EEG studies, interactive signal exploration, and asynchronous analyses with organisation-scoped access control.

## Platform Downloads

### Linux

| No  | Package               | File               | Command                          |
| --- | --------------------- | ------------------ | -------------------------------- |
| 1   | **AppImage**          | `eeg-lab.AppImage` | Run anywhere (no install)        |
| 2   | **Debian/Ubuntu**     | `eeg-lab.deb`      | `sudo apt install ./eeg-lab.deb` |
| 3   | **RPM (Fedora/RHEL)** | `eeg-lab.rpm`      | `sudo dnf install ./eeg-lab.rpm` |
| 4   | **Tarball**           | `eeg-lab.tar.xz`   | Extract and run `./eeg-lab`      |

### macOS

| No  | Package              | File          | Command                          |
| --- | -------------------- | ------------- | -------------------------------- |
| 1   | **DMG**              | `eeg-lab.dmg` | Standard macOS installer         |
| 2   | **ZIP (App Bundle)** | `eeg-lab.zip` | Extract and drag to Applications |

### Windows

| No  | Package           | File          | Command                    |
| --- | ----------------- | ------------- | -------------------------- |
| 1   | **MSI Installer** | `eeg-lab.msi` | Standard Windows installer |
| 2   | **EXE**           | `eeg-lab.exe` | Portable executable        |

## Verification

All artifacts include SHA256 checksums in `SHA256SUMS.txt`:

```bash
# Verify all downloads
sha256sum --check SHA256SUMS.txt
```

## Installation Requirements

| Platform | Requirements                                       |
| -------- | -------------------------------------------------- |
| Linux    | WebKitGTK 4.1+ (Ubuntu 22.04+, Fedora 38+, Arch)   |
| macOS    | macOS 12+ (Monterey) — uses system WebKit          |
| Windows  | Windows 10 1903+ (WebView2 runtime auto-installed) |

## Support

- [Documentation](https://github.com/hieudoanm/eeg-lab/docs)
- [Issues](https://github.com/hieudoanm/eeg-lab/issues)
- [Security](https://github.com/hieudoanm/eeg-lab/SECURITY.md)
