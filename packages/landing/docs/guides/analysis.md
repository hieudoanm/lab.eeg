# Running an analysis

1. **Import data** — open a project and import an EEG (`.edf`, NPY,
   BrainVision), MEG, or MRI (DICOM) recording.
2. **Inspect and validate** — check channels, sampling rate, and annotations
   before processing. EEG supports EDF+ annotations and `.vmrk` markers as
   events.
3. **Build a pipeline** — chain steps such as filtering, bad-channel detection,
   re-referencing (average/CAR/mastoid/median/robust/Laplacian), baseline
   correction, event-locked epoching with ERP, artifact rejection, and FastICA.
4. **Run** — start the analysis; progress is reported in the UI and the run
   can be cancelled.
5. **Export** — save artifacts or export results from the workspace or the
   `neuro` CLI.

See [pipelines](../concepts/pipelines.md) for how chains are represented and
[datasets and provenance](../concepts/datasets-and-provenance.md) for versioning.
