import { ApiClient, apiClient } from './client';
import type { Page, Study, StudyStatus } from './types';

export interface StudyArchiveResult {
  id: string;
  status: StudyStatus;
  archived_at: string;
}

export interface StudyListOptions {
  cursor?: string;
  search?: string;
  status?: StudyStatus;
  limit?: number;
}

export interface StudyCreateBody {
  name: string;
  description?: string | null;
}

export interface StudyUpdateBody {
  name?: string | null;
  description?: string | null;
}

export const fetchStudies = (
  organisationId: string,
  options: StudyListOptions = {},
  client: ApiClient = apiClient,
): Promise<Page<Study>> => {
  return client.get<Page<Study>>(`/organisations/${organisationId}/studies`, {
    query: {
      cursor: options.cursor,
      search: options.search,
      status: options.status,
      limit: options.limit,
    },
  });
};

export const fetchStudy = (
  organisationId: string,
  studyId: string,
  client: ApiClient = apiClient,
): Promise<Study> => {
  return client.get<Study>(`/organisations/${organisationId}/studies/${studyId}`);
};

export const createStudy = (
  organisationId: string,
  body: StudyCreateBody,
  client: ApiClient = apiClient,
): Promise<Study> => {
  return client.post<Study>(`/organisations/${organisationId}/studies`, { body });
};

export const updateStudy = (
  organisationId: string,
  studyId: string,
  body: StudyUpdateBody,
  client: ApiClient = apiClient,
): Promise<Study> => {
  return client.patch<Study>(`/organisations/${organisationId}/studies/${studyId}`, { body });
};

export const archiveStudy = (
  organisationId: string,
  studyId: string,
  client: ApiClient = apiClient,
): Promise<StudyArchiveResult> => {
  return client.post<StudyArchiveResult>(
    `/organisations/${organisationId}/studies/${studyId}/archive`,
  );
};
