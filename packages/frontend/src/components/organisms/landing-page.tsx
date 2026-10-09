import Link from 'next/link';
import type { ReactNode } from 'react';

import { StatusBadge } from '@/components/atoms';
import { buttonClasses } from '@/components/atoms/button';
import { ThemeToggle } from '@/components/atoms/theme-toggle';
import { cn } from '@/lib/cn';

const NAV_LINKS = [
  { href: '#explore', label: 'Explore' },
  { href: '#analyse', label: 'Analyse' },
  { href: '#organise', label: 'Organise' },
  { href: '#docs', label: 'Docs' },
] as const;

const HERO_QUALITIES = [
  'Organisation-scoped',
  'SHA-256 provenance',
  'PhysioNet compatible',
] as const;

const SAMPLE_METADATA = [
  { label: 'Format', value: 'EDF' },
  { label: 'Duration', value: '60 s' },
  { label: 'Sampling rate', value: '160 Hz' },
  { label: 'Channels', value: '64 EEG' },
  { label: 'Task', value: 'Motor imagery' },
  { label: 'Reference', value: 'Average' },
] as const;

const FOOTER_GROUPS = [
  {
    heading: 'Product',
    links: [
      { href: '#explore', label: 'Signal Explorer' },
      { href: '#analyse', label: 'Analyses' },
      { href: '#organise', label: 'Studies' },
      { href: '#', label: 'Explore samples' },
    ],
  },
  {
    heading: 'Documentation',
    links: [
      { href: '#docs', label: 'Getting started' },
      { href: '#docs', label: 'Formats and BIDS EEG' },
      { href: '#docs', label: 'REST API' },
      { href: '#docs', label: 'Python SDK' },
    ],
  },
  {
    heading: 'Legal',
    links: [
      { href: '#', label: 'Licence' },
      { href: '#', label: 'Privacy policy' },
      { href: '#', label: 'Terms of use' },
      { href: '#', label: 'Citation' },
    ],
  },
] as const;

const WaveIcon = ({ className }: { className?: string }) => (
  <svg
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth={1.7}
    strokeLinecap='round'
    strokeLinejoin='round'
    aria-hidden='true'
    className={className}>
    <path d='M3 12h4l3-8 4 16 3-8h4' />
  </svg>
);

const CompassIcon = ({ className }: { className?: string }) => (
  <svg
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth={1.7}
    strokeLinecap='round'
    strokeLinejoin='round'
    aria-hidden='true'
    className={className}>
    <circle cx='12' cy='12' r='8' />
    <path d='m14.5 9.5-1.8 4.7-4.7 1.8 1.8-4.7Z' />
  </svg>
);

const TagIcon = ({ className }: { className?: string }) => (
  <svg
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth={1.7}
    strokeLinecap='round'
    strokeLinejoin='round'
    aria-hidden='true'
    className={className}>
    <rect x='7' y='7' width='10' height='10' rx='1' />
    <path d='M4 10h3M4 14h3M17 10h3M17 14h3M10 4v3M14 4v3M10 17v3M14 17v3' />
  </svg>
);

const GridIcon = ({ className }: { className?: string }) => (
  <svg
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth={1.7}
    strokeLinecap='round'
    strokeLinejoin='round'
    aria-hidden='true'
    className={className}>
    <rect x='3' y='3' width='7' height='7' />
    <rect x='14' y='3' width='7' height='7' />
    <rect x='14' y='14' width='7' height='7' />
    <rect x='3' y='14' width='7' height='7' />
  </svg>
);

const CheckIcon = ({ className }: { className?: string }) => (
  <svg
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth={2}
    strokeLinecap='round'
    strokeLinejoin='round'
    aria-hidden='true'
    className={className}>
    <path d='m20 6-11 11-5-5' />
  </svg>
);

interface Capability {
  id: string;
  icon: ReactNode;
  title: string;
  description: string;
  points: string[];
}

