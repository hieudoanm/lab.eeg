import { ApiClient, apiClient } from './client';
import type { OrganisationDashboardSummary } from './types';

export const fetchOrganisationSummary = (
  organisationId: string,
  client: ApiClient = apiClient,
): Promise<OrganisationDashboardSummary> => {
  return client.get<OrganisationDashboardSummary>(`/organisations/${organisationId}/summary`);
};
