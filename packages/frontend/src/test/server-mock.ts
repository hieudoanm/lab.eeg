import { vi } from 'vitest';

/**
 * Shared, hoistable mock for the tRPC server caller used by React Server
 * Components.
 *
 * Tests mock `@/trpc/server` with {@link buildServerMock}, then configure
 * {@link requireSession} to resolve the session and stub procedure calls on the
 * shared `caller` object.
 */
export const caller = {
  identity: {
    session: vi.fn(),
    current: vi.fn(),
    organisations: vi.fn(),
    organisation: vi.fn(),
    members: vi.fn(),
    signIn: vi.fn(),
    signOut: vi.fn(),
    selectOrganisation: vi.fn(),
  },
  studies: {
    list: vi.fn(),
    byId: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    archive: vi.fn(),
  },
  datasets: {
    list: vi.fn(),
    byId: vi.fn(),
    register: vi.fn(),
  },
  uploads: {
    createSession: vi.fn(),
    finalise: vi.fn(),
  },
  analyses: {
    list: vi.fn(),
    listOrganisation: vi.fn(),
    byId: vi.fn(),
    result: vi.fn(),
    submit: vi.fn(),
    cancel: vi.fn(),
  },
  signals: {
    window: vi.fn(),
  },
  summary: {
    organisation: vi.fn(),
  },
};

export const requireSession = vi.fn();

export const resetServerMock = (): void => {
  requireSession.mockReset();
  for (const group of Object.values(caller)) {
    for (const fn of Object.values(group)) {
      (fn as ReturnType<typeof vi.fn>).mockReset();
    }
  }
};

export const buildServerMock = () => ({ requireSession, getCaller: vi.fn() });
