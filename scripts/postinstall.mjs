#!/usr/bin/env node

/**
 * Postinstall script - downloads the prebuilt server binary for the current platform
 */

import { createWriteStream, existsSync, mkdirSync, chmodSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { pipeline } from 'node:stream/promises';
import { createGunzip } from 'node:zlib';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');
const BIN_DIR = join(ROOT, 'bin');

// GitHub releases URL
const GITHUB_REPO = 'souky-byte/opencode-os';
const VERSION = process.env.npm_package_version || '0.0.1';

// Platform/arch to binary name mapping
const PLATFORM_MAP = {
  'darwin-x64': 'server-darwin-x64',
  'darwin-arm64': 'server-darwin-arm64',
  'linux-x64': 'server-linux-x64',
  'linux-arm64': 'server-linux-arm64',
  'win32-x64': 'server-win32-x64.exe',
};

function getPlatformKey() {
  return `${process.platform}-${process.arch}`;
}

function getBinaryName() {
  const key = getPlatformKey();
  const name = PLATFORM_MAP[key];
  if (!name) {
    throw new Error(`Unsupported platform: ${key}`);
  }
  return name;
}

function getLocalBinaryPath() {
  const isWindows = process.platform === 'win32';
  return join(BIN_DIR, isWindows ? 'server.exe' : 'server');
}

async function downloadBinary() {
  const binaryName = getBinaryName();
  const localPath = getLocalBinaryPath();

  // Skip if binary already exists (e.g., from prebuilt package)
  if (existsSync(localPath)) {
    console.log('Binary already exists, skipping download');
    return;
  }

  // Ensure bin directory exists
  if (!existsSync(BIN_DIR)) {
    mkdirSync(BIN_DIR, { recursive: true });
  }

  const url = `https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${binaryName}.gz`;

  console.log(`Downloading ${binaryName} for ${getPlatformKey()}...`);
  console.log(`URL: ${url}`);

  try {
    const response = await fetch(url);

    if (!response.ok) {
      if (response.status === 404) {
        console.warn(`Binary not found at ${url}`);
        console.warn('You may need to build from source: cargo build --release');
        return;
      }
      throw new Error(`Failed to download: ${response.status} ${response.statusText}`);
    }

    // Download and decompress
    const gunzip = createGunzip();
    const fileStream = createWriteStream(localPath);

    await pipeline(response.body, gunzip, fileStream);

    // Make executable on Unix
    if (process.platform !== 'win32') {
      chmodSync(localPath, 0o755);
    }

    console.log(`Successfully installed to ${localPath}`);
  } catch (error) {
    console.error('Download failed:', error.message);
    console.error('You can build from source instead: cargo build --release');
  }
}

// Only run if not in development
if (!process.env.npm_config_dev && !process.env.OPENCODE_SKIP_DOWNLOAD) {
  downloadBinary().catch(console.error);
} else {
  console.log('Skipping binary download in development mode');
}
