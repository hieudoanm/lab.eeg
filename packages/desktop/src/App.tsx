import { useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { formatResultJson, runProgressLabel } from './lib/format';
import type {
  AnalysisRun,
  Artifact,
  Dataset,
  DatasetInfo,
  EegStats,
  Experiment,
  IcaTopography,
  IntensityHistogram,
  EegPipelineResult,
  MegPipelineResult,
  MegWindow,
  MriSlice,
  Project,
  ProjectKind,
  UploadSession,
  VolumeStatistics,
} from './types';

const kinds: ProjectKind[] = ['eeg', 'meg', 'mri'];

export default function App() {
  const [kind, setKind] = useState<ProjectKind>('eeg');
  const [path, setPath] = useState('');
  const [samplingFrequency, setSamplingFrequency] = useState('250');
  const [projectName, setProjectName] = useState('Local study');
  const [projects, setProjects] = useState<Project[]>([]);
  const [projectId, setProjectId] = useState('');
  const [datasets, setDatasets] = useState<Dataset[]>([]);
  const [dataset, setDataset] = useState<DatasetInfo | null>(null);
  const [eegStats, setEegStats] = useState<EegStats[]>([]);
  const [eegWindow, setEegWindow] = useState<MegWindow | null>(null);
  const [eegStart, setEegStart] = useState('0');
  const [eegEnd, setEegEnd] = useState('1');
  const [eegReref, setEegReref] = useState<'average' | 'mastoid' | 'median' | 'robust_average' | 'car'>('average');
  const [eegIcaExclude, setEegIcaExclude] = useState('');
  const [eegIcaPositions, setEegIcaPositions] = useState('');
  const [megWindow, setMegWindow] = useState<MegWindow | null>(null);
  const [megRecording, setMegRecording] = useState('');
  const [megEvent, setMegEvent] = useState('');
  const [megCompGrade, setMegCompGrade] = useState('');
  const [megApplyProjectors, setMegApplyProjectors] = useState(false);
  const [megBidsDir, setMegBidsDir] = useState('');
  const [mriSlice, setMriSlice] = useState<MriSlice | null>(null);
  const [mriPlane, setMriPlane] = useState<'axial' | 'coronal' | 'sagittal'>('axial');
  const [mriStats, setMriStats] = useState<VolumeStatistics | null>(null);
  const [mipAxis, setMipAxis] = useState<'x' | 'y' | 'z'>('z');
  const [mriOperation, setMriOperation] = useState<'threshold' | 'gaussian_smooth' | 'segment_threshold' | 'resample' | 'apply_mask' | 'bias_field_correct' | 'register_rigid' | 'register_affine' | 'skull_strip'>('gaussian_smooth');
  const [maskPath, setMaskPath] = useState('');
  const [opParamA, setOpParamA] = useState('0');
  const [opParamB, setOpParamB] = useState('100');
  const [opFwhm, setOpFwhm] = useState('4');
  const [mriHistogram, setMriHistogram] = useState<IntensityHistogram | null>(null);
  const [mriSteps, setMriSteps] = useState<Array<{ operation: 'threshold' | 'gaussian_smooth' | 'segment_threshold' | 'resample' | 'apply_mask' | 'bias_field_correct'; p1: string; p2: string }>>([
    { operation: 'gaussian_smooth', p1: '4', p2: '' },
    { operation: 'segment_threshold', p1: '0', p2: '0' },
  ]);
  const [mriPresets, setMriPresets] = useState<Array<{ id: string; name: string; steps: unknown }>>([]);
  const [mriPresetName, setMriPresetName] = useState('');
  const [runs, setRuns] = useState<AnalysisRun[]>([]);
  const [datasetQuery, setDatasetQuery] = useState('');
  const [datasetSort, setDatasetSort] = useState('name');
  const [watch, setWatch] = useState<{ datasetId: string; runId: string; kind: 'eeg_pipeline' | 'meg_pipeline' | 'mri'; outputPath?: string } | null>(null);
  const [exportNote, setExportNote] = useState('');
  const [uploadProgress, setUploadProgress] = useState('');
  const [artifacts, setArtifacts] = useState<Artifact[]>([]);
  const [pipelineResult, setPipelineResult] = useState<MegPipelineResult | null>(null);
  const [eegPipelineResult, setEegPipelineResult] = useState<EegPipelineResult | null>(null);
  const [resampleMethod, setResampleMethod] = useState<'trilinear' | 'nearest' | 'cubic'>('trilinear');
  const [regMetric, setRegMetric] = useState<'ncc' | 'mi'>('ncc');
  const [experiments, setExperiments] = useState<Experiment[]>([]);
  const [experimentId, setExperimentId] = useState('');
  const [experimentName, setExperimentName] = useState('Experiment');
  const [sampleDir, setSampleDir] = useState('');
  const [megFileSet, setMegFileSet] = useState<{ format: string; findings: string[] } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const isTauri = useMemo(() => '__TAURI_INTERNALS__' in window, []);

  useEffect(() => {
    if (!isTauri) return;
    invoke<Project[]>('list_projects')
      .then((items) => {
        setProjects(items);
        setProjectId((current) => current || items[0]?.id || '');
      })
      .catch((cause) => setError(cause instanceof Error ? cause.message : String(cause)));
  }, [isTauri]);

  useEffect(() => {
    if (!isTauri || !projectId) {
      setDatasets([]);
      setExperiments([]);
      setExperimentId('');
      return;
    }
    invoke<Dataset[]>('list_datasets', { projectId, query: datasetQuery || null, sort: datasetSort })
      .then(setDatasets)
      .catch((cause) => setError(cause instanceof Error ? cause.message : String(cause)));
    invoke<Experiment[]>('list_experiments', { projectId })
      .then(setExperiments)
      .catch(() => setExperiments([]));
  }, [isTauri, projectId, datasetQuery, datasetSort]);

  const registeredDatasetId = useMemo(() => {
    if (!dataset) return null;
    return datasets.some((item) => item.id === dataset.dataset.id) ? dataset.dataset.id : null;
  }, [dataset, datasets]);

  useEffect(() => {
    if (!isTauri || !registeredDatasetId) {
      setRuns([]);
      setArtifacts([]);
      return;
    }
    invoke<AnalysisRun[]>('list_analysis_runs', { datasetId: registeredDatasetId })
      .then(async (items) => {
        setRuns(items);
        const artifactLists = await Promise.all(
          items.map((run) =>
            invoke<Artifact[]>('list_artifacts', { runId: run.id }).catch(() => [] as Artifact[]),
          ),
        );
        setArtifacts(artifactLists.flat());
      })
      .catch(() => setRuns([]));
  }, [isTauri, registeredDatasetId]);

  useEffect(() => {
    if (!isTauri) return;
    invoke<Array<{ id: string; name: string; modality: string; steps: unknown }>>('list_pipeline_presets')
      .then((presets) => setMriPresets(presets.filter((preset) => preset.modality === 'mri')))
      .catch(() => setMriPresets([]));
  }, [isTauri]);

  useEffect(() => {
    if (!isTauri || !watch) return;
    const timer = setInterval(() => {
      invoke<AnalysisRun[]>('list_analysis_runs', { datasetId: watch.datasetId })
        .then(async (items) => {
          setRuns(items);
          const run = items.find((item) => item.id === watch.runId);
          if (!run || run.status === 'running') return;
          setWatch(null);
          if (run.status === 'succeeded' && run.result_json) {
            const parsed: unknown = JSON.parse(run.result_json);
            if (watch.kind === 'eeg_pipeline') setEegPipelineResult(parsed as EegPipelineResult);
            else if (watch.kind === 'meg_pipeline') setPipelineResult(parsed as MegPipelineResult);
            if (watch.outputPath) setPath(watch.outputPath);
          }
          const runArtifacts = await invoke<Artifact[]>('list_artifacts', { runId: run.id }).catch(() => [] as Artifact[]);
          setArtifacts((items) => [...runArtifacts, ...items]);
        })
        .catch(() => undefined);
    }, 1000);
    return () => clearInterval(timer);
  }, [isTauri, watch]);

  async function watchRun(datasetId: string, runId: string, kind: 'eeg_pipeline' | 'meg_pipeline' | 'mri', outputPath?: string) {
    setWatch({ datasetId, runId, kind, outputPath });
  }

  function resetResults() {
    setEegStats([]);
    setEegWindow(null);
    setMegWindow(null);
    setMriSlice(null);
    setMriStats(null);
    setMriHistogram(null);
    setPipelineResult(null);
    setEegPipelineResult(null);
  }

  async function createProject() {
    if (!isTauri) return;
    setBusy(true);
    setError(null);
    try {
      const project = await invoke<Project>('create_project', {
        request: { name: projectName, kind, description: '' },
      });
      setProjects((items) => [...items, project]);
      setProjectId(project.id);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function createExperiment() {
    if (!isTauri || !projectId) return;
    setBusy(true);
    setError(null);
    try {
      const experiment = await invoke<Experiment>('create_experiment', {
        request: { project_id: projectId, name: experimentName, description: '' },
      });
      setExperiments((items) => [...items, experiment]);
      setExperimentId(experiment.id);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function cancelRun(runId: string) {
    if (!isTauri) return;
    setBusy(true);
    setError(null);
    try {
      const updated = await invoke<AnalysisRun>('cancel_analysis_run', { runId });
      setRuns((items) => items.map((run) => (run.id === updated.id ? updated : run)));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function inspect() {
    if (!path.trim()) return;
    setBusy(true);
    setError(null);
    resetResults();
    try {
      let effectiveProjectId = projectId || projects[0]?.id || '';
      if (isTauri && !effectiveProjectId) {
        const created = await invoke<Project>('create_project', {
          request: { name: `${kind.toUpperCase()} study`, kind, description: '' },
        });
        setProjects((items) => [...items, created]);
        setProjectId(created.id);
        effectiveProjectId = created.id;
      }
      const request = kind === 'eeg'
        ? { path: path.trim(), kind, sampling_frequency_hz: Number(samplingFrequency), project_id: effectiveProjectId || undefined }
        : { path: path.trim(), kind, project_id: effectiveProjectId || undefined };
      const result = await invoke<DatasetInfo>('inspect_dataset', { request });
      setDataset(result);
    } catch (cause) {
      setDataset(null);
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function registerDataset() {
    if (!dataset || !isTauri) return;
    setBusy(true);
    setError(null);
    try {
      const registered = await invoke<Dataset>('register_dataset', { dataset: dataset.dataset });
      setDatasets((items) => [...items, registered]);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function ensureRegistered(info: DatasetInfo): Promise<Dataset> {
    if (datasets.some((item) => item.id === info.dataset.id)) return info.dataset;
    const registered = await invoke<Dataset>('register_dataset', { dataset: info.dataset });
    setDatasets((items) => (items.some((item) => item.id === registered.id) ? items : [...items, registered]));
    return registered;
  }

  async function analyzeEeg() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const registered = await ensureRegistered(dataset);
      const result = await invoke<{
        run: AnalysisRun;
        artifact: Artifact;
        result: { channels: EegStats[] };
      }>('run_eeg_analysis', {
        request: {
          dataset_id: registered.id,
          path: registered.path,
          sampling_frequency_hz: Number(samplingFrequency),
          experiment_id: experimentId || undefined,
        },
      });
      setEegStats(result.result.channels);
      setRuns((items) => [result.run, ...items]);
      setArtifacts((items) => [result.artifact, ...items]);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function runEegPipeline(pipeline?: Record<string, unknown>) {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const registered = await ensureRegistered(dataset);
      const submitted = await invoke<{ run_id: string }>('submit_eeg_pipeline', {
        request: {
          dataset_id: registered.id,
          path: registered.path,
          experiment_id: experimentId || undefined,
          pipeline: pipeline ?? {
            nodes: [
              { type: 'bandpass', params: { l_freq_hz: 1, h_freq_hz: 40 } },
              { type: 'psd', params: { fmin_hz: 0.5, fmax_hz: 80, n_fft: 256 } },
              { type: 'bad_channels' },
            ],
          },
        },
      });
      await watchRun(registered.id, submitted.run_id, 'eeg_pipeline');
      setRuns((items) => [{ id: submitted.run_id, dataset_id: registered.id, experiment_id: experimentId || null, pipeline: 'eeg_pipeline', parameters_json: '', status: 'running', result_json: null, error: null, progress: 0, progress_message: null, created_at: new Date().toISOString(), completed_at: null }, ...items]);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function readEegWindow() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const shape = dataset.metadata.shape as number[] | undefined;
      const channelCount = shape?.[0] ?? 0;
      const channelIndices = Array.from({ length: Math.min(channelCount, 4) }, (_, index) => index);
      const result = await invoke<MegWindow>('get_eeg_signal_window', {
        request: {
          dataset_id: dataset.dataset.id,
          path: dataset.dataset.path,
          start_seconds: Number(eegStart),
          end_seconds: Number(eegEnd),
          channel_indices: channelIndices,
          sampling_frequency_hz: Number(samplingFrequency),
        },
      });
      setEegWindow(result);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function readMegWindow() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const metadata = dataset.metadata as { channels?: { name: string }[]; duration_seconds?: number };
      const channelNames = metadata.channels?.map((channel) => channel.name) ?? [];
      const result = await invoke<{ window: MegWindow }>('get_meg_window', {
        request: {
          dataset_id: dataset.dataset.id,
          path: dataset.dataset.path,
          start_s: 0,
          end_s: Math.min(metadata.duration_seconds ?? 1, 2),
          channel_names: channelNames.slice(0, 8),
          max_samples: 500,
          recording: megRecording || undefined,
          compensation_grade: megCompGrade.trim() ? Number.parseInt(megCompGrade, 10) : undefined,
          apply_projectors: megApplyProjectors,
        },
      });
      setMegWindow(result.window);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function readMriSlice(index: number, plane: 'axial' | 'coronal' | 'sagittal') {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<MriSlice>('get_mri_slice', {
        request: { dataset_id: dataset.dataset.id, path: dataset.dataset.path, index, plane },
      });
      setMriSlice(result);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function readMriStatistics() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<VolumeStatistics>('get_mri_statistics', {
        path: dataset.dataset.path,
      });
      setMriStats(result);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function readMriProjection() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<MriSlice>('get_mri_projection', {
        path: dataset.dataset.path,
        axis: mipAxis,
      });
      setMriSlice(result);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function convertDicom(series = false) {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<{ output: string }>(
        series ? 'convert_dicom_series_to_nifti' : 'convert_dicom_to_nifti',
        {
          request: { path: dataset.dataset.path },
        },
      );
      setPath(result.output);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function normalizeMriVolume() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<{ output: string }>('normalize_mri_volume', {
        path: dataset.dataset.path,
      });
      setPath(result.output);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function runMriOperation() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const registered = await ensureRegistered(dataset);
      const params: Record<string, unknown> =
        mriOperation === 'gaussian_smooth'
          ? { fwhm_mm: Number(opFwhm) }
          : mriOperation === 'bias_field_correct'
            ? { fwhm_mm: Number(opFwhm) || 60, iterations: Number(opParamA) || 2 }
          : mriOperation === 'resample'
            ? { spacing_mm: [Number(opParamA) || 1, Number(opParamB) || 1, 1], method: resampleMethod }
            : mriOperation === 'apply_mask'
              ? {}
              : (mriOperation === 'register_rigid' || mriOperation === 'register_affine')
                ? { moving_path: maskPath.trim(), max_evaluations: mriOperation === 'register_affine' ? 800 : 200, metric: regMetric }
                : mriOperation === 'skull_strip'
                  ? { erosion_mm: Number(opFwhm) || 4 }
                  : { low: Number(opParamA), high: Number(opParamB) };
      const stem = dataset.dataset.path.replace(/\.[^.]+$/, '');
      const suffix = mriOperation === 'gaussian_smooth' ? 'smooth' : mriOperation === 'resample' ? 'resampled' : mriOperation === 'segment_threshold' ? 'mask' : mriOperation === 'apply_mask' ? 'masked' : mriOperation === 'bias_field_correct' ? 'debias' : mriOperation === 'register_rigid' ? 'registered' : mriOperation === 'register_affine' ? 'affine' : mriOperation === 'skull_strip' ? 'brain' : 'threshold';
      const outputPath = `${stem}.${suffix}.nii.gz`;
      const submitted = await invoke<{ run_id: string }>('submit_mri_operation', {
        request: {
          dataset_id: registered.id,
          path: registered.path,
          operation: mriOperation,
          experiment_id: experimentId || undefined,
          params,
          mask_path: mriOperation === 'apply_mask' ? maskPath.trim() || undefined : undefined,
        },
      });
      await watchRun(registered.id, submitted.run_id, 'mri', outputPath);
      setRuns((items) => [{ id: submitted.run_id, dataset_id: registered.id, experiment_id: experimentId || null, pipeline: `mri_${mriOperation}`, parameters_json: '', status: 'running', result_json: null, error: null, progress: 0, progress_message: null, created_at: new Date().toISOString(), completed_at: null }, ...items]);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function runMriPipeline() {
    if (!dataset) return;
    const steps = mriSteps
      .filter((step) => step.operation)
      .map((step) => {
        const p1 = Number(step.p1);
        const p2 = Number(step.p2);
        const params: Record<string, unknown> =
          step.operation === 'gaussian_smooth'
            ? { fwhm_mm: Number.isFinite(p1) ? p1 : 4 }
            : step.operation === 'bias_field_correct'
              ? { fwhm_mm: Number.isFinite(p1) ? p1 : 60, iterations: Number.isFinite(p2) ? Math.round(p2) : 2 }
            : step.operation === 'resample'
              ? { spacing_mm: [Number.isFinite(p1) ? p1 : 1, Number.isFinite(p2) ? p2 : 1, 1], method: resampleMethod }
              : step.operation === 'apply_mask'
                ? {}
                : { low: Number.isFinite(p1) ? p1 : 0, high: Number.isFinite(p2) ? p2 : 0 };
        return {
          operation: step.operation,
          params,
          mask_path: step.operation === 'apply_mask' ? maskPath.trim() || undefined : undefined,
        };
      });
    if (steps.length === 0) {
      setError('Add at least one pipeline step.');
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const registered = await ensureRegistered(dataset);
      const submitted = await invoke<{ run_id: string }>('submit_mri_operation', {
        request: {
          dataset_id: registered.id,
          path: registered.path,
          operation: 'pipeline',
          experiment_id: experimentId || undefined,
          pipeline: steps,
        },
      });
      await watchRun(registered.id, submitted.run_id, 'mri');
      setRuns((items) => [{ id: submitted.run_id, dataset_id: registered.id, experiment_id: experimentId || null, pipeline: 'mri_pipeline', parameters_json: '', status: 'running', result_json: null, error: null, progress: 0, progress_message: null, created_at: new Date().toISOString(), completed_at: null }, ...items]);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function saveMriPreset() {
    const name = mriPresetName.trim();
    if (!name) {
      setError('Enter a preset name.');
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const steps = mriSteps
        .filter((step) => step.operation)
        .map((step) => ({ operation: step.operation, p1: step.p1, p2: step.p2 }));
      const preset = await invoke<{ id: string; name: string; modality: string; steps: unknown }>('save_pipeline_preset', {
        request: { name, modality: 'mri', steps },
      });
      setMriPresets((items) => [...items.filter((item) => item.id !== preset.id), preset]);
      setMriPresetName('');
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  function loadMriPreset(preset: { steps: unknown }) {
    const steps = preset.steps;
    if (!Array.isArray(steps)) return;
    const validOps = new Set(['threshold', 'gaussian_smooth', 'segment_threshold', 'resample', 'apply_mask', 'bias_field_correct']);
    const loaded = steps
      .filter((step): step is { operation: string; p1?: string; p2?: string } =>
        typeof step === 'object' && step !== null && validOps.has(String((step as { operation?: unknown }).operation)))
      .map((step) => ({
        operation: step.operation as typeof mriSteps[number]['operation'],
        p1: typeof step.p1 === 'string' ? step.p1 : '',
        p2: typeof step.p2 === 'string' ? step.p2 : '',
      }));
    if (loaded.length > 0) setMriSteps(loaded.slice(0, 8));
  }

  async function deleteMriPreset(presetId: string) {
    setBusy(true);
    setError(null);
    try {
      await invoke('delete_pipeline_preset', { presetId });
      setMriPresets((items) => items.filter((item) => item.id !== presetId));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function readMriHistogram() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<IntensityHistogram>('get_mri_histogram', {
        path: dataset.dataset.path,
        bins: 32,
      });
      setMriHistogram(result);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function validateMegFiles() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<{ format: string; findings: string[] }>('validate_meg_file_set', {
        path: dataset.dataset.path,
      });
      setMegFileSet(result);
    } catch (cause) {
      setMegFileSet(null);
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function validateEegFiles() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<{ format: string; findings: string[] }>('validate_eeg_file_set', {
        path: dataset.dataset.path,
      });
      setMegFileSet(result);
    } catch (cause) {
      setMegFileSet(null);
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function exportEeg() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    setExportNote('');
    try {
      const result = await invoke<{ output_path: string }>('export_eeg_edf', {
        request: {
          dataset_id: dataset.dataset.id,
          path: dataset.dataset.path,
          sampling_frequency_hz: Number(samplingFrequency),
        },
      });
      setExportNote(`Exported to ${result.output_path}`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function exportMeg() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    setExportNote('');
    try {
      const result = await invoke<{ output_path: string }>('export_meg_npy', {
        request: {
          path: dataset.dataset.path,
          channel_names: [],
          recording: megRecording || undefined,
          compensation_grade: megCompGrade.trim() ? Number.parseInt(megCompGrade, 10) : undefined,
          apply_projectors: megApplyProjectors,
        },
      });
      setExportNote(`Exported to ${result.output_path}`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function exportMegCsv() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    setExportNote('');
    try {
      const result = await invoke<{ output_path: string }>('export_meg_csv', {
        request: {
          path: dataset.dataset.path,
          channel_names: [],
          recording: megRecording || undefined,
          compensation_grade: megCompGrade.trim() ? Number.parseInt(megCompGrade, 10) : undefined,
          apply_projectors: megApplyProjectors,
        },
      });
      setExportNote(`Exported to ${result.output_path}`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function exportMegFif() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    setExportNote('');
    try {
      const result = await invoke<{ output_path: string }>('export_meg_fif', {
        request: {
          path: dataset.dataset.path,
          channel_names: [],
          recording: megRecording || undefined,
          compensation_grade: megCompGrade.trim() ? Number.parseInt(megCompGrade, 10) : undefined,
          apply_projectors: megApplyProjectors,
        },
      });
      setExportNote(`Exported to ${result.output_path}`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function exportMegEdf() {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    setExportNote('');
    try {
      const result = await invoke<{ output_path: string }>('export_meg_edf', {
        request: {
          path: dataset.dataset.path,
          channel_names: [],
          recording: megRecording || undefined,
          compensation_grade: megCompGrade.trim() ? Number.parseInt(megCompGrade, 10) : undefined,
          apply_projectors: megApplyProjectors,
        },
      });
      setExportNote(`Exported to ${result.output_path}`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function exportMegBids() {
    if (!dataset) return;
    const outputDir = megBidsDir.trim();
    if (!outputDir) {
      setError('Enter an output directory for the BIDS export.');
      return;
    }
    setBusy(true);
    setError(null);
    setExportNote('');
    try {
      const result = await invoke<{ output_dir: string }>('export_meg_bids', {
        request: {
          path: dataset.dataset.path,
          channel_names: [],
          output_dir: outputDir,
          recording: megRecording || undefined,
          compensation_grade: megCompGrade.trim() ? Number.parseInt(megCompGrade, 10) : undefined,
          apply_projectors: megApplyProjectors,
        },
      });
      setExportNote(`Exported BIDS dataset to ${result.output_dir}`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function exportArtifact(runId: string, name: string) {
    setBusy(true);
    setError(null);
    setExportNote('');
    try {
      const result = await invoke<{ output_path: string }>('export_run_artifact', { runId, artifactName: name });
      setExportNote(`Exported to ${result.output_path}`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function importFile(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    event.target.value = '';
    if (!file || !isTauri || !projectId) return;
    setBusy(true);
    setError(null);
    setUploadProgress('creating session…');
    try {
      // Resume an existing open session for the same file when possible.
      const sessions = await invoke<UploadSession[]>('list_upload_sessions', { projectId });
      const resumable = sessions.find(
        (session) =>
          session.status === 'open' &&
          session.filename === file.name &&
          session.total_bytes === file.size &&
          session.received_bytes > 0 &&
          session.received_bytes < file.size,
      );
      // Digest small-enough files up front so finalize can verify integrity.
      let expectedSha256: string | undefined;
      if (file.size <= 256 * 1024 * 1024 && crypto?.subtle) {
        const digest = await crypto.subtle.digest('SHA-256', await file.arrayBuffer());
        expectedSha256 = Array.from(new Uint8Array(digest))
          .map((byte) => byte.toString(16).padStart(2, '0'))
          .join('');
      }
      let sessionId: string;
      let offset = 0;
      if (resumable) {
        const report = await invoke<{
          size_matches: boolean;
          expected_match: boolean | null;
        }>('verify_upload_session', { sessionId: resumable.id });
        if (!report.size_matches || report.expected_match === false) {
          throw new Error('Resumable upload failed integrity verification; start a new upload.');
        }
        sessionId = resumable.id;
        offset = resumable.received_bytes;
        setUploadProgress(`resuming at ${Math.round((offset / file.size) * 100)}%…`);
      } else {
        const session = await invoke<{ id: string }>('create_upload_session', {
          request: {
            project_id: projectId,
            filename: file.name,
            total_bytes: file.size,
            expected_sha256: expectedSha256,
          },
        });
        sessionId = session.id;
      }
      const chunkSize = 4 * 1024 * 1024;
      for (; offset < file.size; offset += chunkSize) {
        const chunk = new Uint8Array(await file.slice(offset, offset + chunkSize).arrayBuffer());
        await invoke('append_upload_chunk', { sessionId, data: Array.from(chunk), offset });
        setUploadProgress(`${Math.round(((offset + chunk.length) / file.size) * 100)}%`);
      }
      const finalized = await invoke<{ dataset: Dataset }>('finalize_upload_session', { sessionId });
      setDatasets((items) => [...items, finalized.dataset]);
      setUploadProgress('');
    } catch (cause) {
      setUploadProgress('');
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function generateEegSample() {
    if (!sampleDir.trim()) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<{ path: string }>('generate_eeg_sample', {
        request: { directory: sampleDir.trim() },
      });
      setPath(result.path);
      setKind('eeg');
      setSamplingFrequency('250');
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function generateMegSample() {
    if (!sampleDir.trim()) return;
    setBusy(true);
    setError(null);
    try {
      await invoke('generate_meg_sample', {
        request: { directory: sampleDir.trim() },
      });
      setPath(sampleDir.trim());
      setKind('meg');
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function runMegPipeline(pipeline: Record<string, unknown>) {
    if (!dataset) return;
    setBusy(true);
    setError(null);
    try {
      const registered = await ensureRegistered(dataset);
      const metadata = dataset.metadata as { duration_seconds?: number };
      const submitted = await invoke<{ run_id: string }>('submit_meg_pipeline', {
        request: {
          dataset_id: registered.id,
          path: registered.path,
          start_s: 0,
          end_s: Math.min(metadata.duration_seconds ?? 1, 2),
          max_samples: 500,
          pipeline,
          experiment_id: experimentId || undefined,
          recording: megRecording || undefined,
          compensation_grade: megCompGrade.trim() ? Number.parseInt(megCompGrade, 10) : undefined,
          apply_projectors: megApplyProjectors,
        },
      });
      await watchRun(registered.id, submitted.run_id, 'meg_pipeline');
      setRuns((items) => [{ id: submitted.run_id, dataset_id: registered.id, experiment_id: experimentId || null, pipeline: 'meg_pipeline', parameters_json: '', status: 'running', result_json: null, error: null, progress: 0, progress_message: null, created_at: new Date().toISOString(), completed_at: null }, ...items]);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function runMegBatch() {
    const targets = datasets.filter((item) => item.format === 'meg');
    if (targets.length === 0 || !projectId) return;
    setBusy(true);
    setError(null);
    try {
      await invoke('submit_meg_batch', {
        request: {
          dataset_ids: targets.map((item) => item.id),
          start_s: 0,
          end_s: 2,
          max_samples: 500,
          pipeline: {
            nodes: [
              { type: 'filter', params: { l_freq_hz: 1, h_freq_hz: 40 } },
              { type: 'psd' },
              { type: 'bad_channels' },
            ],
          },
          experiment_id: experimentId || undefined,
        },
      });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="shell">
      <header className="masthead">
        <div>
          <p className="eyebrow">Local research workspace</p>
          <h1>Inspect and analyze neural data on this device</h1>
        </div>
        <div className="runtime">{isTauri ? 'Desktop runtime' : 'Browser preview'}</div>
      </header>

      <section className="workspace" aria-label="Workspace">
        <div className="project-bar">
          <label>
            Project
            <select value={projectId} onChange={(event) => setProjectId(event.target.value)}>
              <option value="">No project</option>
              {projects.map((project) => <option key={project.id} value={project.id}>{project.name}</option>)}
            </select>
          </label>
          <label>
            New project
            <input value={projectName} onChange={(event) => setProjectName(event.target.value)} />
          </label>
          <button className="secondary" disabled={busy || !isTauri} onClick={createProject}>Create</button>
          <label>
            Experiment
            <select value={experimentId} onChange={(event) => setExperimentId(event.target.value)}>
              <option value="">None</option>
              {experiments.map((experiment) => <option key={experiment.id} value={experiment.id}>{experiment.name}</option>)}
            </select>
          </label>
          <label>
            New experiment
            <input value={experimentName} onChange={(event) => setExperimentName(event.target.value)} />
          </label>
          <button className="secondary" disabled={busy || !isTauri || !projectId} onClick={createExperiment}>Create</button>
          <label>
            Import file
            <input type="file" disabled={busy || !isTauri || !projectId} onChange={importFile} />
          </label>
          {uploadProgress && <span className="upload-progress">{uploadProgress}</span>}
        </div>

        <div className="controls">
          <label>
            Modality
            <select value={kind} onChange={(event) => { setKind(event.target.value as ProjectKind); resetResults(); setDataset(null); }}>
              {kinds.map((value) => <option key={value} value={value}>{value.toUpperCase()}</option>)}
            </select>
          </label>
          <label className="path-field">
            Local path
            <input value={path} onChange={(event) => setPath(event.target.value)} placeholder={kind === 'meg' ? '/path/to/recording-directory' : '/path/to/file'} />
          </label>
          {kind === 'eeg' && (
            <label>
              Sampling frequency
              <input inputMode="decimal" value={samplingFrequency} onChange={(event) => setSamplingFrequency(event.target.value)} />
            </label>
          )}
          <button disabled={busy || !isTauri} onClick={inspect}>{busy ? 'Working…' : 'Inspect'}</button>
        </div>
        <div className="controls sample-controls">
          <label>
            Sample directory
            <input value={sampleDir} onChange={(event) => setSampleDir(event.target.value)} placeholder="/path/to/output" />
          </label>
          <button className="secondary" disabled={busy || !isTauri || !sampleDir.trim()} onClick={generateEegSample}>Generate EEG sample</button>
          <button className="secondary" disabled={busy || !isTauri || !sampleDir.trim()} onClick={generateMegSample}>Generate MEG sample</button>
        </div>
        {!isTauri && <p className="notice">Run <code>pnpm tauri:dev</code> to connect this interface to local Rust commands.</p>}
        {error && <p className="error" role="alert">{error}</p>}
        {exportNote && <p className="notice">{exportNote}</p>}

        {dataset && (
          <article className="result">
            <div className="result-heading">
              <div>
                <h2>{dataset.dataset.name}</h2>
                <p>{dataset.dataset.format} · v{dataset.dataset.version} · {(dataset.dataset.size_bytes / 1024).toFixed(1)} KiB</p>
              </div>
              <div className="result-actions">
                <code>{dataset.dataset.checksum_sha256.slice(0, 12)}</code>
                <button className="secondary" disabled={busy || !isTauri} onClick={registerDataset}>Register</button>
              </div>
            </div>
            <MetadataList metadata={dataset.metadata} />
          </article>
        )}

        {dataset?.dataset.format === 'eeg' && (
          <div className="analysis-actions">
            <button className="secondary" disabled={busy} onClick={analyzeEeg}>Run descriptive analysis</button>
            <button className="secondary" disabled={busy} onClick={() => runEegPipeline()}>Filter + PSD + QC</button>
            <button className="secondary" disabled={busy} onClick={() => runEegPipeline({
              nodes: [
                { type: 'epoch', params: { tmin_s: -0.2, tmax_s: 0.5, reject_ptp: 100 } },
              ],
            })}>Epoch + ERP</button>
            <label>
              Ref
              <select value={eegReref} onChange={(event) => setEegReref(event.target.value as typeof eegReref)}>
                <option value="average">Average</option>
                <option value="car">CAR</option>
                <option value="mastoid">Mastoids</option>
                <option value="median">Median</option>
                <option value="robust_average">Robust avg</option>
              </select>
            </label>
            <button className="secondary" disabled={busy} onClick={() => runEegPipeline({
              nodes: [
                { type: 'reref', params: { method: eegReref } },
                { type: 'epoch', params: { tmin_s: -0.2, tmax_s: 0.5, reject_ptp: 100 } },
              ],
            })}>Re-ref + epoch</button>
            <label>
              ICA exclude
              <input value={eegIcaExclude} onChange={(event) => setEegIcaExclude(event.target.value)} placeholder="0-based indices, e.g. 0,3" />
            </label>
            <button className="secondary" disabled={busy} onClick={() => {
              const components = eegIcaExclude.split(',').map((v) => Number.parseInt(v.trim(), 10)).filter((v) => Number.isInteger(v) && v >= 0);
              const nodes: Record<string, unknown>[] = [
                { type: 'bandpass', params: { l_freq_hz: 1, h_freq_hz: 40 } },
                { type: 'ica', params: {} },
              ];
              if (components.length > 0) nodes.push({ type: 'ica_remove', params: { components } });
              nodes.push({ type: 'epoch', params: { tmin_s: -0.2, tmax_s: 0.5, reject_ptp: 100 } });
              runEegPipeline({ nodes });
            }}>ICA clean + epoch</button>
            <label>
              ICA positions (JSON)
              <input value={eegIcaPositions} onChange={(event) => setEegIcaPositions(event.target.value)} placeholder="[[x,y,z],…] per channel" />
            </label>
            <button className="secondary" disabled={busy} onClick={() => {
              let positions: unknown;
              const raw = eegIcaPositions.trim();
              if (raw) {
                try {
                  positions = JSON.parse(raw);
                } catch {
                  setError('ICA positions must be valid JSON [[x,y,z],…].');
                  return;
                }
              }
              const components = eegIcaExclude.split(',').map((v) => Number.parseInt(v.trim(), 10)).filter((v) => Number.isInteger(v) && v >= 0);
              const nodes: Record<string, unknown>[] = [
                { type: 'bandpass', params: { l_freq_hz: 1, h_freq_hz: 40 } },
                { type: 'ica', params: { topographies: true, ...(positions !== undefined ? { positions } : {}) } },
              ];
              if (components.length > 0) nodes.push({ type: 'ica_remove', params: { components } });
              nodes.push({ type: 'epoch', params: { tmin_s: -0.2, tmax_s: 0.5, reject_ptp: 100 } });
              runEegPipeline({ nodes });
            }}>ICA + topographies</button>
            <label>Start<input inputMode="decimal" value={eegStart} onChange={(event) => setEegStart(event.target.value)} /></label>
            <label>End<input inputMode="decimal" value={eegEnd} onChange={(event) => setEegEnd(event.target.value)} /></label>
            <button className="secondary" disabled={busy} onClick={readEegWindow}>Load signal window</button>
            <button className="secondary" disabled={busy || !isTauri} onClick={exportEeg}>Export EDF</button>
            <button className="secondary" disabled={busy} onClick={validateEegFiles}>Validate file set</button>
          </div>
        )}
        {dataset?.dataset.format === 'meg' && (
          <div className="analysis-actions">
            <label>
              Recording
              <input value={megRecording} onChange={(event) => setMegRecording(event.target.value)} placeholder="index or name (BIDS)" />
            </label>
            <label>
              Event
              <input value={megEvent} onChange={(event) => setMegEvent(event.target.value)} placeholder="code substring (KIT/CTF)" />
            </label>
            <label>
              Comp grade
              <input value={megCompGrade} onChange={(event) => setMegCompGrade(event.target.value)} placeholder="CTF/FIF grade" inputMode="numeric" />
            </label>
            <label className="checkbox-label">
              <input type="checkbox" checked={megApplyProjectors} onChange={(event) => setMegApplyProjectors(event.target.checked)} />
              Apply projectors
            </label>
            <button className="secondary" disabled={busy} onClick={readMegWindow}>Read bounded window</button>
            <button className="secondary" disabled={busy} onClick={validateMegFiles}>Validate file set</button>
            <button className="secondary" disabled={busy || !isTauri} onClick={exportMeg}>Export NPY</button>
            <button className="secondary" disabled={busy || !isTauri} onClick={exportMegCsv}>Export CSV</button>
            <button className="secondary" disabled={busy || !isTauri} onClick={exportMegEdf}>Export EDF</button>
            <button className="secondary" disabled={busy || !isTauri} onClick={exportMegFif}>Export FIF</button>
            <label>
              BIDS dir
              <input value={megBidsDir} onChange={(event) => setMegBidsDir(event.target.value)} placeholder="/path/to/out" />
            </label>
            <button className="secondary" disabled={busy || !isTauri || !megBidsDir.trim()} onClick={exportMegBids}>Export BIDS</button>
            <button className="secondary" disabled={busy} onClick={() => runMegPipeline({
              nodes: [
                { type: 'filter', params: { l_freq_hz: 1, h_freq_hz: 40 } },
                { type: 'psd' },
                { type: 'bad_channels' },
              ],
            })}>Filter + PSD + QC</button>
            <button className="secondary" disabled={busy} onClick={() => runMegPipeline({
              nodes: [
                { type: 'filter', params: { l_freq_hz: 1, h_freq_hz: 30 } },
                { type: 'epoch', params: { tmin_s: -0.2, tmax_s: 0.5, ...(megEvent.trim() ? { event: megEvent.trim() } : {}) } },
                { type: 'evoked' },
              ],
            })}>Epoch + evoked</button>
            <button className="secondary" disabled={busy} onClick={() => runMegPipeline({
              nodes: [
                { type: 'synthetic_gradiometer', params: { min_distance_mm: 2, max_distance_mm: 30 } },
              ],
            })}>Synthetic grads</button>
            <button className="secondary" disabled={busy} onClick={() => runMegPipeline({
              nodes: [
                { type: 'filter', params: { l_freq_hz: 1, h_freq_hz: 40 } },
                { type: 'ssp', params: { ...(megEvent.trim() ? { event: megEvent.trim() } : {}), n_components: 1 } },
                { type: 'bad_channels' },
              ],
            })}>SSP clean + QC</button>
            <button className="secondary" disabled={busy || !isTauri || datasets.filter((item) => item.format === 'meg').length === 0} onClick={runMegBatch}>Run on all MEG datasets</button>
          </div>
        )}
        {dataset?.dataset.format === 'mri' && (
          <div className="analysis-actions">
            <MriControls busy={busy} plane={mriPlane} onPlane={setMriPlane} onSlice={(index) => readMriSlice(index, mriPlane)} />
            <label>
              Projection axis
              <select value={mipAxis} onChange={(event) => setMipAxis(event.target.value as 'x' | 'y' | 'z')}>
                <option value="x">X (sagittal)</option>
                <option value="y">Y (coronal)</option>
                <option value="z">Z (axial)</option>
              </select>
            </label>
            <button className="secondary" disabled={busy} onClick={readMriProjection}>Max-intensity projection</button>
            <button className="secondary" disabled={busy} onClick={readMriStatistics}>Volume statistics</button>
            <button className="secondary" disabled={busy} onClick={readMriHistogram}>Intensity histogram</button>
            <button className="secondary" disabled={busy} onClick={normalizeMriVolume}>Z-score normalize</button>
            <button className="secondary" disabled={busy || !isTauri} onClick={() => convertDicom()}>Convert DICOM</button>
            <button className="secondary" disabled={busy || !isTauri} onClick={() => convertDicom(true)}>Convert DICOM series</button>
            <label>
              Operation
              <select value={mriOperation} onChange={(event) => setMriOperation(event.target.value as typeof mriOperation)}>
                <option value="gaussian_smooth">Gaussian smooth</option>
                <option value="threshold">Threshold</option>
                <option value="segment_threshold">Segment</option>
                <option value="resample">Resample</option>
                <option value="apply_mask">Apply mask</option>
                <option value="bias_field_correct">Bias-field correct</option>
                <option value="register_rigid">Register (rigid)</option>
                <option value="register_affine">Register (affine)</option>
                <option value="skull_strip">Skull strip</option>
              </select>
            </label>
            {(mriOperation === 'apply_mask' || mriOperation === 'register_rigid' || mriOperation === 'register_affine') && (
              <label>{mriOperation === 'apply_mask' ? 'Mask path' : 'Moving volume path'}<input value={maskPath} onChange={(event) => setMaskPath(event.target.value)} placeholder={mriOperation === 'apply_mask' ? '/path/to/mask.nii.gz' : '/path/to/moving.nii.gz'} /></label>
            )}
            {(mriOperation === 'register_rigid' || mriOperation === 'register_affine') && (
              <label>Metric
                <select value={regMetric} onChange={(event) => setRegMetric(event.target.value as 'ncc' | 'mi')}>
                  <option value="ncc">Correlation (NCC)</option>
                  <option value="mi">Mutual information</option>
                </select>
              </label>
            )}
            {(mriOperation === 'gaussian_smooth' || mriOperation === 'bias_field_correct' || mriOperation === 'skull_strip') && (
              <label>{mriOperation === 'skull_strip' ? 'Erosion (mm)' : 'FWHM (mm)'}<input inputMode="decimal" value={opFwhm} onChange={(event) => setOpFwhm(event.target.value)} /></label>
            )}
            {mriOperation === 'resample' && (
              <label>Interpolation
                <select value={resampleMethod} onChange={(event) => setResampleMethod(event.target.value as 'trilinear' | 'nearest' | 'cubic')}>
                  <option value="trilinear">Trilinear</option>
                  <option value="nearest">Nearest neighbour</option>
                  <option value="cubic">Cubic spline</option>
                </select>
              </label>
            )}
            {mriOperation === 'bias_field_correct' && (
              <label>Iterations<input inputMode="numeric" value={opParamA} onChange={(event) => setOpParamA(event.target.value)} /></label>
            )}
            {mriOperation !== 'gaussian_smooth' && mriOperation !== 'apply_mask' && mriOperation !== 'bias_field_correct' && mriOperation !== 'register_rigid' && mriOperation !== 'register_affine' && (
              <>
                <label>{mriOperation === 'resample' ? 'Spacing X (mm)' : 'Low'}<input inputMode="decimal" value={opParamA} onChange={(event) => setOpParamA(event.target.value)} /></label>
                <label>{mriOperation === 'resample' ? 'Spacing Y (mm)' : 'High'}<input inputMode="decimal" value={opParamB} onChange={(event) => setOpParamB(event.target.value)} /></label>
              </>
            )}
            <button className="secondary" disabled={busy} onClick={runMriOperation}>Run operation</button>
            <div className="pipeline-builder">
              <span className="pipeline-builder-title">Chain</span>
              {mriSteps.map((step, index) => (
                <div className="pipeline-step" key={index}>
                  <select
                    value={step.operation}
                    onChange={(event) => setMriSteps((items) => items.map((item, i) => i === index ? { ...item, operation: event.target.value as typeof step.operation } : item))}
                  >
                    <option value="gaussian_smooth">Smooth</option>
                    <option value="threshold">Threshold</option>
                    <option value="segment_threshold">Segment</option>
                    <option value="resample">Resample</option>
                    <option value="apply_mask">Mask</option>
                    <option value="bias_field_correct">Debias</option>
                  </select>
                  {step.operation === 'gaussian_smooth' && (
                    <input inputMode="decimal" value={step.p1} placeholder="FWHM mm" onChange={(event) => setMriSteps((items) => items.map((item, i) => i === index ? { ...item, p1: event.target.value } : item))} />
                  )}
                  {step.operation === 'resample' && (
                    <>
                      <input inputMode="decimal" value={step.p1} placeholder="Spacing X" onChange={(event) => setMriSteps((items) => items.map((item, i) => i === index ? { ...item, p1: event.target.value } : item))} />
                      <input inputMode="decimal" value={step.p2} placeholder="Spacing Y" onChange={(event) => setMriSteps((items) => items.map((item, i) => i === index ? { ...item, p2: event.target.value } : item))} />
                    </>
                  )}
                  {step.operation === 'bias_field_correct' && (
                    <>
                      <input inputMode="decimal" value={step.p1} placeholder="FWHM mm" onChange={(event) => setMriSteps((items) => items.map((item, i) => i === index ? { ...item, p1: event.target.value } : item))} />
                      <input inputMode="numeric" value={step.p2} placeholder="Iterations" onChange={(event) => setMriSteps((items) => items.map((item, i) => i === index ? { ...item, p2: event.target.value } : item))} />
                    </>
                  )}
                  {(step.operation === 'threshold' || step.operation === 'segment_threshold') && (
                    <>
                      <input inputMode="decimal" value={step.p1} placeholder="Low" onChange={(event) => setMriSteps((items) => items.map((item, i) => i === index ? { ...item, p1: event.target.value } : item))} />
                      <input inputMode="decimal" value={step.p2} placeholder="High" onChange={(event) => setMriSteps((items) => items.map((item, i) => i === index ? { ...item, p2: event.target.value } : item))} />
                    </>
                  )}
                  {mriSteps.some((item) => item.operation === 'apply_mask') && step.operation === 'apply_mask' && (
                    <input value={maskPath} placeholder="mask path" onChange={(event) => setMaskPath(event.target.value)} />
                  )}
                  <button
                    className="secondary"
                    disabled={mriSteps.length <= 1}
                    onClick={() => setMriSteps((items) => items.filter((_, i) => i !== index))}
                  >×</button>
                </div>
              ))}
              <div className="pipeline-step-actions">
                <button
                  className="secondary"
                  disabled={mriSteps.length >= 8}
                  onClick={() => setMriSteps((items) => [...items, { operation: 'gaussian_smooth', p1: '4', p2: '' }])}
                >+ step</button>
                <button className="secondary" disabled={busy} onClick={runMriPipeline}>Run chain</button>
              </div>
              <div className="pipeline-step-actions">
                <input
                  value={mriPresetName}
                  placeholder="Preset name"
                  onChange={(event) => setMriPresetName(event.target.value)}
                />
                <button className="secondary" disabled={busy || !mriPresetName.trim()} onClick={saveMriPreset}>Save preset</button>
                {mriPresets.map((preset) => (
                  <span key={preset.id} className="preset-chip">
                    <button className="secondary" disabled={busy} onClick={() => loadMriPreset(preset)}>{preset.name}</button>
                    <button className="secondary" disabled={busy} onClick={() => deleteMriPreset(preset.id)}>×</button>
                  </span>
                ))}
              </div>
            </div>
          </div>
        )}
        {eegStats.length > 0 && <StatsTable stats={eegStats} />}
        {eegWindow && <MegPreview window={eegWindow} title="EEG signal window" />}
        {megWindow && <MegPreview window={megWindow} title="MEG signal window" />}
        {megFileSet && (
          <article className="result">
            <h2>File set: {megFileSet.format}</h2>
            {megFileSet.findings.length === 0 ? (
              <p>All required files present.</p>
            ) : (
              <ul>{megFileSet.findings.map((finding) => <li key={finding}>{finding}</li>)}</ul>
            )}
          </article>
        )}
        {pipelineResult && <PipelineResultView result={pipelineResult} />}
        {eegPipelineResult && <EegPipelineResultView result={eegPipelineResult} />}
        {mriSlice && <MriPreview slice={mriSlice} />}
        {mriStats && <MriStatsView stats={mriStats} />}
        {mriHistogram && <HistogramView histogram={mriHistogram} />}
        {runs.length > 0 && <RunsView runs={runs} artifacts={artifacts} onCancel={cancelRun} onExport={exportArtifact} busy={busy} />}

        {datasets.length > 0 && (
          <section className="dataset-list" aria-label="Registered datasets">
            <div className="dataset-controls">
              <h2>Registered datasets</h2>
              <input
                value={datasetQuery}
                onChange={(event) => setDatasetQuery(event.target.value)}
                placeholder="Search name, path, format"
              />
              <select value={datasetSort} onChange={(event) => setDatasetSort(event.target.value)}>
                <option value="name">Name</option>
                <option value="newest">Newest</option>
                <option value="version">Version</option>
              </select>
            </div>
            {datasets.map((item) => (
              <button key={item.id} className="dataset-row" onClick={() => setDataset({ dataset: item, metadata: {} })}>
                <span>{item.name}</span>
                <span>{item.format} v{item.version}</span>
                <code>{item.checksum_sha256.slice(0, 10)}</code>
              </button>
            ))}
          </section>
        )}
      </section>
    </main>
  );
}

function MetadataList({ metadata }: { metadata: Record<string, unknown> }) {
  const keys = ['shape', 'sampling_frequency_hz', 'duration_seconds', 'channel_count', 'spacing_mm', 'voxel_count'];
  const entries = Object.entries(metadata).filter(([key]) => keys.includes(key)).slice(0, 8);
  return (
    <dl>
      {entries.map(([key, value]) => (
        <div key={key}>
          <dt>{key.replaceAll('_', ' ')}</dt>
          <dd>{Array.isArray(value) ? value.join(' × ') : String(value)}</dd>
        </div>
      ))}
    </dl>
  );
}

function StatsTable({ stats }: { stats: EegStats[] }) {
  return (
    <div className="table-wrap">
      <table>
        <thead><tr><th>Channel</th><th>Mean</th><th>Std dev</th><th>Range</th><th>Valid samples</th></tr></thead>
        <tbody>
          {stats.map((stat) => (
            <tr key={stat.channel_name}>
              <td>{stat.channel_name}</td>
              <td>{stat.mean.toFixed(4)}</td>
              <td>{stat.standard_deviation.toFixed(4)}</td>
              <td>{stat.minimum.toFixed(2)} – {stat.maximum.toFixed(2)}</td>
              <td>{stat.valid_sample_count}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function IcaTopoGrid({ topo }: { topo: IcaTopography }) {
  const cols = topo.grid_cols || 72;
  const rows = topo.grid_rows || 36;
  const grid = topo.grid ?? [];
  if (grid.length === 0) return null;
  // Downsample for SVG: keep every 2nd cell in each dimension.
  const step = 2;
  const w = Math.ceil(cols / step);
  const h = Math.ceil(rows / step);
  const maxAbs = Math.max(...grid.map((v) => Math.abs(v)), 1e-9);
  const rects: React.ReactNode[] = [];
  for (let r = 0; r < rows; r += step) {
    for (let c = 0; c < cols; c += step) {
      const v = grid[r * cols + c] ?? 0;
      const t = Math.min(1, Math.abs(v) / maxAbs);
      const fill = v >= 0 ? `rgba(220, 60, 60, ${0.15 + 0.85 * t})` : `rgba(60, 90, 220, ${0.15 + 0.85 * t})`;
      rects.push(<rect key={`${r}-${c}`} x={c / step} y={r / step} width={1.02} height={1.02} fill={fill} />);
    }
  }
  return (
    <svg viewBox={`0 0 ${w} ${h}`} width={w * 6} height={h * 6} role="img" aria-label={`ICA component ${topo.component} topography`}>
      {rects}
    </svg>
  );
}

function MegPreview({ window, title = 'Signal window' }: { window: MegWindow; title?: string }) {
  const max = Math.max(...window.data.flat().map(Math.abs), 1);
  return (
    <article className="result">
      <h2>{title}</h2>
      <p>{window.times_s.length} samples at {window.effective_sampling_frequency_hz.toFixed(1)} Hz</p>
      <div className="waveforms">
        {window.data.map((channel, index) => (
          <svg key={index} viewBox="0 0 600 80" role="img" aria-label={`Channel ${index + 1}`}>
            <path d={channel.map((value, point) => `${point === 0 ? 'M' : 'L'}${(point / Math.max(channel.length - 1, 1)) * 600},${40 - (value / max) * 34}`).join(' ')} fill="none" stroke="#3048a8" strokeWidth="1.5" />
          </svg>
        ))}
      </div>
    </article>
  );
}

function EegPipelineResultView({ result }: { result: EegPipelineResult }) {
  return (
    <article className="pipeline-result">
      <h2>EEG pipeline</h2>
      <p>{result.n_channels} channels · {result.n_samples} samples</p>
      <ul>{result.steps.map((step) => <li key={step}><code>{step}</code></li>)}</ul>
      {result.bad_channels.length > 0 && (
        <p className="error">Flagged channels: {result.bad_channels.join(', ')}</p>
      )}
      {result.band_powers && (
        <table>
          <thead><tr><th>Band</th><th>Mean power</th></tr></thead>
          <tbody>
            {Object.entries(result.band_powers).map(([band, power]) => (
              <tr key={band}><td>{band}</td><td>{power.toExponential(3)}</td></tr>
            ))}
          </tbody>
        </table>
      )}
      {result.ica_components && result.ica_components.length > 0 && (
        <table>
          <thead><tr><th>ICA component</th><th>Variance</th><th>Share</th></tr></thead>
          <tbody>
            {result.ica_components.map((component) => (
              <tr key={component.index}>
                <td>{component.index}</td>
                <td>{component.variance.toExponential(3)}</td>
                <td>{(component.variance_fraction * 100).toFixed(1)}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {result.ica_topographies && result.ica_topographies.length > 0 && (
        <div className="ica-topos">
          {result.ica_topographies.map((topo) => (
            <figure key={topo.component}>
              <figcaption>Component {topo.component}</figcaption>
              {topo.grid && topo.grid.length > 0 ? (
                <IcaTopoGrid topo={topo} />
              ) : (
                <p className="muted">
                  Weights: {topo.channel_weights.slice(0, 8).map((w) => w.toFixed(3)).join(', ')}
                  {topo.channel_weights.length > 8 ? ' …' : ''}
                </p>
              )}
            </figure>
          ))}
        </div>
      )}
      {(result.n_epochs !== undefined || result.erp) && (
        <p>
          Epochs: {result.n_epochs ?? 0} kept{result.n_epochs_rejected ? `, ${result.n_epochs_rejected} rejected` : ''}
          {result.erp ? ' · ERP computed' : ''}
        </p>
      )}
    </article>
  );
}

function PipelineResultView({ result }: { result: MegPipelineResult }) {
  return (
    <article className="result">
      <h2>Pipeline result</h2>
      <p>Steps: {result.steps.join(' → ') || 'none'}</p>
      {result.band_powers && (
        <dl>
          {Object.entries(result.band_powers).map(([band, power]) => (
            <div key={band}>
              <dt>{band}</dt>
              <dd>{power.toFixed(4)}</dd>
            </div>
          ))}
        </dl>
      )}
      {result.epoch_summary && (
        <p>
          Epochs: {result.epoch_summary.n_epochs} kept, {result.epoch_summary.skipped_events} skipped
          {result.epoch_summary.event_codes.length > 0 &&
            ` · events: ${result.epoch_summary.event_codes.join(', ')}`}
        </p>
      )}
      {result.ica_components && result.ica_components.length > 0 && (
        <table>
          <thead><tr><th>ICA component</th><th>Variance</th><th>Share</th></tr></thead>
          <tbody>
            {result.ica_components.map((component) => (
              <tr key={component.index}>
                <td>{component.index}</td>
                <td>{component.variance.toExponential(3)}</td>
                <td>{(component.variance_fraction * 100).toFixed(1)}%</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {result.evoked && (
        <p>
          Evoked response: {result.evoked.n_epochs} epochs averaged across{' '}
          {result.evoked.channels.length} channels
        </p>
      )}
      {result.bad_channels.length > 0 && (
        <p>Flagged channels: {result.bad_channels.join(', ')}</p>
      )}
      {result.window && <MegPreview window={result.window} title="Filtered window" />}
    </article>
  );
}

function MriStatsView({ stats }: { stats: VolumeStatistics }) {
  return (
    <article className="result">
      <h2>Volume statistics</h2>
      <dl>
        <div><dt>Voxels</dt><dd>{stats.voxel_count}</dd></div>
        <div><dt>Valid</dt><dd>{stats.valid_sample_count}</dd></div>
        <div><dt>Mean</dt><dd>{stats.mean.toFixed(4)}</dd></div>
        <div><dt>Std dev</dt><dd>{stats.std_dev.toFixed(4)}</dd></div>
        <div><dt>Range</dt><dd>{stats.min_value.toFixed(2)} – {stats.max_value.toFixed(2)}</dd></div>
        <div><dt>Non-finite</dt><dd>{stats.non_finite_count}</dd></div>
      </dl>
    </article>
  );
}

function HistogramView({ histogram }: { histogram: IntensityHistogram }) {
  const max = Math.max(...histogram.counts, 1);
  return (
    <article className="result">
      <h2>Intensity histogram</h2>
      <div className="histogram" role="img" aria-label="Intensity histogram">
        {histogram.counts.map((count, index) => (
          <div
            key={index}
            className="histogram-bar"
            style={{ height: `${(count / max) * 100}%` }}
            title={`${histogram.bins[index]?.toFixed(2)}: ${count}`}
          />
        ))}
      </div>
    </article>
  );
}

function RunsView({ runs, artifacts, onCancel, onExport, busy }: { runs: AnalysisRun[]; artifacts: Artifact[]; onCancel: (runId: string) => void; onExport: (runId: string, name: string) => void; busy: boolean }) {
  const artifactByRun = new Map<string, Artifact[]>();
  for (const artifact of artifacts) {
    const list = artifactByRun.get(artifact.run_id) ?? [];
    list.push(artifact);
    artifactByRun.set(artifact.run_id, list);
  }
  return (
    <section className="dataset-list" aria-label="Analysis runs">
      <h2>Analysis runs</h2>
      {runs.map((run) => (
        <article key={run.id} className="run-row">
          <div>
            <strong>{run.pipeline}</strong>
            <span className={`run-status run-${run.status}`}>{run.status}</span>
            {run.status === 'running' && run.progress != null && (
              <span className="run-progress">{runProgressLabel(run)}</span>
            )}
            {run.status === 'running' && (
              <button className="secondary" disabled={busy} onClick={() => onCancel(run.id)}>Cancel</button>
            )}
          </div>
          <p>{new Date(run.created_at).toLocaleString()}</p>
          {run.error && <p className="error">{run.error}</p>}
          {run.result_json && (
            <details className="run-result">
              <summary>Result JSON</summary>
              <pre>{formatResultJson(run.result_json)}</pre>
            </details>
          )}
          {(artifactByRun.get(run.id) ?? []).map((artifact) => (
            <p key={artifact.id}>
              <code>{artifact.name}</code> · {(artifact.size_bytes / 1024).toFixed(1)} KiB ·{' '}
              {artifact.checksum_sha256.slice(0, 12)}{' '}
              <button className="secondary" disabled={busy} onClick={() => onExport(run.id, artifact.name)}>Export</button>
            </p>
          ))}
        </article>
      ))}
    </section>
  );
}

function MriControls({ busy, plane, onPlane, onSlice }: { busy: boolean; plane: 'axial' | 'coronal' | 'sagittal'; onPlane: (plane: 'axial' | 'coronal' | 'sagittal') => void; onSlice: (index: number) => void }) {
  const [index, setIndex] = useState(0);
  return (
    <div className="slice-controls">
      <label>
        Plane
        <select value={plane} onChange={(event) => onPlane(event.target.value as 'axial' | 'coronal' | 'sagittal')}>
          <option value="axial">Axial</option>
          <option value="coronal">Coronal</option>
          <option value="sagittal">Sagittal</option>
        </select>
      </label>
      <label>Slice index<input inputMode="numeric" value={index} onChange={(event) => setIndex(Number(event.target.value))} /></label>
      <button className="secondary" disabled={busy} onClick={() => onSlice(index)}>Load slice</button>
    </div>
  );
}

function MriPreview({ slice }: { slice: MriSlice }) {
  const range = slice.max_value - slice.min_value || 1;
  return (
    <article className="result">
      <h2>{slice.plane} slice {slice.index}</h2>
      <p>{slice.width} × {slice.height} · {slice.min_value.toFixed(2)} to {slice.max_value.toFixed(2)}</p>
      <canvas
        width={slice.width}
        height={slice.height}
        style={{ width: '100%', imageRendering: 'pixelated', background: '#111827' }}
        ref={(canvas) => {
          if (!canvas) return;
          const context = canvas.getContext('2d');
          if (!context) return;
          const image = context.createImageData(slice.width, slice.height);
          slice.pixels.forEach((value, index) => {
            const normalized = Math.max(0, Math.min(1, (value - slice.min_value) / range));
            const shade = Math.round(normalized * 255);
            image.data[index * 4] = shade;
            image.data[index * 4 + 1] = shade;
            image.data[index * 4 + 2] = shade;
            image.data[index * 4 + 3] = 255;
          });
          context.putImageData(image, 0, 0);
        }}
      />
    </article>
  );
}
