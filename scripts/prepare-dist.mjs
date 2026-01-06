#!/usr/bin/env node

/**
 * Prepares the distribution package:
 * 1. Copies frontend build to dist/
 * 2. Copies server binary to bin/
 */

import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');

const FRONTEND_BUILD = join(ROOT, 'frontend', 'dist');
const DIST_DIR = join(ROOT, 'dist');
const BIN_DIR = join(ROOT, 'bin');

// Clean dist
if (existsSync(DIST_DIR)) {
  rmSync(DIST_DIR, { recursive: true });
}
mkdirSync(DIST_DIR, { recursive: true });

// Copy frontend build
if (existsSync(FRONTEND_BUILD)) {
  console.log('Copying frontend build to dist/...');
  cpSync(FRONTEND_BUILD, join(DIST_DIR, 'frontend'), { recursive: true });
} else {
  console.error('Frontend build not found! Run: pnpm run build:frontend');
  process.exit(1);
}

// Check for server binary
const isWindows = process.platform === 'win32';
const binaryName = isWindows ? 'server.exe' : 'server';
const serverBinary = join(ROOT, 'target', 'release', binaryName);

if (!existsSync(BIN_DIR)) {
  mkdirSync(BIN_DIR, { recursive: true });
}

if (existsSync(serverBinary)) {
  console.log('Copying server binary to bin/...');
  cpSync(serverBinary, join(BIN_DIR, binaryName));
} else {
  console.warn('Server binary not found in target/release/');
  console.warn('Binary will be downloaded during installation');
}

console.log('Distribution package prepared!');
console.log('');
console.log('To publish:');
console.log('  npm publish');
console.log('');
console.log('To test locally:');
console.log('  npm link');
console.log('  opencode-studio');