const CAPABILITIES: Capability[] = [
  {
    id: 'explore',
    icon: <WaveIcon className='h-5 w-5' />,
    title: 'Explore',
    description:
      'An interactive multichannel signal viewer with channel selection, time navigation, amplitude scaling, and event markers — plus filtering previews that never touch the stored recording.',
    points: [
      'Channel browser with explicit types and units',
      'Time interval selection with validation',
      'Display downsampling never alters analysis inputs',
    ],
  },
  {
    id: 'analyse',
    icon: <TagIcon className='h-5 w-5' />,
    title: 'Analyse',
    description:
      'Run asynchronous descriptive analyses — mean, standard deviation, min, max, peak-to-peak — with explicit parameters, honest status feedback, and full provenance tracking.',
    points: [
      'Descriptive statistics with valid sample counts',
      'Parameters rendered from typed schema with units',
      'Results include algorithm version and checksums',
    ],
  },
  {
    id: 'organise',
    icon: <GridIcon className='h-5 w-5' />,
    title: 'Organise',
    description:
      'Organise research studies, upload datasets, and control access at the organisation level — every tenant is isolated by default.',
    points: [
      'Studies group related datasets and analyses',
      'Organisation-scoped membership and roles',
      'Every tenant query filters by verified context',
    ],
  },
];

const Wordmark = () => (
  <Link href='/' className='flex items-center gap-2.5'>
    <span
      aria-hidden='true'
      className='bg-primary text-primary-content flex h-7 w-7 items-center justify-center rounded-md text-[13px] font-bold'>
      E
    </span>
    <span className='text-base-content text-[15px] font-semibold tracking-tight'>EEG Lab</span>
  </Link>
);

const LandingHeader = () => (
  <header className='border-base-300 bg-base-100/90 sticky top-0 z-30 border-b backdrop-blur'>
    <div className='mx-auto flex h-14 max-w-6xl items-center justify-between gap-4 px-4 md:px-6'>
      <Wordmark />
      <nav aria-label='Landing' className='hidden items-center gap-1 text-[13px] md:flex'>
        {NAV_LINKS.map((link) => (
          <a
            key={link.label}
            href={link.href}
            className='text-base-content/60 hover:text-base-content rounded-md px-3 py-2'>
            {link.label}
          </a>
        ))}
      </nav>
      <div className='flex items-center gap-2'>
        <ThemeToggle />
        <Link
          href='/login'
          className='text-base-content/60 hover:text-base-content inline-flex h-9 items-center rounded-md px-3 text-[13px] font-medium'>
          Sign in
        </Link>
        <a href='#' className={cn(buttonClasses('primary', 'sm'), 'h-9 gap-2 px-3.5 text-[13px]')}>
          <WaveIcon className='h-4 w-4' />
          Open sample recording
        </a>
      </div>
    </div>
  </header>
);

