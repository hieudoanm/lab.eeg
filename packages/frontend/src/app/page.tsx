import type { Metadata, NextPage } from 'next';

import { LandingPage } from '@/components/organisms';

export const metadata: Metadata = {
  title: 'EEG Lab — Organise, explore, and analyse EEG in the cloud',
  description:
    'EEG Lab is a cloud workbench for neuroscience research: interactive signal exploration, asynchronous descriptive analyses, organisation-scoped access control, and machine-readable provenance.',
};

const RootPage: NextPage = () => {
  return <LandingPage />;
};

export default RootPage;
