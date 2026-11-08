export type ProjectKind = 'eeg' | 'meg' | 'mri';

export interface Project {
  id: string;
  name: string;
  kind: ProjectKind;
  description: string;
}

export interface Dataset {
  id: string;
  project_id: string;
  name: string;
  path: string;
  format: string;
  size_bytes: number;
  checksum_sha256: string;
  version: number;
}

export interface DatasetInfo {
  dataset: Dataset;
  metadata: Record<string, unknown>;
}

export interface Experiment {
  id: string;
  project_id: string;
  name: string;
  description: string;
  created_at: string;
}

export interface AnalysisRun {
  id: string;
  dataset_id: string;
  experiment_id: string | null;
  pipeline: string;
  parameters_json: string;
  status: string;
  result_json: string | null;
  error: string | null;
  progress?: number | null;
  progress_message?: string | null;
  created_at: string;
  completed_at: string | null;
}

export interface UploadSession {
  id: string;
  project_id: string;
  filename: string;
  total_bytes: number;
  received_bytes: number;
  status: string;
  checksum_sha256: string | null;
  expected_sha256?: string | null;
}

export interface Artifact {
  id: string;
  run_id: string;
  name: string;
  path: string;
  media_type: string;
  checksum_sha256: string;
  size_bytes: number;
}

export interface VolumeStatistics {
  voxel_count: number;
  valid_sample_count: number;
  min_value: number;
  max_value: number;
  mean: number;
  std_dev: number;
  non_finite_count: number;
}

export interface IntensityHistogram {
  bins: number[];
  counts: number[];
}

export interface EpochSummary {
  n_epochs: number;
  n_channels: number;
  tmin_s: number;
  tmax_s: number;
  skipped_events: number;
  event_codes: string[];
}

export interface MegPipelineResult {
  steps: string[];
  epochs: { event_sample: number; event_code: string | null; times_s: number[]; data: number[][] }[] | null;
  epoch_summary: EpochSummary | null;
  evoked: { n_epochs: number; channels: string[]; times_s: number[]; data: number[][] } | null;
  band_powers: Record<string, number> | null;
  bad_channels: string[];
  window: MegWindow | null;
  ica_components?: IcaComponentInfo[] | null;
}

export interface MegWindow {
  times_s: number[];
  data: number[][];
  effective_sampling_frequency_hz: number;
}

export interface EegStats {
  channel_name: string;
  mean: number;
  standard_deviation: number;
  minimum: number;
  maximum: number;
  valid_sample_count: number;
}

export interface MriSlice {
  plane: string;
  index: number;
  width: number;
  height: number;
  min_value: number;
  max_value: number;
  pixels: number[];
}

export interface IcaComponentInfo {
  index: number;
  variance: number;
  variance_fraction: number;
}

export interface IcaTopography {
  component: number;
  channel_weights: number[];
  grid?: number[] | null;
  grid_cols: number;
  grid_rows: number;
}

export interface EegPipelineResult {
  steps: string[];
  band_powers: Record<string, number> | null;
  bad_channels: string[];
  n_channels: number;
  n_samples: number;
  ica_components?: IcaComponentInfo[] | null;
  ica_topographies?: IcaTopography[] | null;
  n_epochs?: number;
  n_epochs_rejected?: number;
  erp?: number[][] | null;
}
