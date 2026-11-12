#!/usr/bin/env node
// Boots the guest once under Node, records what is installed in it, and saves
// a state image so the browser never has to cold boot. Run by build.sh.
import child_process from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import url from 'node:url';
import { V86 } from 'v86';

const here = path.dirname(url.fileURLToPath(import.meta.url));
const out = process.env.GUEST_OUT;
const bios = process.env.GUEST_BIOS;
const memoryMb = Number(process.env.GUEST_MEMORY_MB ?? 128);
if (!out || !bios) throw new Error('GUEST_OUT and GUEST_BIOS are required');
const v86Package = JSON.parse(fs.readFileSync(path.join(here, '../../node_modules/v86/package.json'), 'utf8'));
if (!v86Package.version.includes(process.env.V86_COMMIT.slice(0, 7))) {
  throw new Error(`versions.env pins v86 ${process.env.V86_COMMIT}, but node_modules has ${v86Package.version}`);
}

const emulator = new V86({
  wasm_path: path.join(out, 'v86.wasm'),
  bios: { url: path.join(bios, 'seabios.bin') },
  vga_bios: { url: path.join(bios, 'vgabios.bin') },
  autostart: true,
  memory_size: memoryMb * 1024 * 1024,
  vga_memory_size: 2 * 1024 * 1024,
  network_relay_url: '<UNUSED>',
  bzimage_initrd_from_filesystem: true,
  // cache=loose lets the page read a file the moment the guest has written it.
  cmdline: 'rw root=host9p rootfstype=9p rootflags=trans=virtio,cache=loose modules=virtio_pci tsc=reliable init_on_free=on',
  filesystem: { baseurl: path.join(out, 'rootfs'), basefs: path.join(out, 'fs.json') },
});

let text = '';
let waiters = [];
// The shell prints its prompt, then asks the terminal where the cursor is.
const PROMPT = /:~# (\x1b\[6n)?$/;
emulator.add_listener('serial0-output-byte', (byte) => {
  text += String.fromCharCode(byte);
  if (PROMPT.test(text)) waiters.splice(0).forEach((resolve) => resolve());
});
const prompt = () => new Promise((resolve) => waiters.push(resolve));
const run = async (command) => {
  const done = prompt();
  emulator.serial0_send(`${command}\n`);
  await done;
};
const read = async (file) => new TextDecoder().decode(await emulator.read_file(file));

const started = Date.now();
await prompt();
console.log(`cold boot to prompt: ${((Date.now() - started) / 1000).toFixed(1)} s`);

await run('hostname server');
await run('apk list -I 2>/dev/null | sort > /root/.packages; uname -r > /root/.kernel; cat /etc/alpine-release > /root/.release; sync');
const packages = (await read('/root/.packages'))
  .split('\n')
  .map((line) => /^(\S+) \S+ \{(\S+)\} \((.*)\) \[installed\]$/.exec(line.trim()))
  .filter(Boolean)
  .map(([, name, origin, licence]) => ({ name, origin, licence }));
const kernel = (await read('/root/.kernel')).trim();
const release = (await read('/root/.release')).trim();
if (packages.length === 0) throw new Error('could not read the package list from the guest');
await run('rm -f /root/.packages /root/.kernel /root/.release /root/.ash_history; sync; echo 3 >/proc/sys/vm/drop_caches; clear');

await new Promise((resolve) => setTimeout(resolve, 5000));
const statePath = path.join(out, 'state.bin');
fs.writeFileSync(statePath, new Uint8Array(await emulator.save_state()));
child_process.execFileSync('zstd', ['-19', '-q', '-f', '--rm', statePath]);
await emulator.destroy();

const size = (file) => fs.statSync(path.join(out, file)).size;
const rootfs = fs.readdirSync(path.join(out, 'rootfs'));
const manifest = {
  engine: 'v86',
  engine_version: v86Package.version,
  engine_commit: process.env.V86_COMMIT,
  alpine: release,
  alpine_image: `i386/alpine:${process.env.ALPINE_VERSION}`,
  kernel,
  memory_mb: memoryMb,
  // What a learner downloads before the first prompt. Root filesystem files
  // are fetched one by one, only when a command needs them.
  first_download_bytes: size('state.bin.zst') + size('v86.wasm') + size('libv86.mjs') + size('fs.json'),
  state_bytes: size('state.bin.zst'),
  rootfs_files: rootfs.length,
  rootfs_bytes: rootfs.reduce((sum, file) => sum + size(path.join('rootfs', file)), 0),
  packages,
};
fs.writeFileSync(path.join(out, 'manifest.json'), `${JSON.stringify(manifest, null, 1)}\n`);

const sources = `# Source code for the terminal's Linux image

The terminal on this site runs a small Linux system in your browser. It is
Alpine Linux ${release} for 32-bit x86, kernel ${kernel}, built from the
official \`${manifest.alpine_image}\` image with the packages below. Many of
them are under the GNU GPL or LGPL.

## Written offer

For three years from the date this image was published, the maintainers of
this project will give anyone the complete corresponding source code of
every GPL and LGPL program in this image, at no charge, on request through
the project's issue tracker.

## Where the sources already are

Nothing in the image is modified. Every package is built by Alpine Linux from
the recipe in its \`aports\` repository, branch \`${release.split('.').slice(0, 2).join('.')}-stable\`:
https://gitlab.alpinelinux.org/alpine/aports . The recipe for a package is
the \`APKBUILD\` file in the folder named in the "origin" column, and it names
the upstream source archive and every patch applied.

The image itself is rebuilt from scratch by \`tools/guest/\` in this project's
repository. The emulator is v86 ${v86Package.version} (BSD-2-Clause),
https://github.com/copy/v86 at commit ${process.env.V86_COMMIT}.

## Packages

| Package and version | Origin | Licence |
|---|---|---|
${packages.map((p) => `| ${p.name} | ${p.origin} | ${p.licence} |`).join('\n')}
`;
fs.writeFileSync(path.join(out, 'SOURCES.md'), sources);
console.log(`state ${manifest.state_bytes} B, first download ${manifest.first_download_bytes} B, ${packages.length} packages, kernel ${kernel}`);
process.exit(0);
