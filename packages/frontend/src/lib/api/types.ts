/**
 * Domain types for the EEG Lab REST API (docs/07-api-spec.md).
 *
 * These mirror the server contract. JSON field names stay in `snake_case` so no
 * translation layer is required between the API and the UI.
 */

export type Uuid = string;

/** ISO 8601 timestamp with an explicit UTC offset, normally `Z`. */
export type IsoTimestamp = string;

export type MembershipRole = 'owner' | 'researcher' | 'viewer';

export type OrganisationStatus = 'active' | 'archived';

export type StudyStatus = 'active' | 'archived';

export type DatasetFormat = 'npy';

export type DatasetStatus =
  'pending_upload' | 'validating' | 'ready' | 'rejected' | 'failed' | 'archived';

export type UploadSessionStatus =
  'created' | 'uploading' | 'uploaded' | 'finalising' | 'completed' | 'failed' | 'expired';

export type UploadMethod = 'backend' | 'signed_url';

export type AnalysisType = 'descriptive_statistics';

export type AnalysisJobStatus = 'queued' | 'running' | 'succeeded' | 'failed' | 'cancelled';

export interface MembershipSummary {
  membership_id: Uuid;
  organisation_id: Uuid;
  organisation_name: string;
  role: MembershipRole;
}

export interface CurrentUser {
  id: Uuid;
  display_name: string | null;
  email: string | null;
  memberships: MembershipSummary[];
}

export interface OrganisationSummary {
  id: Uuid;
  name: string;
  slug: string | null;
  status: OrganisationStatus;
  role: MembershipRole;
}

export interface OrganisationDetail extends OrganisationSummary {
  created_at: IsoTimestamp;
  updated_at: IsoTimestamp;
}

export interface Membership {
  id: Uuid;
  user_id: Uuid;
  display_name: string | null;
  email: string | null;
  role: MembershipRole;
  created_at: IsoTimestamp;
  updated_at: IsoTimestamp;
}

export interface Study {
  id: Uuid;
  name: string;
  description: string | null;
  status: StudyStatus;
  created_at: IsoTimestamp;
  updated_at: IsoTimestamp;
  archived_at: IsoTimestamp | null;
}

export interface Dataset {
  id: Uuid;
  study_id: Uuid;
  name: string;
  description: string | null;
  format: DatasetFormat;
  status: DatasetStatus;
  sampling_frequency_hz: number | null;
  channel_count: number | null;
  sample_count: number | null;
  duration_seconds: number | null;
  channel_names: string[] | null;
  channel_units: string[] | null;
  checksum_sha256: string | null;
  file_size_bytes: number | null;
  source_description: string | null;
  created_at: IsoTimestamp;
  updated_at: IsoTimestamp;
  validated_at: IsoTimestamp | null;
}

export interface UploadManifest {
  sampling_frequency_hz: number;
  channel_names: string[];
  channel_units?: string[] | null;
}

export interface UploadSession {
  upload_session_id: Uuid;
  status: UploadSessionStatus;
  upload_method: UploadMethod;
  upload_url: string;
  expires_at: IsoTimestamp;
}

export interface UploadFinaliseResponse {
  upload_session_id: Uuid;
  upload_method: 'backend';
  status: UploadSessionStatus;
  dataset: Dataset;
}

export interface AnalysisJobError {
  code: string;
  message: string;
}

export interface AnalysisJob {
  id: Uuid;
  dataset_id: Uuid;
  analysis_type: AnalysisType;
  algorithm_version: string;
  status: AnalysisJobStatus;
  attempt_count: number;
  max_attempts: number;
  requested_at: IsoTimestamp;
  started_at: IsoTimestamp | null;
  completed_at: IsoTimestamp | null;
  error: AnalysisJobError | null;
}

export interface AnalysesByStatus {
  queued: number;
  running: number;
  succeeded: number;
  failed: number;
  cancelled: number;
}

export interface OrganisationDashboardSummary {
  study_count: number;
  active_study_count: number;
  dataset_count: number;
  ready_dataset_count: number;
  analysis_count: number;
  analyses_by_status: AnalysesByStatus;
}

export interface ChannelStatistics {
  channel_index: number;
  channel_name: string;
  unit: string | null;
  mean: number | null;
  standard_deviation: number | null;
  minimum: number | null;
  maximum: number | null;
  peak_to_peak: number | null;
  valid_sample_count: number;
}

export interface AnalysisResult {
  analysis_id: Uuid;
  dataset_id: Uuid;
  algorithm_version: string;
  result_schema_version: string;
  created_at: IsoTimestamp;
  channels: ChannelStatistics[];
  provenance: {
    input_checksum_sha256: string | null;
    sampling_frequency_hz: number | null;
    channel_count: number;
    sample_count: number;
    channel_names: string[];
    channel_units: (string | null)[];
    parameters: {
      include_mean: boolean;
      include_standard_deviation: boolean;
      include_min_max: boolean;
      include_peak_to_peak: boolean;
    };
    [key: string]: unknown;
  };
}

export interface Page<T> {
  items: T[];
  next_cursor: string | null;
  has_more: boolean;
}

export interface ApiErrorDetail {
  field: string;
  code: string;
  message: string;
}

export interface SignalWindowResponse {
  dataset_id: string;
  sampling_frequency_hz: number;
  channel_count: number;
  sample_count: number;
  duration_seconds: number;
  start_seconds: number;
  end_seconds: number;
  sample_start: number;
  sample_end: number;
  channel_indices: number[];
  points_per_channel: number;
  downsampled: boolean;
  downsample_factor: number;
  time_seconds: number[];
  channels: SignalChannel[];
}

export interface SignalChannel {
  channel_index: number;
  channel_name: string;
  unit: string | null;
  samples: (number | null)[];
}
