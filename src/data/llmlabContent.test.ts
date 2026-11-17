import test from 'node:test';
import assert from 'node:assert/strict';
import {
  calculateFreedRamMb,
  ENGINE_SPECS,
  formatRamGb,
  getInstallCommand,
  HF_MODEL_SAMPLES,
  INITIAL_FOCUS_PROCESSES,
  LOGO_LINES,
  SITE_COPY,
} from './llmlabContent';

test('install commands target tatarhost/llmLab repository across Windows, Linux, and Termux', () => {
  const ps = getInstallCommand('powershell');
  assert.ok(ps.includes('https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1'));

  const termux = getInstallCommand('termux');
  assert.ok(termux.includes('https://raw.githubusercontent.com/tatarhost/llmLab/main/install.sh'));

  const linux = getInstallCommand('linux');
  assert.ok(linux.includes('install.sh'));

  const cmd = getInstallCommand('cmd');
  assert.ok(cmd.startsWith('powershell -NoProfile -ExecutionPolicy Bypass'));
  assert.ok(cmd.includes('tatarhost/llmLab'));

  const src = getInstallCommand('source');
  assert.ok(src.includes('cargo build --release'));
});

test('calculateFreedRamMb sums only non-kept processes', () => {
  const freed = calculateFreedRamMb(INITIAL_FOCUS_PROCESSES);
  // chrome (2840) + steam (940) + spotify (410) = 4190 MB
  assert.equal(freed, 4190);
  assert.equal(formatRamGb(4190), '4.09 GB');
});

test('bilingual content, HF catalog, and logo lines stay complete', () => {
  assert.equal(LOGO_LINES.length, 18);
  assert.equal(SITE_COPY.ru.capabilities.items.length, 4);
  assert.equal(SITE_COPY.en.capabilities.items.length, 4);
  assert.equal(Object.keys(SITE_COPY.ru.simulator.tabs).length, 5);
  assert.equal(Object.keys(SITE_COPY.en.simulator.tabs).length, 5);
  assert.equal(ENGINE_SPECS.length, 4);
  assert.ok(HF_MODEL_SAMPLES.length >= 4);
  assert.ok(HF_MODEL_SAMPLES.some((m) => m.fitsPc));
  assert.ok(HF_MODEL_SAMPLES.some((m) => !m.fitsPc));
});
