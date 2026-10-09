import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { api, TRPCReactProvider } from '@/trpc/react';

describe('TRPCReactProvider', () => {
  it('provides the tRPC and query clients and renders children', () => {
    render(
      <TRPCReactProvider>
        <span>hello</span>
      </TRPCReactProvider>,
    );

    expect(screen.getByText('hello')).toBeInTheDocument();
    expect(api.Provider).toBeDefined();
  });
});
