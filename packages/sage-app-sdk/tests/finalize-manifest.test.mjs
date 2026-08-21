import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { afterEach, test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { bech32m } from 'bech32';

const cliPath = fileURLToPath(
  new URL('../cli/finalize-manifest.mjs', import.meta.url),
);
const tempDirs = [];

afterEach(() => {
  for (const tempDir of tempDirs.splice(0)) {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

function runFinalizer(donationAddress) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'sage-manifest-test-'));
  tempDirs.push(tempDir);

  const sourcePath = path.join(tempDir, 'sage-manifest.json');
  const distDir = path.join(tempDir, 'dist');
  const outPath = path.join(distDir, 'sage-manifest.json');
  const manifest = {
    manifestVersion: 0,
    name: 'Test app',
    version: '1.0.0',
    donation: { address: donationAddress },
  };

  fs.mkdirSync(distDir);
  fs.writeFileSync(path.join(distDir, 'index.html'), '<!doctype html>');
  fs.writeFileSync(sourcePath, JSON.stringify(manifest));

  const result = spawnSync(
    process.execPath,
    [cliPath, 'finalize-manifest', '--source', sourcePath, '--dist', distDir],
    { encoding: 'utf8' },
  );

  return { result, outPath };
}

test('accepts a valid XCH donation address', () => {
  const { result, outPath } = runFinalizer(
    'xch10hwpheqwv4m4dzuqc5hp8se3p0chkrkvmtf8m6d830wfagg9p22sxd82v7',
  );

  assert.equal(result.status, 0, result.stderr);
  assert.equal(fs.existsSync(outPath), true);
});

test('rejects a valid TXCH donation address', () => {
  const { result, outPath } = runFinalizer(
    'txch19hutewzq3z4l6y3fsw5laatre79tuz5p43jlvag0yz466xx9l7vs4vnpem',
  );

  assert.equal(result.status, 1);
  assert.match(result.stderr, /donation\.address must be a valid xch address/);
  assert.equal(fs.existsSync(outPath), false);
});

test('rejects a donation address with an invalid checksum', () => {
  const { result, outPath } = runFinalizer(
    'xch10hwpheqwv4m4dzuqc5hp8se3p0chkrkvmtf8m6d830wfagg9p22sxd82v8',
  );

  assert.equal(result.status, 1);
  assert.match(result.stderr, /donation\.address must be a valid/);
  assert.equal(fs.existsSync(outPath), false);
});

test('rejects a valid Bech32m address with a non-wallet prefix', () => {
  const address = bech32m.encode('nft', bech32m.toWords(Buffer.alloc(32, 1)));
  const { result } = runFinalizer(address);

  assert.equal(result.status, 1);
  assert.match(result.stderr, /donation\.address must be a valid/);
});

test('rejects a valid XCH address with the wrong payload length', () => {
  const address = bech32m.encode('xch', bech32m.toWords(Buffer.alloc(31, 1)));
  const { result } = runFinalizer(address);

  assert.equal(result.status, 1);
  assert.match(result.stderr, /donation\.address must be a valid/);
});
