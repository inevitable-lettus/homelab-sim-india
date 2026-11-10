// Phase 1 spike, throwaway. Boots Alpine in v86, lets the guest write a real
// WireGuard config, and reads it back on the host with read_file.
import { V86 } from 'v86';
import wasmUrl from 'v86/build/v86.wasm?url';

const params = new URLSearchParams(location.search);
const stateMb = params.get('state');
const memMb = Number(stateMb ?? params.get('mem') ?? 256);
const cache = params.get('cache') ?? 'loose';
const auto = params.has('auto');

const $ = (id) => document.getElementById(id);
const term = $('term');
const status = $('status');

const t0 = performance.now();
const seconds = () => Number(((performance.now() - t0) / 1000).toFixed(2));

const options = {
  wasm_path: wasmUrl,
  memory_size: memMb * 1024 * 1024,
  vga_memory_size: 2 * 1024 * 1024,
  autostart: true,
  filesystem: {
    baseurl: './images/alpine-rootfs-flat',
    basefs: './images/alpine-fs.json',
  },
};
if (stateMb) {
  options.initial_state = { url: `./images/alpine-state-${stateMb}.bin.zst` };
} else {
  options.bios = { url: './.v86/bios/seabios.bin' };
  options.vga_bios = { url: './.v86/bios/vgabios.bin' };
  options.bzimage_initrd_from_filesystem = true;
  options.cmdline = `rw root=host9p rootfstype=9p rootflags=trans=virtio,cache=${cache} modules=virtio_pci tsc=reliable init_on_free=on`;
}
const emulator = new V86(options);
window.emulator = emulator;

// Basic terminal: enough for a shell prompt. Phase 7 needs a real emulator
// (xterm.js) for nano and friends.
let text = '';
let screen = '';
let escape = '';
const PROMPT = /:~# (\x1b\[6n)?$/;
let waiters = [];
let paintQueued = false;
emulator.add_listener('serial0-output-byte', (byte) => {
  const c = String.fromCharCode(byte);
  text += c;
  if (escape) {
    escape += c;
    if (/[A-Za-z]$/.test(escape) && escape.length > 1) escape = '';
  } else if (c === '\x1b') escape = c;
  else if (c === '\b') screen = screen.slice(0, -1);
  else if (c !== '\r' && c !== '\x07') screen += c;
  if (!paintQueued) {
    paintQueued = true;
    requestAnimationFrame(() => {
      paintQueued = false;
      term.textContent = screen.slice(-6000);
      term.scrollTop = term.scrollHeight;
    });
  }
  if (PROMPT.test(text)) {
    const w = waiters;
    waiters = [];
    w.forEach((resolve) => resolve());
  }
});
term.addEventListener('keydown', (e) => {
  let s = e.key;
  if (s === 'Enter') s = '\r';
  else if (s === 'Backspace') s = '\x7f';
  else if (s === 'Tab') s = '\t';
  else if (e.ctrlKey && s.length === 1) s = String.fromCharCode(s.toUpperCase().charCodeAt(0) - 64);
  else if (s.length !== 1) return;
  e.preventDefault();
  emulator.serial0_send(s);
});

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
const WRITE = `umask 077; printf '[Interface]\\nPrivateKey = %s\\nAddress = 10.8.0.2/24\\n' "$(wg genkey)" > /root/wg0.conf`;

$('write').onclick = () => run(WRITE);
$('read').onclick = async () => {
  $('host').textContent = await read('/root/wg0.conf');
};

const result = { mode: stateMb ? `state-${stateMb}` : 'cold', memMb, cache, crossOriginIsolated };
window.__result = null;

async function main() {
  if (stateMb) {
    // A restored state is already sitting at the prompt and prints nothing.
    await new Promise((resolve) => emulator.add_listener('emulator-started', resolve));
    result.startedS = seconds();
    await run('');
  } else {
    await prompt();
  }
  result.promptS = seconds();
  status.textContent = `prompt after ${result.promptS} s (${result.mode}, ${memMb} MB)`;
  if (!auto) return;

  const tw = performance.now();
  await run(WRITE);
  result.readBeforeSync = await read('/root/wg0.conf');
  await run('sync');
  result.readAfterSync = await read('/root/wg0.conf');
  result.writeReadS = Number(((performance.now() - tw) / 1000).toFixed(2));
  // A larger write, to see whether cache=loose delays what the host sees.
  await run('head -c 300000 /dev/urandom | base64 > /root/big.txt; wc -c < /root/big.txt');
  result.guestBigBytes = Number(/(\d+)\s+\S*:~# /.exec(text.slice(-200))?.[1] ?? -1);
  result.hostBigBytesBeforeSync = (await read('/root/big.txt')).length;
  await run('sync');
  result.hostBigBytesAfterSync = (await read('/root/big.txt')).length;
  await run('wg --version');
  result.wgVersion = /wireguard-tools v[\d.]+/.exec(text.slice(-300))?.[0] ?? null;
  $('host').textContent = result.readAfterSync;
  window.__result = result;
  status.textContent += ' · auto script done';
}
main();
