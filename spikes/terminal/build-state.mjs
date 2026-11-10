#!/usr/bin/env node
// Phase 1 spike, throwaway. Boots the Alpine image under Node, proves that a
// guest-written file is visible to read_file, then saves a state image.
// Usage: MEM_MB=256 node spikes/terminal/build-state.mjs
import fs from 'node:fs';
import path from 'node:path';
import url from 'node:url';
import child_process from 'node:child_process';
import { V86 } from 'v86';

const here = path.dirname(url.fileURLToPath(import.meta.url));
const MEM_MB = Number(process.env.MEM_MB ?? 256);
const CACHE = process.env.CACHE ?? 'loose';
const out = path.join(here, `images/alpine-state-${MEM_MB}.bin`);
const v86Build = path.join(here, '../../node_modules/v86/build');

const t0 = Date.now();
const emulator = new V86({
  wasm_path: path.join(v86Build, 'v86.wasm'),
  bios: { url: path.join(here, '.v86/bios/seabios.bin') },
  vga_bios: { url: path.join(here, '.v86/bios/vgabios.bin') },
  autostart: true,
  memory_size: MEM_MB * 1024 * 1024,
  vga_memory_size: 2 * 1024 * 1024,
  network_relay_url: '<UNUSED>',
  bzimage_initrd_from_filesystem: true,
  cmdline: `rw root=host9p rootfstype=9p rootflags=trans=virtio,cache=${CACHE} modules=virtio_pci tsc=reliable init_on_free=on`,
  filesystem: {
    baseurl: path.join(here, 'images/alpine-rootfs-flat'),
    basefs: path.join(here, 'images/alpine-fs.json'),
  },
});

let text = '';
const waiters = [];
emulator.add_listener('serial0-output-byte', (byte) => {
  text += String.fromCharCode(byte);
  if (process.env.VERBOSE) process.stdout.write(String.fromCharCode(byte));
  // The shell prints its prompt, then asks the terminal for the cursor position.
  if (PROMPT.test(text)) waiters.splice(0).forEach((resolve) => resolve());
});
// Hostname is "(none)" in this image, not "localhost" as in v86's own script.
const PROMPT = /:~# (\x1b\[6n)?$/;
const prompt = () => new Promise((resolve) => waiters.push(resolve));
const run = async (cmd) => {
  const p = prompt();
  emulator.serial0_send(cmd + '\n');
  await p;
};
const read = async (file) => {
  try {
    return new TextDecoder().decode(await emulator.read_file(file));
  } catch (e) {
    return `<<read_file failed: ${e.message}>>`;
  }
};

await prompt();
console.log(`cold boot to prompt (node): ${((Date.now() - t0) / 1000).toFixed(1)} s, mem ${MEM_MB} MB, cache=${CACHE}`);

// Guest writes a real WireGuard config. Read it before and after sync.
await run(
  `umask 077; printf '[Interface]\\nPrivateKey = %s\\nAddress = 10.8.0.2/24\\n' "$(wg genkey)" > /root/wg0.conf`,
);
const beforeSync = await read('/root/wg0.conf');
await run('sync');
const afterSync = await read('/root/wg0.conf');
console.log('read_file before sync:', JSON.stringify(beforeSync));
console.log('read_file after sync: ', JSON.stringify(afterSync));
await run('rm /root/wg0.conf; sync; echo 3 >/proc/sys/vm/drop_caches');

await new Promise((r) => setTimeout(r, 5000));
const state = await emulator.save_state();
fs.writeFileSync(out, new Uint8Array(state));
child_process.execFileSync('zstd', ['-19', '-f', '--rm', out], { stdio: 'inherit' });
console.log(`state: ${out}.zst ${fs.statSync(out + '.zst').size} bytes`);
await emulator.destroy();
process.exit(0);
