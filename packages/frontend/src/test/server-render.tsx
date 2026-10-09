import { renderToReadableStream } from 'react-dom/server';
import { afterEach } from 'vitest';

/**
 * Render a (possibly async) React Server Component to a DOM subtree so that
 * `@testing-library` queries can inspect the produced markup.
 *
 * The Next.js App Router renders server components on the server, so async
 * components and `<Suspense>` boundaries are not supported by the client
 * renderer used by `render()`. Rendering through `react-dom/server` mirrors the
 * production path and lets the rendered output be asserted directly.
 */
export const renderServer = async (ui: React.ReactElement): Promise<string> => {
  const stream = await renderToReadableStream(ui);
  await stream.allReady;
  const html = await new Response(stream).text();
  document.body.innerHTML = html;
  return html;
};

afterEach(() => {
  document.body.innerHTML = '';
});
