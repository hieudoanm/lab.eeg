import { ApiClient, apiClient } from './client';
import type { Dataset, DatasetFormat, Page } from './types';

export interface DatasetListOptions {
  cursor?: string;
  limit?: number;
}

export interface DatasetCreateBody {
  name: string;
  description?: string | null;
  format?: DatasetFormat;
  source_description?: string | null;
  metadata?: Record<string, unknown>;
}

export const fetchDatasets = (
  organisationId: string,
  studyId: string,
  options: DatasetListOptions = {},
  client: ApiClient = apiClient,
): Promise<Page<Dataset>> => {
  return client.get<Page<Dataset>>(`/organisations/${organisationId}/studies/${studyId}/datasets`, {
    query: { cursor: options.cursor, limit: options.limit },
  });
};

export const fetchDataset = (
  organisationId: string,
  datasetId: string,
  client: ApiClient = apiClient,
): Promise<Dataset> => {
  return client.get<Dataset>(`/organisations/${organisationId}/datasets/${datasetId}`);
};

export const registerDataset = (
  organisationId: string,
  studyId: string,
  body: DatasetCreateBody,
  client: ApiClient = apiClient,
): Promise<Dataset> => {
  return client.post<Dataset>(`/organisations/${organisationId}/studies/${studyId}/datasets`, {
    body,
  });
};
