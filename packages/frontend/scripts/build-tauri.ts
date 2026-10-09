import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

console.log('Building Next.js...');
execSync('pnpm build', { stdio: 'inherit', cwd: __dirname });

console.log('Preparing Tauri assets...');
const standaloneDir = path.join(__dirname, '.next', 'standalone');
const tauriAssetsDir = path.join(__dirname, '..', 'tauri-assets');

// Clean and create tauri-assets directory
if (fs.existsSync(tauriAssetsDir)) {
  fs.rmSync(tauriAssetsDir, { recursive: true });
}
fs.mkdirSync(tauriAssetsDir, { recursive: true });

// Copy standalone files (excluding node_modules)
function copyDir(src: string, dest: string, exclude: string[] = []) {
  if (!fs.existsSync(src)) return;

  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    if (exclude.includes(entry.name)) continue;

    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);

    if (entry.isDirectory()) {
      fs.mkdirSync(destPath, { recursive: true });
      copyDir(srcPath, destPath, exclude);
    } else {
      fs.copyFileSync(srcPath, destPath);
    }
  }
}

// Copy .next/standalone without node_modules
copyDir(standaloneDir, path.join(tauriAssetsDir, '.next', 'standalone'), ['node_modules']);

// Copy public files
const publicSrc = path.join(__dirname, 'public');
const publicDest = path.join(tauriAssetsDir, '.next', 'standalone', 'public');
if (fs.existsSync(publicSrc)) {
  copyDir(publicSrc, publicDest);
}

console.log('Tauri assets prepared at:', tauriAssetsDir);
