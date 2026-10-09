import { ApiClient, apiClient } from './client';
import type {
  CurrentUser,
  IsoTimestamp,
  Membership,
  OrganisationDetail,
  OrganisationSummary,
  Page,
} from './types';

export interface DevelopmentTokenRequest {
  subject?: string;
  identity_provider?: string;
  display_name?: string;
  email?: string;
}

export interface AccessTokenResponse {
  access_token: string;
  token_type: 'Bearer';
  expires_at: IsoTimestamp;
}

export const fetchCurrentUser = (client: ApiClient = apiClient): Promise<CurrentUser> => {
  return client.get<CurrentUser>('/me');
};

export const fetchOrganisations = (
  client: ApiClient = apiClient,
): Promise<Page<OrganisationSummary>> => {
  return client.get<Page<OrganisationSummary>>('/organisations');
};

export const fetchOrganisation = (
  organisationId: string,
  client: ApiClient = apiClient,
): Promise<OrganisationDetail> => {
  return client.get<OrganisationDetail>(`/organisations/${organisationId}`);
};

export const fetchOrganisationMembers = (
  organisationId: string,
  client: ApiClient = apiClient,
): Promise<Page<Membership>> => {
  return client.get<Page<Membership>>(`/organisations/${organisationId}/memberships`);
};

export const issueDevelopmentToken = (
  payload: DevelopmentTokenRequest = {},
  client: ApiClient = apiClient,
): Promise<AccessTokenResponse> => {
  return client.post<AccessTokenResponse>('/auth/token', { body: payload });
};

export const endSession = (client: ApiClient = apiClient): Promise<void> => {
  return client.post<void>('/auth/logout');
};
