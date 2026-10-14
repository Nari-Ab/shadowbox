#!/usr/bin/env node
const path = require('path');
const fs = require('fs');
const { spawnSync } = require('child_process');

const binPath = path.join(__dirname, 'shadowbox-bin');

if (!fs.existsSync(binPath)) {
  console.error('[shadowbox] Native binary not found. Running postinstall download...');
  try {
    require('../scripts/install.js');
  } catch (e) {
    console.error('[shadowbox] Error:', e.message);
    process.exit(1);
  }
}

const args = process.argv.slice(2);
const result = spawnSync(binPath, args, { stdio: 'inherit' });

if (result.error) {
  console.error('[shadowbox] Execution failed:', result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);
