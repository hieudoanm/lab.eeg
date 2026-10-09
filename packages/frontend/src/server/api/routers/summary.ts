import { fetchOrganisationSummary } from '@/lib/api';

import { guard, organisationProcedure, router } from '../trpc';

export const summaryRouter = router({
  organisation: organisationProcedure.query(({ ctx }) => {
    return guard(() => fetchOrganisationSummary(ctx.membership.organisation_id, ctx.client));
  }),
});

export type SummaryRouter = typeof summaryRouter;
