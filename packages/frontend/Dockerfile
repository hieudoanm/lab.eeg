# Frontend Dockerfile
# Next.js standalone output for minimal image

# Stage 1: Install dependencies
FROM node:26.13.0-alpine AS deps
WORKDIR /app

RUN npm install --global pnpm@11.7.0

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY packages/frontend/package.json packages/frontend/package.json
RUN pnpm install --frozen-lockfile --filter @eeg-lab/web...

# Stage 2: Build
FROM node:26.13.0-alpine AS builder
WORKDIR /app

RUN npm install --global pnpm@11.7.0

COPY --from=deps /app/node_modules ./node_modules
COPY --from=deps /app/packages/frontend/node_modules ./packages/frontend/node_modules
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY packages/frontend ./packages/frontend

ENV NEXT_TELEMETRY_DISABLED=1
ARG NEXT_PUBLIC_API_BASE_URL=http://localhost:8000/api/v1
ENV NEXT_PUBLIC_API_BASE_URL=${NEXT_PUBLIC_API_BASE_URL}
RUN pnpm --filter @eeg-lab/web build

# Stage 3: Runtime
FROM node:26.13.0-alpine AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

# Copy standalone output
COPY --from=builder --chown=nextjs:nodejs /app/packages/frontend/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/packages/frontend/.next/static ./packages/frontend/.next/static
COPY --from=builder --chown=nextjs:nodejs /app/packages/frontend/public ./packages/frontend/public

USER nextjs

EXPOSE 3000

ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

CMD ["node", "packages/frontend/server.js"]
