import type { Metadata, Viewport } from 'next';
import { Inter } from 'next/font/google';

import { ThemeProvider, themeBootScript } from '@/components/atoms/theme-provider';
import { ToastProvider } from '@/components/atoms/toast';
import { TRPCReactProvider } from '@/trpc/react';

import './globals.css';

const inter = Inter({
  variable: '--font-inter',
  subsets: ['latin'],
  display: 'swap',
});

export const metadata: Metadata = {
  title: {
    default: 'EEG Lab',
    template: '%s · EEG Lab',
  },
  description:
    'EEG Lab is a research platform for organising EEG studies, datasets, and descriptive analyses.',
  metadataBase: new URL('https://eeg-lab.example.com'),
  alternates: {
    canonical: '/',
  },
  openGraph: {
    type: 'website',
    locale: 'en_US',
    url: 'https://eeg-lab.example.com',
    siteName: 'EEG Lab',
    title: 'EEG Lab — EEG Research Platform',
    description:
      'Organise studies, analyse EEG recordings, and run descriptive analyses with full tenant isolation.',
    images: [
      {
        url: '/og/og.png',
        width: 1200,
        height: 630,
        alt: 'EEG Lab - EEG Research Platform',
      },
    ],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'EEG Lab — EEG Research Platform',
    description:
      'Organise studies, analyse EEG recordings, and run descriptive analyses with full tenant isolation.',
    images: ['/og/og.png'],
    creator: '@eeg-lab',
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-video-preview': -1,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
  icons: {
    icon: [
      { url: '/icons/icon-192x192.png', sizes: '192x192', type: 'image/png' },
      { url: '/icons/icon-512x512.png', sizes: '512x512', type: 'image/png' },
    ],
    shortcut: '/favicon.ico',
    apple: [{ url: '/icons/icon-180x180.png', sizes: '180x180', type: 'image/png' }],
    other: [
      {
        rel: 'manifest',
        url: '/manifest.json',
      },
    ],
  },
  manifest: '/manifest.json',
};

export const viewport: Viewport = {
  themeColor: '#3b82f6',
  width: 'device-width',
  initialScale: 1,
  maximumScale: 5,
};

const RootLayout = ({ children }: LayoutProps<'/'>) => {
  return (
    <html lang='en' className={`${inter.variable} h-full`}>
      <head>
        <link rel='preconnect' href='https://fonts.googleapis.com' />
        <link rel='preconnect' href='https://fonts.gstatic.com' crossOrigin='anonymous' />
        <script dangerouslySetInnerHTML={{ __html: themeBootScript }} />
      </head>
      <body className='min-h-full'>
        <TRPCReactProvider>
          <ThemeProvider>
            <ToastProvider>{children}</ToastProvider>
          </ThemeProvider>
        </TRPCReactProvider>
      </body>
    </html>
  );
};

export default RootLayout;
