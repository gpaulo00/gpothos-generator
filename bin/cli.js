#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

const BIN_NAME = 'gpothos-generator';
const binDir = path.join(__dirname);
const binPath = path.join(binDir, process.platform === 'win32' ? `${BIN_NAME}.exe` : BIN_NAME);

const child = spawn(binPath, process.argv.slice(2), {
  stdio: 'inherit',
});

child.on('close', (code) => {
  process.exit(code);
});

child.on('error', (err) => {
  console.error(`Failed to start subprocess: ${err}`);
  console.error(`Binary path: ${binPath}`);
  process.exit(1);
});
