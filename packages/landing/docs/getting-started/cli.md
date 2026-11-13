# neuro CLI

The `neuro` CLI scripts the same operations as the desktop app. Output is JSON
on stdout; diagnostics go to stderr.

## Install

```bash
cargo install --path crates/neuro-cli
# or run without installing:
cargo run -p neuro-cli -- <cmd>
```

## Commands

```bash
neuro inspect-eeg recording.edf --pretty
neuro validate-meg /path/to/meg/ds
neuro convert-dicom scan.dcm -o volume.nii
neuro --help                            # all subcommands
```

`inspect-eeg` summarises an EEG recording, `validate-meg` checks a MEG
dataset directory, and `convert-dicom` exports a volume (e.g. NIfTI).