const SignalPreview = () => (
  <svg
    viewBox='0 0 640 400'
    role='img'
    aria-label='Multichannel EEG signal preview with channel selection and time navigation'
    className='block w-full'>
    <rect width='640' height='400' fill='#ffffff' />
    <g stroke='#dce3ec' strokeWidth={1}>
      <line x1='0' y1='196' x2='640' y2='196' />
      <line x1='0' y1='400' x2='640' y2='400' />
    </g>
    <g fontFamily='monospace' fontSize={11} fill='#5b6578'>
      <text x='24' y='20'>
        RECORDING
      </text>
      <text x='24' y='392' fontSize={10}>
        0.0 s
      </text>
      <text x='300' y='392' fontSize={10}>
        3.0 s
      </text>
      <text x='580' y='392' fontSize={10}>
        6.0 s
      </text>
    </g>
    <g stroke='#dce3ec'>
      <line x1='46' y1='30' x2='46' y2='386' />
    </g>
    <g fontFamily='monospace' fontSize={10}>
      <text x='52' y='42' fill='#4059c7'>
        ☑ Fp1
      </text>
      <text x='52' y='72' fill='#4059c7'>
        ☑ Fp2
      </text>
      <text x='52' y='102' fill='#5b6578'>
        ☐ F3
      </text>
      <text x='52' y='132' fill='#5b6578'>
        ☐ F4
      </text>
      <text x='52' y='162' fill='#4059c7'>
        ☑ C3
      </text>
      <text x='52' y='192' fill='#4059c7'>
        ☑ C4
      </text>
      <text x='52' y='222' fill='#5b6578'>
        ☐ P3
      </text>
      <text x='52' y='252' fill='#5b6578'>
        ☐ P4
      </text>
      <text x='52' y='282' fill='#4059c7'>
        ☑ O1
      </text>
      <text x='52' y='312' fill='#4059c7'>
        ☑ O2
      </text>
      <text x='52' y='382' fill='#172033'>
        6 selected
      </text>
    </g>
    <g fill='none' strokeWidth={1.1} stroke='#3b82f6'>
      <polyline points='110,40 140,45 170,38 200,50 230,42 260,55 290,48 320,60 350,52 380,65 410,58 440,70 470,62 500,75 530,68 560,80 590,72' />
      <polyline points='110,70 140,75 170,68 200,80 230,72 260,85 290,78 320,90 350,82 380,95 410,88 440,100 470,92 500,105 530,98 560,110 590,102' />
      <polyline
        points='110,160 140,165 170,158 200,170 230,162 260,175 290,168 320,180 350,172 380,185 410,178 440,190 470,182 500,195 530,188 560,200 590,192'
        opacity={0.7}
      />
      <polyline
        points='110,190 140,195 170,188 200,200 230,192 260,205 290,198 320,210 350,202 380,215 410,208 440,220 470,212 500,225 530,218 560,230 590,222'
        opacity={0.65}
      />
      <polyline
        points='110,280 140,285 170,278 200,290 230,282 260,295 290,288 320,300 350,292 380,305 410,298 440,310 470,302 500,315 530,308 560,320 590,312'
        opacity={0.6}
      />
      <polyline
        points='110,310 140,315 170,308 200,320 230,312 260,325 290,318 320,330 350,322 380,335 410,328 440,340 470,332 500,345 530,338 560,350 590,342'
        opacity={0.55}
      />
    </g>
    <g stroke='#986500' strokeWidth={1} strokeDasharray='3 3'>
      <line x1='200' y1='28' x2='200' y2='392' />
      <text x='182' y='20' fill='#986500' fontFamily='monospace' fontSize={10}>
        T1
      </text>
      <line x1='380' y1='28' x2='380' y2='392' />
      <text x='362' y='20' fill='#986500' fontFamily='monospace' fontSize={10}>
        T2
      </text>
    </g>
    <g>
      <rect x='488' y='210' width='140' height='176' rx='4' fill='#ffffff' stroke='#dce3ec' />
      <text x='496' y='230' fill='#5b6578' fontFamily='monospace' fontSize={10}>
        INTERVAL
      </text>
      <text x='496' y='250' fill='#172033' fontFamily='monospace' fontSize={11}>
        Start: 0.0 s
      </text>
      <text x='496' y='270' fill='#172033' fontFamily='monospace' fontSize={11}>
        End: 6.0 s
      </text>
      <text x='496' y='300' fill='#5b6578' fontFamily='monospace' fontSize={10}>
        SAMPLING
      </text>
      <text x='496' y='320' fill='#172033' fontFamily='monospace' fontSize={11}>
        160 Hz
      </text>
      <text x='496' y='350' fill='#5b6578' fontFamily='monospace' fontSize={10}>
        CHANNELS
      </text>
      <text x='496' y='370' fill='#172033' fontFamily='monospace' fontSize={11}>
        64 EEG
      </text>
    </g>
  </svg>
);

const Hero = () => (
  <section className='border-base-300 relative overflow-hidden border-b'>
    <div className='bg-primary/10 pointer-events-none absolute -top-40 left-1/2 h-[420px] w-[820px] -translate-x-1/2 rounded-full opacity-60 blur-3xl' />
    <div className='relative mx-auto grid max-w-6xl items-center gap-12 px-4 py-16 md:px-6 md:py-24 lg:grid-cols-[1.05fr_.95fr]'>
      <div>
        <div className='border-base-300 bg-base-200 text-base-content/60 mb-5 inline-flex items-center gap-2 rounded-full border px-3 py-1 text-xs font-medium'>
          <span className='bg-success h-1.5 w-1.5 rounded-full' />
          Open-source · Research and educational tool
        </div>
        <h1 className='text-base-content text-[40px] leading-[46px] font-bold tracking-tight md:text-[52px] md:leading-[58px]'>
          Organise studies, explore EEG signals, and run reproducible analyses in the cloud.
        </h1>
        <p className='text-base-content/60 mt-5 max-w-xl text-base leading-7'>
          EEG Lab is a cloud workbench for neuroscience research: interactive signal exploration,
          asynchronous descriptive analyses, organisation-scoped access control, and
          machine-readable provenance. No server to configure, no analysis to reproduce by hand.
        </p>
        <div className='mt-8 flex flex-wrap items-center gap-3'>
          <a href='#' className={cn(buttonClasses('primary'), 'h-11 gap-2 px-5')}>
            <CompassIcon className='h-4 w-4' />
            Explore the sample recording
          </a>
          <Link href='/login' className={cn(buttonClasses('secondary'), 'h-11 gap-2 px-5')}>
            Sign in
          </Link>
        </div>
        <div className='text-base-content/50 mt-8 flex flex-wrap items-center gap-x-6 gap-y-2 text-xs'>
          {HERO_QUALITIES.map((quality) => (
            <span key={quality} className='inline-flex items-center gap-1.5'>
              <CheckIcon className='h-3.5 w-3.5' />
              {quality}
            </span>
          ))}
        </div>
      </div>

      <div className='card border-base-300 bg-base-100 rounded-xl border p-3 shadow-sm'>
        <div className='flex items-center justify-between px-1 pb-2'>
          <div className='flex items-center gap-2'>
            <span className='bg-error h-2.5 w-2.5 rounded-full' />
            <span className='bg-warning h-2.5 w-2.5 rounded-full' />
            <span className='bg-success h-2.5 w-2.5 rounded-full' />
            <span className='text-base-content/50 ml-2 text-xs'>
              signal explorer — sample / EDF
            </span>
          </div>
          <StatusBadge tone='success' label='ready' />
        </div>
        <div className='border-base-300 bg-base-100 overflow-hidden rounded-lg border'>
          <SignalPreview />
        </div>
      </div>
    </div>
  </section>
);

