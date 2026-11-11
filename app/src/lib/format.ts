// Turns numbers the core computed into words. Nothing here decides anything:
// a missing number stays missing and is shown as "unknown".

const trim = (value: number, digits: number) => String(Number(value.toFixed(digits)));

/** Nanoseconds as a short time: `25 µs`, `4.1 ms`, `1.2 s`. */
export function time(ns: number | null | undefined): string {
  if (ns === null || ns === undefined) return 'unknown';
  if (ns < 1_000) return `${ns} ns`;
  if (ns < 1_000_000) return `${trim(ns / 1_000, ns < 10_000 ? 1 : 0)} µs`;
  if (ns < 1_000_000_000) return `${trim(ns / 1_000_000, ns < 100_000_000 ? 1 : 0)} ms`;
  return `${trim(ns / 1_000_000_000, 1)} s`;
}

/** Milliseconds as a duration a person would say: `12 min 51 s`. */
export function long(ms: number | null | undefined): string {
  if (ms === null || ms === undefined) return 'unknown';
  const seconds = Math.round(ms / 1000);
  if (seconds < 60) return `${seconds} s`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes} min ${seconds % 60} s`;
  return `${Math.floor(minutes / 60)} h ${minutes % 60} min`;
}

export function rate(bps: number | null | undefined): string {
  if (bps === null || bps === undefined) return 'unknown';
  if (bps >= 1_000_000_000) return `${trim(bps / 1_000_000_000, 1)} Gbit/s`;
  if (bps >= 1_000_000) return `${trim(bps / 1_000_000, 1)} Mbit/s`;
  return `${trim(bps / 1_000, 0)} kbit/s`;
}

export function size(bytes: number | null | undefined): string {
  if (bytes === null || bytes === undefined) return 'unknown';
  if (bytes >= 1_000_000_000) return `${trim(bytes / 1_000_000_000, 2)} GB`;
  if (bytes >= 1_000_000) return `${trim(bytes / 1_000_000, 1)} MB`;
  if (bytes >= 1_000) return `${trim(bytes / 1_000, 1)} kB`;
  return `${bytes} bytes`;
}

export function distance(metres: number | null | undefined): string {
  if (metres === null || metres === undefined) return 'unknown';
  return metres >= 1000 ? `${trim(metres / 1000, 0)} km` : `${metres} m`;
}

/** `192.168.1.20:8096` without the port, for tight labels. */
export function host(sock: string): string {
  const cut = sock.lastIndexOf(':');
  return cut > 0 && !sock.endsWith(']') ? sock.slice(0, cut) : sock;
}
