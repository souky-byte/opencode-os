#!/usr/bin/env node

import { spawn, execSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'node:http';
import { createReadStream } from 'node:fs';
import { readFile } from 'node:fs/promises';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');

// Configuration
const OPENCODE_PORT = 4199;  // Dedicated port for opencode-studio
const SERVER_PORT = process.env.PORT || 3001;
const OPENCODE_URL = `http://localhost:${OPENCODE_PORT}`;

// Parse CLI arguments
const args = process.argv.slice(2);
const showHelp = args.includes('--help') || args.includes('-h');

if (showHelp) {
  console.log(`
opencode-studio - AI-powered code analysis and documentation

Usage:
  npx opencode-studio [options]

Options:
  --port <port>     Server port (default: 3001)
  --help, -h        Show this help message

Environment:
  PORT              Server port (default: 3001)
  OPENCODE_URL      OpenCode server URL (default: http://localhost:4199)
`);
  process.exit(0);
}

// Find opencode binary
function findOpencodeBinary() {
  const homeDir = process.env.HOME || process.env.USERPROFILE;

  // Check ~/.opencode/bin/opencode
  if (homeDir) {
    const opencodePath = join(homeDir, '.opencode', 'bin', 'opencode');
    if (existsSync(opencodePath)) {
      return opencodePath;
    }
  }

  // Try to find in PATH
  try {
    const which = process.platform === 'win32' ? 'where' : 'which';
    const result = execSync(`${which} opencode`, { encoding: 'utf-8' }).trim();
    if (result) return result.split('\n')[0];
  } catch {
    // Not found in PATH
  }

  return null;
}

// Check if server is healthy
async function healthCheck(url) {
  try {
    const response = await fetch(`${url}/doc`, {
      signal: AbortSignal.timeout(500)
    });
    return response.ok;
  } catch {
    return false;
  }
}

// Wait for server to be ready
async function waitForServer(url, maxAttempts = 20) {
  for (let i = 0; i < maxAttempts; i++) {
    if (await healthCheck(url)) {
      return true;
    }
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  return false;
}

// Find the server binary
function findServerBinary() {
  // Check for prebuilt binary in package
  const binaryName = process.platform === 'win32' ? 'server.exe' : 'server';
  const prebuiltPath = join(ROOT, 'bin', binaryName);
  if (existsSync(prebuiltPath)) {
    return prebuiltPath;
  }

  // Check target/release (for development)
  const releasePath = join(ROOT, 'target', 'release', binaryName);
  if (existsSync(releasePath)) {
    return releasePath;
  }

  // Check target/debug (for development)
  const debugPath = join(ROOT, 'target', 'debug', binaryName);
  if (existsSync(debugPath)) {
    return debugPath;
  }

  return null;
}

async function main() {
  console.log('🚀 Starting opencode-studio...\n');

  // 1. Check for opencode
  const opencodeBinary = findOpencodeBinary();
  if (!opencodeBinary) {
    console.error('❌ OpenCode not found!\n');
    console.error('Install with: curl -fsSL https://opencode.ai/install.sh | sh\n');
    process.exit(1);
  }
  console.log(`✓ Found opencode: ${opencodeBinary}`);

  // 2. Check if opencode is already running on our port
  const opencodeRunning = await healthCheck(OPENCODE_URL);
  let opencodeProcess = null;

  if (!opencodeRunning) {
    console.log(`  Starting opencode on port ${OPENCODE_PORT}...`);

    opencodeProcess = spawn(opencodeBinary, ['serve', '--port', String(OPENCODE_PORT)], {
      stdio: ['ignore', 'pipe', 'pipe'],
      detached: false,
    });

    opencodeProcess.on('error', (err) => {
      console.error(`❌ Failed to start opencode: ${err.message}`);
      process.exit(1);
    });

    // Wait for opencode to be ready
    const ready = await waitForServer(OPENCODE_URL);
    if (!ready) {
      console.error('❌ OpenCode failed to start within timeout');
      opencodeProcess.kill();
      process.exit(1);
    }
  }
  console.log(`✓ OpenCode ready at ${OPENCODE_URL}`);

  // 3. Start the server
  const serverBinary = findServerBinary();
  if (!serverBinary) {
    console.error('❌ Server binary not found!');
    console.error('Run: cargo build --release');
    if (opencodeProcess) opencodeProcess.kill();
    process.exit(1);
  }

  console.log(`  Starting server on port ${SERVER_PORT}...`);

  const serverProcess = spawn(serverBinary, [], {
    stdio: 'inherit',
    env: {
      ...process.env,
      PORT: String(SERVER_PORT),
      OPENCODE_URL: OPENCODE_URL,
    },
  });

  serverProcess.on('error', (err) => {
    console.error(`❌ Failed to start server: ${err.message}`);
    if (opencodeProcess) opencodeProcess.kill();
    process.exit(1);
  });

  console.log(`\n✓ opencode-studio ready at http://localhost:${SERVER_PORT}\n`);

  // Handle shutdown
  const cleanup = () => {
    console.log('\n🛑 Shutting down...');
    serverProcess.kill();
    if (opencodeProcess) opencodeProcess.kill();
    process.exit(0);
  };

  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);

  // Wait for server to exit
  serverProcess.on('exit', (code) => {
    if (opencodeProcess) opencodeProcess.kill();
    process.exit(code || 0);
  });
}

main().catch((err) => {
  console.error('❌ Fatal error:', err);
  process.exit(1);
});
