import { ApiClient, apiClient } from './client';
import type { AnalysisJob, AnalysisJobStatus, AnalysisResult, Page } from './types';

export interface AnalysisListOptions {
  cursor?: string;
  limit?: number;
}

export interface OrganisationAnalysisListOptions {
  cursor?: string;
  limit?: number;
  datasetId?: string;
  studyId?: string;
  status?: AnalysisJobStatus;
}

export interface AnalysisSubmitBody {
  analysis_type: 'descriptive_statistics';
  algorithm_version: string;
  parameters: {
    include_mean?: boolean;
    include_standard_deviation?: boolean;
    include_min_max?: boolean;
    include_peak_to_peak?: boolean;
  };
}

export const fetchAnalyses = (
  organisationId: string,
  datasetId: string,
  options: AnalysisListOptions = {},
  client: ApiClient = apiClient,
): Promise<Page<AnalysisJob>> => {
  return client.get<Page<AnalysisJob>>(
    `/organisations/${organisationId}/datasets/${datasetId}/analyses`,
    {
      query: { cursor: options.cursor, limit: options.limit },
    },
  );
};

export const fetchOrganisationAnalyses = (
  organisationId: string,
  options: OrganisationAnalysisListOptions = {},
  client: ApiClient = apiClient,
): Promise<Page<AnalysisJob>> => {
  return client.get<Page<AnalysisJob>>(`/organisations/${organisationId}/analyses`, {
    query: {
      cursor: options.cursor,
      limit: options.limit,
      dataset_id: options.datasetId,
      study_id: options.studyId,
      status: options.status,
    },
  });
};

export const fetchAnalysisJob = (
  organisationId: string,
  jobId: string,
  client: ApiClient = apiClient,
): Promise<AnalysisJob> => {
  return client.get<AnalysisJob>(`/organisations/${organisationId}/analyses/${jobId}`);
};

export const fetchAnalysisResult = (
  organisationId: string,
  jobId: string,
  client: ApiClient = apiClient,
): Promise<AnalysisResult> => {
  return client.get<AnalysisResult>(`/organisations/${organisationId}/analyses/${jobId}/results`);
};

export const submitAnalysis = (
  organisationId: string,
  datasetId: string,
  body: AnalysisSubmitBody,
  idempotencyKey: string,
  client: ApiClient = apiClient,
): Promise<AnalysisJob> => {
  return client.post<AnalysisJob>(
    `/organisations/${organisationId}/datasets/${datasetId}/analyses`,
    {
      body,
      headers: { 'Idempotency-Key': idempotencyKey },
    },
  );
};

export const cancelAnalysis = (
  organisationId: string,
  jobId: string,
  client: ApiClient = apiClient,
): Promise<AnalysisJob> => {
  return client.post<AnalysisJob>(`/organisations/${organisationId}/analyses/${jobId}/cancel`, {
    body: {},
  });
};
