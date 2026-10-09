import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import AppLoading from '@/app/app/loading';

describe('AppLoading', () => {
  it('renders the loading indicator', () => {
    render(<AppLoading />);

    expect(screen.getByRole('status')).toHaveTextContent('Loading workspace…');
  });
});
