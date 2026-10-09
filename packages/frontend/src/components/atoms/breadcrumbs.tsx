import Link from 'next/link';

export interface Breadcrumb {
  label: string;
  href?: string;
}

export const Breadcrumbs = ({ items }: { items: Breadcrumb[] }) => {
  if (items.length === 0) {
    return null;
  }
  return (
    <nav aria-label='Breadcrumb' className='breadcrumbs text-base-content/60 text-sm'>
      <ol>
        {items.map((item, index) => {
          const isLast = index === items.length - 1;
          return (
            <li key={`${item.label}-${index}`}>
              {item.href && !isLast ? (
                <Link href={item.href}>{item.label}</Link>
              ) : (
                <span aria-current={isLast ? 'page' : undefined} className='text-base-content'>
                  {item.label}
                </span>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
};
