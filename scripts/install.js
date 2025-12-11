const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const packageJson = require('../package.json');
const version = packageJson.version;
const REPO = 'gpaulo00/gpothos-generator'; // Adjust based on your username/repo
const BIN_NAME = 'gpothos-generator';

const supportedPlatforms = {
  'linux-x64': 'gpothos-linux-amd64',
  'linux-arm64': 'gpothos-linux-arm64',
  'darwin-x64': 'gpothos-darwin-amd64',
  'darwin-arm64': 'gpothos-darwin-arm64',
  'win32-x64': 'gpothos-windows-amd64.exe',
};

const platformKey = `${process.platform}-${process.arch}`;
const artifactName = supportedPlatforms[platformKey];

if (!artifactName) {
  console.error(`Unsupported platform: ${platformKey}`);
  process.exit(1);
}

const binDir = path.join(__dirname, '..', 'bin');
const binPath = path.join(binDir, process.platform === 'win32' ? `${BIN_NAME}.exe` : BIN_NAME);
const url = `https://github.com/${REPO}/releases/download/v${version}/${artifactName}`;

if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

console.log(`Downloading ${artifactName} from ${url}...`);

const file = fs.createWriteStream(binPath);

https.get(url, (response) => {
  if (response.statusCode === 302 || response.statusCode === 301) {
    // Handle redirect
    https.get(response.headers.location, (response) => {
      download(response);
    });
  } else {
    download(response);
  }
}).on('error', (err) => {
    fs.unlink(binPath, () => {});
    console.error(`Error downloading binary: ${err.message}`);
    process.exit(1);
});

function download(response) {
  if (response.statusCode !== 200) {
    console.error(`Failed to download binary. Status Code: ${response.statusCode}`);
    process.exit(1);
  }

  response.pipe(file);

  file.on('finish', () => {
    file.close(() => {
      console.log('Download complete.');
      if (process.platform !== 'win32') {
        fs.chmodSync(binPath, 0o755);
      }
    });
  });
}
