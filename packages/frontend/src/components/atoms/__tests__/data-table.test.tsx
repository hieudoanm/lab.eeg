import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { DataTable } from '../../atoms/data-table';

describe('DataTable', () => {
  it('renders column headers and rows', () => {
    render(
      <DataTable columns={['Name', 'Status']}>
        <tr>
          <td>Study A</td>
          <td>Ready</td>
        </tr>
      </DataTable>,
    );
    expect(screen.getByRole('columnheader', { name: 'Name' })).toBeInTheDocument();
    expect(screen.getByRole('columnheader', { name: 'Status' })).toBeInTheDocument();
    expect(screen.getByRole('cell', { name: 'Study A' })).toBeInTheDocument();
    expect(screen.getByRole('cell', { name: 'Ready' })).toBeInTheDocument();
  });

  it('renders headers with no body rows when empty', () => {
    render(<DataTable columns={['Name']}>{null}</DataTable>);
    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getAllByRole('row')).toHaveLength(1);
    expect(screen.getByRole('columnheader', { name: 'Name' })).toBeInTheDocument();
    expect(screen.queryByRole('cell')).not.toBeInTheDocument();
  });

  it('merges a custom className onto the scroll wrapper', () => {
    render(
      <DataTable columns={['Name']} className='custom-table'>
        {null}
      </DataTable>,
    );
    const wrapper = screen.getByRole('table').parentElement;
    expect(wrapper).toHaveClass('custom-table');
    expect(wrapper?.className).toContain('overflow-x-auto');
  });
});
