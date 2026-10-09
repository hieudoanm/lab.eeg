import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('server-only', () => ({}));

const { fetchRequestHandler, createTRPCContext } = vi.hoisted(() => ({
  fetchRequestHandler: vi.fn(),
  createTRPCContext: vi.fn(),
}));

vi.mock('@trpc/server/adapters/fetch', () => ({ fetchRequestHandler }));
vi.mock('@/server/api/trpc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/server/api/trpc')>();
  return { ...actual, createTRPCContext };
});

import { appRouter } from '@/server/api/root';
import { GET, POST } from '@/app/api/trpc/[trpc]/route';

beforeEach(() => {
  vi.clearAllMocks();
});

describe('tRPC route handler', () => {
  it('serves GET and POST through the fetch request handler', async () => {
    const response = new Response(null, { status: 204 });
    fetchRequestHandler.mockResolvedValue(response);
    const request = new Request('http://localhost/api/trpc/identity.current');

    await expect(GET(request)).resolves.toBe(response);

    expect(fetchRequestHandler).toHaveBeenCalledTimes(1);
    const options = fetchRequestHandler.mock.calls[0]?.[0];
    expect(options).toMatchObject({ endpoint: '/api/trpc', req: request, router: appRouter });

    await options.createContext();
    expect(createTRPCContext).toHaveBeenCalledTimes(1);

    expect(POST).toBe(GET);
  });
});
