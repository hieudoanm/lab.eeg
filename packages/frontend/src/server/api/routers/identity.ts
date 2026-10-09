import {
  endSession,
  fetchOrganisation,
  fetchOrganisationMembers,
  fetchOrganisations,
  issueDevelopmentToken,
} from '@/lib/api';
import { selectActiveMembership } from '@/lib/session';
import { revalidatePath } from 'next/cache';
import { z } from 'zod';

import { guard, organisationProcedure, protectedProcedure, publicProcedure, router } from '../trpc';

const tokenMaxAgeSeconds = (expiresAt: string): number => {
  const remaining = Math.floor((Date.parse(expiresAt) - Date.now()) / 1000);
  return Number.isFinite(remaining) && remaining > 0 ? remaining : 3600;
};

/**
 * Identity and session procedures.
 *
 * `session` is the single read used by server components to resolve the signed
 * in user and their active organisation; the mutations own the auth cookies.
 */
export const identityRouter = router({
  session: protectedProcedure.query(({ ctx }) => ({
    user: ctx.user,
    membership: selectActiveMembership(ctx.user, ctx.organisationPreference),
  })),

  current: protectedProcedure.query(({ ctx }) => ctx.user),

  organisations: protectedProcedure.query(({ ctx }) => guard(() => fetchOrganisations(ctx.client))),

  organisation: organisationProcedure.query(({ ctx }) =>
    guard(() => fetchOrganisation(ctx.membership.organisation_id, ctx.client)),
  ),

  members: organisationProcedure.query(({ ctx }) =>
    guard(() => fetchOrganisationMembers(ctx.membership.organisation_id, ctx.client)),
  ),

  signIn: publicProcedure
    .input(
      z.object({
        subject: z.string().trim().min(1),
        email: z.string().email().optional(),
        displayName: z.string().min(1).optional(),
      }),
    )
    .mutation(async ({ ctx, input }) => {
      const response = await guard(() =>
        issueDevelopmentToken(
          {
            subject: input.subject,
            email: input.email,
            display_name: input.displayName,
          },
          ctx.client,
        ),
      );

      await ctx.setSessionToken(response.access_token, tokenMaxAgeSeconds(response.expires_at));
      revalidatePath('/app', 'layout');
      return { expiresAt: response.expires_at };
    }),

  signOut: protectedProcedure.mutation(async ({ ctx }) => {
    try {
      await endSession(ctx.client);
    } catch {
      // A failed upstream logout must not trap the user in a signed-in state.
    }
    await ctx.clearSessionToken();
    revalidatePath('/app', 'layout');
  }),

  selectOrganisation: protectedProcedure
    .input(z.object({ organisationId: z.string().min(1) }))
    .mutation(async ({ ctx, input }) => {
      await ctx.setOrganisationPreference(input.organisationId);
      revalidatePath('/app', 'layout');
    }),
});

export type IdentityRouter = typeof identityRouter;
