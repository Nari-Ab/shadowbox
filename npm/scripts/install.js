#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const REPO = 'Nari-Ab/shadowbox';
const VERSION = 'v0.1.0';

const platform = process.platform;
const arch = process.arch;

let assetName = '';
if (platform === 'darwin') {
  assetName = arch === 'arm64' ? 'shadowbox-macos-arm64.tar.gz' : 'shadowbox-macos-x86_64.tar.gz';
} else if (platform === 'linux') {
  assetName = arch === 'arm64' ? 'shadowbox-linux-arm64.tar.gz' : 'shadowbox-linux-x86_64.tar.gz';
} else {
  console.warn(`[shadowbox] Warning: Platform ${platform}-${arch} is not yet officially supported.`);
  process.exit(0);
}

const binDir = path.join(__dirname, '..', 'bin');
const targetBinary = path.join(binDir, 'shadowbox-bin');

if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

// Download tarball or direct binary
const downloadUrl = `https://github.com/${REPO}/releases/download/${VERSION}/${assetName}`;
const fallbackUrl = `https://github.com/${REPO}/releases/download/${VERSION}/shadowbox`;

function download(url, dest, callback) {
  https.get(url, (res) => {
    if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
      return download(res.headers.location, dest, callback);
    }
    if (res.statusCode !== 200) {
      return callback(new Error(`Failed with status ${res.statusCode}`));
    }
    const file = fs.createWriteStream(dest);
    res.pipe(file);
    file.on('finish', () => file.close(callback));
  }).on('error', callback);
}

const tarballPath = path.join(binDir, 'download.tar.gz');

console.log(`[shadowbox] Fetching native pre-built binary for ${platform}-${arch}...`);

download(downloadUrl, tarballPath, (err) => {
  if (err) {
    // Try fallback direct binary
    download(fallbackUrl, targetBinary, (err2) => {
      if (err2) {
        console.warn(`[shadowbox] Pre-built binary not downloaded. Will build from source if cargo is available.`);
        return;
      }
      fs.chmodSync(targetBinary, 0o755);
      console.log(`[shadowbox] Ready!`);
    });
    return;
  }

  try {
    execSync(`tar -xzf "${tarballPath}" -C "${binDir}"`);
    fs.unlinkSync(tarballPath);
    const extracted = path.join(binDir, 'shadowbox');
    if (fs.existsSync(extracted)) {
      if (fs.existsSync(targetBinary)) fs.unlinkSync(targetBinary);
      fs.renameSync(extracted, targetBinary);
    }
    fs.chmodSync(targetBinary, 0o755);
    console.log(`[shadowbox] Binary installed successfully.`);
  } catch (e) {
    console.warn(`[shadowbox] Error extracting binary: ${e.message}`);
  }
});
