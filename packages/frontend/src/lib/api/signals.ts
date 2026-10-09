import { ApiClient, apiClient } from './client';
import type { SignalWindowResponse } from './types';

export interface SignalWindowOptions {
  startSeconds: number;
  endSeconds: number;
  channels?: number[];
}

export const fetchSignalWindow = (
  organisationId: string,
  datasetId: string,
  options: SignalWindowOptions,
  client: ApiClient = apiClient,
): Promise<SignalWindowResponse> => {
  const query: Record<string, string> = {
    start_seconds: String(options.startSeconds),
    end_seconds: String(options.endSeconds),
  };
  if (options.channels && options.channels.length > 0) {
    query.channels = options.channels.join(',');
  }
  return client.get<SignalWindowResponse>(
    `/organisations/${organisationId}/datasets/${datasetId}/signals`,
    { query },
  );
};
