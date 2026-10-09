import type { StatusTone } from '@/components/atoms/status-badge';
import type { AnalysisJobStatus, DatasetStatus, StudyStatus } from '@/lib/api/types';

const DATASET_TONES: Record<DatasetStatus, StatusTone> = {
  pending_upload: 'neutral',
  validating: 'info',
  ready: 'success',
  rejected: 'danger',
  failed: 'danger',
  archived: 'neutral',
};

const DATASET_LABELS: Record<DatasetStatus, string> = {
  pending_upload: 'Pending upload',
  validating: 'Validating',
  ready: 'Ready',
  rejected: 'Rejected',
  failed: 'Failed',
  archived: 'Archived',
};

const JOB_TONES: Record<AnalysisJobStatus, StatusTone> = {
  queued: 'neutral',
  running: 'info',
  succeeded: 'success',
  failed: 'danger',
  cancelled: 'neutral',
};

const JOB_LABELS: Record<AnalysisJobStatus, string> = {
  queued: 'Queued',
  running: 'Running',
  succeeded: 'Succeeded',
  failed: 'Failed',
  cancelled: 'Cancelled',
};

const STUDY_TONES: Record<StudyStatus, StatusTone> = {
  active: 'success',
  archived: 'neutral',
};

const STUDY_LABELS: Record<StudyStatus, string> = {
  active: 'Active',
  archived: 'Archived',
};

export const datasetStatusTone = (status: DatasetStatus): StatusTone => {
  return DATASET_TONES[status];
};

export const datasetStatusLabel = (status: DatasetStatus): string => {
  return DATASET_LABELS[status];
};

export const analysisJobStatusTone = (status: AnalysisJobStatus): StatusTone => {
  return JOB_TONES[status];
};

export const analysisJobStatusLabel = (status: AnalysisJobStatus): string => {
  return JOB_LABELS[status];
};

export const studyStatusTone = (status: StudyStatus): StatusTone => {
  return STUDY_TONES[status];
};

export const studyStatusLabel = (status: StudyStatus): string => {
  return STUDY_LABELS[status];
};
