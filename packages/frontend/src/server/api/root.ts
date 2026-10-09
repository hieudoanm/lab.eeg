import { analysesRouter } from './routers/analyses';
import { datasetsRouter } from './routers/datasets';
import { identityRouter } from './routers/identity';
import { signalsRouter } from './routers/signals';
import { studiesRouter } from './routers/studies';
import { summaryRouter } from './routers/summary';
import { uploadsRouter } from './routers/uploads';
import { router } from './trpc';

/**
 * The BFF router surface exposed to the React tree.
 *
 * Every procedure delegates to the typed REST client (`@/lib/api`), so the
 * browser never talks to the Python API directly.
 */
export const appRouter = router({
  identity: identityRouter,
  studies: studiesRouter,
  datasets: datasetsRouter,
  uploads: uploadsRouter,
  analyses: analysesRouter,
  signals: signalsRouter,
  summary: summaryRouter,
});

export type AppRouter = typeof appRouter;