const Capabilities = () => (
  <section id='explore' className='mx-auto max-w-6xl px-4 py-16 md:px-6'>
    <div className='mb-10 max-w-2xl'>
      <div className='text-base-content/50 mb-2 text-xs font-semibold tracking-wider uppercase'>
        Capabilities
      </div>
      <h2 className='text-base-content text-[28px] leading-9 font-semibold tracking-tight'>
        Studies, signals, and analyses on the same platform
      </h2>
      <p className='text-base-content/60 mt-2 text-sm'>
        Display transforms are never recorded as analytical inputs. The recording is the primary
        content; everything else recedes.
      </p>
    </div>
    <div className='grid gap-6 md:grid-cols-3'>
      {CAPABILITIES.map((capability) => (
        <div
          key={capability.id}
          id={capability.id}
          className='card border-base-300 bg-base-100 rounded-lg border p-6'>
          <div className='bg-primary/10 text-primary mb-4 flex h-10 w-10 items-center justify-center rounded-md'>
            {capability.icon}
          </div>
          <h3 className='text-base-content text-lg font-semibold'>{capability.title}</h3>
          <p className='text-base-content/60 mt-2 text-sm'>{capability.description}</p>
          <ul className='text-base-content/60 mt-4 space-y-2 text-[13px]'>
            {capability.points.map((point) => (
              <li key={point} className='flex items-start gap-2'>
                <span className='bg-primary mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full' />
                {point}
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  </section>
);

const SampleThumbnail = () => (
  <svg
    viewBox='0 0 320 220'
    role='img'
    aria-label='Sample EEG recording thumbnail'
    className='block w-full'>
    <rect width='320' height='220' fill='#ffffff' />
    <g stroke='#dce3ec'>
      <line x1='160' y1='0' x2='160' y2='220' />
      <line x1='0' y1='110' x2='320' y2='110' />
    </g>
    <g fill='none' stroke='#3b82f6' strokeWidth={1.1}>
      <polyline points='18,60 58,64 98,58 138,66 178,60 218,74 258,66 300,80' />
      <polyline points='18,100 58,104 98,98 138,106 178,100 218,114 258,106 300,120' />
      <polyline points='18,140 58,144 98,138 138,146 178,140 218,154 258,146 300,160' />
      <polyline points='18,180 58,184 98,178 138,186 178,180 218,194 258,186 300,200' />
    </g>
    <g stroke='#986500' strokeDasharray='2 3'>
      <line x1='178' y1='30' x2='178' y2='192' />
    </g>
    <text x='14' y='26' fill='#5b6578' fontSize={11} fontFamily='monospace'>
      S001 · EDF · PhysioNet
    </text>
  </svg>
);

const SampleRecording = () => (
  <section className='border-base-300 bg-base-200/50 border-t'>
    <div className='mx-auto max-w-6xl px-4 py-16 md:px-6'>
      <div className='mb-8 max-w-2xl'>
        <div className='text-base-content/50 mb-2 text-xs font-semibold tracking-wider uppercase'>
          Start without uploading anything
        </div>
        <h2 className='text-base-content text-[28px] leading-9 font-semibold tracking-tight'>
          Public sample recording
        </h2>
      </div>
      <div className='card border-base-300 bg-base-100 grid gap-6 rounded-xl border p-6 md:grid-cols-[1fr_1.2fr]'>
        <div className='border-base-300 bg-base-100 overflow-hidden rounded-lg border'>
          <SampleThumbnail />
        </div>
        <div>
          <div className='flex flex-wrap items-center gap-2'>
            <h3 className='text-base-content text-lg font-semibold'>
              Motor Movement/Imagery — S001
            </h3>
            <span className='badge badge-soft badge-primary font-medium'>public sample</span>
            <StatusBadge tone='success' label='validated' />
          </div>
          <p className='text-base-content/60 mt-3 text-sm'>
            A short anonymised EEG recording from the PhysioNet EEG Motor Movement/Imagery Dataset,
            with 64 EEG channels, 160 Hz sampling rate, and task markers. Explore it read-only, then
            sign in to save a view or run a descriptive analysis.
          </p>
          <dl className='mt-5 grid grid-cols-2 gap-x-6 gap-y-3 text-[13px] sm:grid-cols-3'>
            {SAMPLE_METADATA.map((item) => (
              <div key={item.label}>
                <dt className='text-base-content/50'>{item.label}</dt>
                <dd className='text-base-content mt-0.5 font-mono'>{item.value}</dd>
              </div>
            ))}
          </dl>
          <div className='border-base-300 bg-base-200 text-base-content/60 mt-5 rounded-md border p-3 font-mono text-xs leading-5'>
            sha256:8f2d3b9c5e…f315d9e7c22f6e04
          </div>
          <p className='text-base-content/50 mt-4 text-xs leading-5'>
            Source: PhysioNet EEG Motor Movement/Imagery Dataset (PhysioNet Licence). Attribution is
            shown with the sample and is never hidden behind an interaction. Not for clinical use.
          </p>
          <div className='mt-5 flex flex-wrap gap-3'>
            <a href='#' className={cn(buttonClasses('primary', 'sm'), 'gap-2 px-3.5')}>
              Open sample recording
            </a>
            <a href='#' className={cn(buttonClasses('secondary', 'sm'), 'gap-2 px-3.5')}>
              Read the citation
            </a>
          </div>
        </div>
      </div>
    </div>
  </section>
);

const CallToAction = () => (
  <section className='border-base-300 border-t'>
    <div className='mx-auto max-w-6xl px-4 py-16 md:px-6'>
      <div className='card border-base-300 from-base-100 to-primary/10 flex flex-col items-start justify-between gap-6 rounded-xl border bg-gradient-to-br p-8 md:flex-row md:items-center'>
        <div className='max-w-xl'>
          <h2 className='text-base-content text-[22px] leading-8 font-semibold'>
            From &ldquo;I found an EEG recording&rdquo; to a shareable result
          </h2>
          <p className='text-base-content/60 mt-2 text-sm'>
            Create a study, upload a supported recording, open the Signal Explorer, and run your
            first descriptive analysis in minutes.
          </p>
        </div>
        <div className='flex flex-wrap gap-3'>
          <Link href='/login' className={cn(buttonClasses('primary'), 'h-11 px-5')}>
            Create an account
          </Link>
          <a href='#' className={cn(buttonClasses('secondary'), 'h-11 px-5')}>
            Open sample recording
          </a>
        </div>
      </div>
    </div>
  </section>
);

const LandingFooter = () => (
  <footer className='border-base-300 bg-base-200/50 border-t'>
    <div className='mx-auto grid max-w-6xl gap-8 px-4 py-12 md:grid-cols-4 md:px-6'>
      <div>
        <Wordmark />
        <p className='text-base-content/60 mt-3 max-w-xs text-[13px]'>
          A cloud-based neuroscience research workbench for developers, computational
          neuroscientists, and neuroscience enthusiasts.
        </p>
      </div>
      {FOOTER_GROUPS.map((group) => (
        <div key={group.heading}>
          <div className='text-base-content/50 text-xs font-semibold tracking-wider uppercase'>
            {group.heading}
          </div>
          <ul className='mt-3 space-y-2 text-[13px]'>
            {group.links.map((link) => (
              <li key={link.label}>
                <a href={link.href} className='text-base-content/60 hover:text-base-content'>
                  {link.label}
                </a>
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
    <div className='border-base-300 border-t'>
      <div className='text-base-content/50 mx-auto flex max-w-6xl flex-col justify-between gap-2 px-4 py-5 text-xs md:flex-row md:px-6'>
        <span>© 2026 EEG Lab · Research and educational use only. Not a medical device.</span>
        <span className='font-mono'>v0.1.0 · prototype</span>
      </div>
    </div>
  </footer>
);

export const LandingPage = () => (
  <div className='bg-background text-base-content min-h-screen antialiased'>
    <LandingHeader />
    <main>
      <Hero />
      <Capabilities />
      <SampleRecording />
      <CallToAction />
    </main>
    <LandingFooter />
  </div>
);
