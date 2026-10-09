/* eslint-disable react-hooks/rules-of-hooks -- factory that builds tRPC-style hooks for tests. */
import type { ReactNode } from 'react';
import { useCallback, useState } from 'react';
import { vi } from 'vitest';

type OutcomeMode = 'value' | 'error';

interface Outcome {
  mode: OutcomeMode;
  payload: unknown;
}

export interface MutationOptions<TData = unknown, TVariables = unknown> {
  onSuccess?: (data: TData, variables: TVariables) => void | Promise<void>;
  onError?: (error: unknown, variables: TVariables) => void;
  onSettled?: () => void;
}

/**
 * Test double for a tRPC mutation hook.
 *
 * Tests queue outcomes with `resolveWith` / `rejectWith`; the mock drives the
 * component's `onSuccess`/`onError` callbacks and tracks pending state so React
 * re-renders behave like the real React Query mutation.
 */
export class MutationController {
  calls: unknown[] = [];

  options: MutationOptions = {};

  private queue: Outcome[] = [];

  private last: Outcome = { mode: 'value', payload: undefined };

  resolveWith(payload: unknown): this {
    this.queue.push({ mode: 'value', payload });
    return this;
  }

  rejectWith(payload: unknown): this {
    this.queue.push({ mode: 'error', payload });
    return this;
  }

  nextOutcome(): Outcome {
    const outcome = this.queue.shift();
    if (outcome) {
      this.last = outcome;
    }
    return this.last;
  }
}

const controllers = new Map<string, MutationController>();

export const mutationController = (key: string): MutationController => {
  const existing = controllers.get(key);
  if (existing) {
    return existing;
  }
  const created = new MutationController();
  controllers.set(key, created);
  return created;
};

export const resetTrpcMocks = (): void => {
  controllers.clear();
};

const useMutationMock = (key: string) => {
  return (options: MutationOptions = {}) => {
    const controller = mutationController(key);
    controller.options = options;
    const [isPending, setIsPending] = useState(false);
    const [isError, setIsError] = useState(false);
    const [isSuccess, setIsSuccess] = useState(false);

    const mutateAsync = useCallback(
      async (variables: unknown) => {
        controller.calls.push(variables);
        setIsPending(true);
        setIsError(false);
        setIsSuccess(false);
        try {
          const outcome = controller.nextOutcome();
          const settled = await outcome.payload;
          if (outcome.mode === 'error') {
            throw settled;
          }
          await options.onSuccess?.(settled, variables);
          setIsSuccess(true);
          return settled;
        } catch (error) {
          setIsError(true);
          options.onError?.(error, variables);
          throw error;
        } finally {
          setIsPending(false);
          options.onSettled?.();
        }
      },
      [controller, options],
    );

    const mutate = useCallback(
      (variables: unknown) => {
        void mutateAsync(variables).catch(() => undefined);
      },
      [mutateAsync],
    );

    return { mutate, mutateAsync, isPending, isError, isSuccess, reset: vi.fn() };
  };
};

const useQueryMock = (key: string) => {
  return (options?: { enabled?: boolean }) => {
    const controller = mutationController(key);
    const [data, setData] = useState<unknown>(undefined);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<Error | null>(null);

    const fetchData = useCallback(async () => {
      if (options?.enabled === false) return;
      setIsLoading(true);
      setError(null);
      try {
        const outcome = controller.nextOutcome();
        const settled = await outcome.payload;
        if (outcome.mode === 'error') {
          throw settled;
        }
        setData(settled);
      } catch (err) {
        setError(err instanceof Error ? err : new Error(String(err)));
      } finally {
        setIsLoading(false);
      }
    }, [controller, options?.enabled]);

    return {
      data,
      isLoading,
      error,
      isError: !!error,
      isSuccess: !isLoading && !error,
      refetch: fetchData,
    };
  };
};

const procedure = (key: string) => ({
  useMutation: useMutationMock(key),
  useQuery: useQueryMock(key),
  useInfiniteQuery: vi.fn(),
  useUtils: () => ({
    [key]: {
      fetch: vi.fn(),
    },
  }),
});

const TRPCReactProvider = ({ children }: { children: ReactNode }) => children;

/**
 * Build the value returned by `vi.mock('@/trpc/react', ...)`.
 *
 * Components call `api.<router>.<procedure>.useMutation(...)`; each procedure
 * receives its own {@link MutationController} keyed by `router.procedure`.
 */
export const buildTrpcMock = () => ({
  api: {
    useUtils: () => ({
      signals: {
        window: { fetch: vi.fn() },
      },
    }),
    identity: {
      session: procedure('identity.session'),
      current: procedure('identity.current'),
      organisations: procedure('identity.organisations'),
      organisation: procedure('identity.organisation'),
      members: procedure('identity.members'),
      signIn: procedure('identity.signIn'),
      signOut: procedure('identity.signOut'),
      selectOrganisation: procedure('identity.selectOrganisation'),
    },
    studies: {
      list: procedure('studies.list'),
      byId: procedure('studies.byId'),
      create: procedure('studies.create'),
      update: procedure('studies.update'),
      archive: procedure('studies.archive'),
    },
    datasets: {
      list: procedure('datasets.list'),
      byId: procedure('datasets.byId'),
      register: procedure('datasets.register'),
    },
    uploads: {
      createSession: procedure('uploads.createSession'),
      finalise: procedure('uploads.finalise'),
    },
    analyses: {
      list: procedure('analyses.list'),
      byId: procedure('analyses.byId'),
      result: procedure('analyses.result'),
      submit: procedure('analyses.submit'),
      cancel: procedure('analyses.cancel'),
    },
    signals: {
      window: procedure('signals.window'),
    },
  },
  TRPCReactProvider,
});
