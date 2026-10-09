import Link from 'next/link';

const NotFound = () => {
  return (
    <main className='mx-auto flex min-h-screen w-full max-w-md flex-col items-center justify-center gap-4 px-6 text-center'>
      <p className='text-base-content/60 font-mono text-sm'>404</p>
      <h1 className='text-base-content text-2xl font-semibold'>Page not found</h1>
      <p className='text-base-content/60 text-sm'>
        The page you requested does not exist or is not available to you.
      </p>
      <Link href='/app/dashboard' className='text-primary text-sm font-medium hover:underline'>
        Return to dashboard
      </Link>
    </main>
  );
};

export default NotFound;
