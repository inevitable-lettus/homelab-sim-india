// The real terminal: a small Linux system running in this browser tab. It has
// no path to the internet. The page only ever reads the files a mission names.
//
// The interface is engine-neutral. The one engine behind it is v86, loaded
// from /guest/ together with the state it was built against.

export interface GuestManifest {
  engine: string;
  engine_version: string;
  alpine: string;
  kernel: string;
  memory_mb: number;
  first_download_bytes: number;
  rootfs_files: number;
  packages: { name: string; origin: string; licence: string }[];
}

export interface TerminalHost {
  writeInput(data: string): void;
  onOutput(listener: (data: Uint8Array) => void): void;
  /** Null when the file does not exist. */
  readFile(path: string): Promise<Uint8Array | null>;
  writeFile(path: string, data: Uint8Array): Promise<void>;
  dispose(): void;
}

const guestUrl = (file: string) => new URL(`${import.meta.env.BASE_URL}guest/${file}`, location.href).href;

let manifest: Promise<GuestManifest | null> | undefined;

/** Null when this build of the site has no guest image. */
export function loadGuestManifest(): Promise<GuestManifest | null> {
  manifest ??= fetch(guestUrl('manifest.json'))
    .then((response) => (response.ok ? (response.json() as Promise<GuestManifest>) : null))
    .catch(() => null);
  return manifest;
}

// The parts of v86 this file uses.
interface V86 {
  add_listener(event: string, listener: (value: never) => void): void;
  serial0_send(data: string): void;
  read_file(path: string): Promise<Uint8Array>;
  create_file(path: string, data: Uint8Array): Promise<void>;
  destroy(): Promise<void>;
}
interface V86Module {
  V86: new (options: Record<string, unknown>) => V86;
}

/** Start the guest from its saved state. `onProgress` gets 0 to 1 while it downloads. */
export async function bootGuest(guest: GuestManifest, onProgress: (fraction: number) => void): Promise<TerminalHost> {
  const { V86 } = (await import(/* @vite-ignore */ guestUrl('libv86.mjs'))) as V86Module;
  const emulator = new V86({
    wasm_path: guestUrl('v86.wasm'),
    memory_size: guest.memory_mb * 1024 * 1024,
    vga_memory_size: 2 * 1024 * 1024,
    autostart: true,
    disable_keyboard: true,
    disable_mouse: true,
    filesystem: { baseurl: guestUrl('rootfs/'), basefs: guestUrl('fs.json') },
    initial_state: { url: guestUrl('state.bin.zst') },
  });
  emulator.add_listener('download-progress', ((event: { loaded?: number; total?: number }) => {
    if (event.total) onProgress(Math.min(1, (event.loaded ?? 0) / event.total));
  }) as (value: never) => void);

  const listeners: ((data: Uint8Array) => void)[] = [];
  // Bytes arrive one at a time. Hand them on once per frame.
  let pending: number[] = [];
  let queued = false;
  emulator.add_listener('serial0-output-byte', ((byte: number) => {
    pending.push(byte);
    if (queued) return;
    queued = true;
    setTimeout(() => {
      queued = false;
      const data = Uint8Array.from(pending);
      pending = [];
      listeners.forEach((listener) => listener(data));
    }, 16);
  }) as (value: never) => void);

  await new Promise<void>((resolve) => emulator.add_listener('emulator-started', (() => resolve()) as (value: never) => void));

  return {
    writeInput: (data) => emulator.serial0_send(data),
    onOutput: (listener) => void listeners.push(listener),
    readFile: (path) => emulator.read_file(path).catch(() => null),
    writeFile: (path, data) => emulator.create_file(path, data),
    dispose: () => void emulator.destroy(),
  };
}
