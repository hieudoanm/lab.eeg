import { resolveApiBaseUrl } from '@/lib/api/client';
import { getSessionToken } from '@/lib/session';

const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

const errorResponse = (status: number, code: string, message: string): Response => {
  return Response.json({ error: { code, message } }, { status });
};

/**
 * Proxy recording uploads to the API.
 *
 * The session cookie is HttpOnly, so the browser cannot authenticate a direct
 * upload to the API. This handler streams the request body to the authenticated
 * `PUT .../content` endpoint on behalf of the session. Tenant identifiers are
 * validated as UUIDs before being interpolated into the upstream URL, so a
 * crafted value cannot redirect the request to another resource.
 */
export const PUT = async (request: Request): Promise<Response> => {
  const { searchParams } = new URL(request.url);
  const organisationId = searchParams.get('organisationId') ?? '';
  const datasetId = searchParams.get('datasetId') ?? '';
  const sessionId = searchParams.get('sessionId') ?? '';

  if (
    !UUID_PATTERN.test(organisationId) ||
    !UUID_PATTERN.test(datasetId) ||
    !UUID_PATTERN.test(sessionId)
  ) {
    return errorResponse(400, 'invalid_request', 'The upload target is invalid.');
  }

  const token = await getSessionToken();
  if (!token) {
    return errorResponse(401, 'unauthenticated', 'Sign in to upload recordings.');
  }

  const baseUrl = resolveApiBaseUrl().replace(/\/+$/, '');
  const target = `${baseUrl}/organisations/${organisationId}/datasets/${datasetId}/upload-sessions/${sessionId}/content`;

  const init: RequestInit & { duplex: 'half' } = {
    method: 'PUT',
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': request.headers.get('content-type') ?? 'application/octet-stream',
    },
    body: request.body,
    signal: request.signal,
    duplex: 'half',
  };

  let response: Response;
  try {
    response = await fetch(target, init as RequestInit);
  } catch {
    return errorResponse(502, 'upload_unavailable', 'The upload could not reach the server.');
  }

  if (response.status === 204) {
    return new Response(null, { status: 204 });
  }

  const body = await response.text();
  return new Response(body, {
    status: response.status,
    headers: { 'content-type': response.headers.get('content-type') ?? 'application/json' },
  });
};
