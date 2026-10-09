/**
 * In-memory access-token holder used by the API client.
 *
 * Authentication is wired up in a later phase; this module provides a single
 * place for the client to read the current token without coupling to a specific
 * auth provider. The token is intentionally not persisted to `localStorage`
 * here — that decision belongs to the authentication phase.
 */

let accessToken: string | null = null;

export const setAccessToken = (token: string | null): void => {
  accessToken = token;
};

export const getAccessToken = (): string | null => {
  return accessToken;
};
