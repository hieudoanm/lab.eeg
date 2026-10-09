import { ApiClient, apiClient } from './client';
import type { DatasetFormat, UploadFinaliseResponse, UploadManifest, UploadSession } from './types';

export interface UploadSessionCreateBody {
  file_name: string;
  file_size_bytes: number;
  format?: DatasetFormat;
  expected_checksum_sha256?: string | null;
  manifest: UploadManifest;
}

/**
 * Create an upload session for a registered dataset.
 *
 * The session describes where and how to send the recording bytes; the
 * recording content itself is uploaded separately (docs/07-api-spec.md §10).
 */
export const createUploadSession = (
  organisationId: string,
  datasetId: string,
  body: UploadSessionCreateBody,
  client: ApiClient = apiClient,
): Promise<UploadSession> => {
  return client.post<UploadSession>(
    `/organisations/${organisationId}/datasets/${datasetId}/upload-sessions`,
    { body },
  );
};

/**
 * Finalise an upload session and validate the stored recording.
 *
 * Validation runs synchronously in Phase 5, so a `200` carries the dataset in
 * its terminal state. An invalid recording rejects with `422 invalid_recording`.
 */
export const finaliseUpload = (
  organisationId: string,
  datasetId: string,
  sessionId: string,
  client: ApiClient = apiClient,
): Promise<UploadFinaliseResponse> => {
  return client.post<UploadFinaliseResponse>(
    `/organisations/${organisationId}/datasets/${datasetId}/upload-sessions/${sessionId}/finalise`,
  );
};
