// Key making for the lesson. The random bytes come from the browser. The
// public key is worked out by the core, the same way `wg pubkey` does it.
import { publicKeyOf } from './sim';

function base64(bytes: Uint8Array): string {
  let text = '';
  for (const byte of bytes) text += String.fromCharCode(byte);
  return btoa(text);
}

/** A new private key, as `wg genkey` prints it. */
export function newPrivateKey(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(32));
  // The same clamping `wg genkey` applies.
  bytes[0]! &= 248;
  bytes[31] = (bytes[31]! & 127) | 64;
  return base64(bytes);
}

export interface KeyPair {
  privateKey: string;
  publicKey: string;
}

export function newKeyPair(): KeyPair | null {
  const privateKey = newPrivateKey();
  const publicKey = publicKeyOf(privateKey);
  return publicKey ? { privateKey, publicKey } : null;
}

/** Put a private key into a config: on its `PrivateKey` line, or under `[Interface]`. */
export function withPrivateKey(conf: string, privateKey: string): string {
  const line = /^([ \t]*PrivateKey[ \t]*=).*$/im;
  if (line.test(conf)) return conf.replace(line, `$1 ${privateKey}`);
  const section = /^[ \t]*\[Interface\][ \t]*$/im;
  if (section.test(conf)) return conf.replace(section, (found) => `${found}\nPrivateKey = ${privateKey}`);
  return `[Interface]\nPrivateKey = ${privateKey}\n${conf}`;
}
