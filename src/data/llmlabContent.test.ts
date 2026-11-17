import test from 'node:test';
import assert from 'node:assert/strict';
import {
  calculateFreedRamMb,
  ENGINE_SPECS,
  formatRamGb,
  getInstallCommand,
  INITIAL_FOCUS_PROCESSES,
  LOGO_LINES,
  SITE_COPY,
} from './llmlabContent';

test('install commands target tatarhost/llmLab repository', () => {
  const ps = getInstallCommand('powershell');
  assert.ok(ps.includes('https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1'));

  const cmd = getInstallCommand('cmd');
  assert.ok(cmd.startsWith('powershell -NoProfile -ExecutionPolicy Bypass'));
  assert.ok(cmd.includes('tatarhost/llmLab'));

  const src = getInstallCommand('source');
  assert.ok(src.includes('cargo build --release --target x86_64-pc-windows-msvc'));
});

test('calculateFreedRamMb sums only non-kept processes', () => {
  const freed = calculateFreedRamMb(INITIAL_FOCUS_PROCESSES);
  // chrome (2840) + steam (940) + spotify (410) = 4190 MB
  assert.equal(freed, 4190);
  assert.equal(formatRamGb(4190), '4.09 GB');
});

test('bilingual content and logo lines stay complete', () => {
  assert.equal(LOGO_LINES.length, 18);
  assert.equal(SITE_COPY.ru.capabilities.items.length, 4);
  assert.equal(SITE_COPY.en.capabilities.items.length, 4);
  assert.equal(ENGINE_SPECS.length, 4);
});
