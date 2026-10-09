import { describe, expect, it, vi } from 'vitest';

vi.mock('next/font/google', () => ({
  Inter: () => ({ variable: '--font-inter' }),
}));

import { renderServer } from '@/test/server-render';
import RootLayout from '@/app/layout';

describe('RootLayout', () => {
  it('renders the document shell with the toast provider', async () => {
    const html = await renderServer(
      <RootLayout params={Promise.resolve({})}>
        <p>child content</p>
      </RootLayout>,
    );

    expect(html).toContain('lang="en"');
    expect(html).toContain('--font-inter');
    expect(html).toContain('child content');
  });
});
