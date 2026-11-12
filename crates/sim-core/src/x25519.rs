//! Curve25519 key agreement, enough to turn a WireGuard private key into its
//! public key, as `wg pubkey` does. Integer arithmetic only.
//!
//! The field arithmetic follows TweetNaCl (public domain): a field element is
//! sixteen 16-bit limbs held in `i64`. It is checked against RFC 7748.

type Gf = [i64; 16];

const A24: Gf = [0xDB41, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const BASE_POINT: [u8; 32] =
    [9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

fn carry(o: &mut Gf) {
    for i in 0..16 {
        o[i] += 1 << 16;
        let c = o[i] >> 16;
        if i < 15 {
            o[i + 1] += c - 1;
        } else {
            o[0] += 38 * (c - 1);
        }
        o[i] -= c << 16;
    }
}

/// Swap `p` and `q` when `bit` is 1.
fn swap(p: &mut Gf, q: &mut Gf, bit: i64) {
    let mask = !(bit - 1);
    for i in 0..16 {
        let t = mask & (p[i] ^ q[i]);
        p[i] ^= t;
        q[i] ^= t;
    }
}

fn pack(n: &Gf) -> [u8; 32] {
    let mut t = *n;
    carry(&mut t);
    carry(&mut t);
    carry(&mut t);
    for _ in 0..2 {
        let mut m: Gf = [0; 16];
        m[0] = t[0] - 0xffed;
        for i in 1..15 {
            m[i] = t[i] - 0xffff - ((m[i - 1] >> 16) & 1);
            m[i - 1] &= 0xffff;
        }
        m[15] = t[15] - 0x7fff - ((m[14] >> 16) & 1);
        let borrow = (m[15] >> 16) & 1;
        m[14] &= 0xffff;
        swap(&mut t, &mut m, 1 - borrow);
    }
    let mut out = [0u8; 32];
    for i in 0..16 {
        out[2 * i] = (t[i] & 0xff) as u8;
        out[2 * i + 1] = ((t[i] >> 8) & 0xff) as u8;
    }
    out
}

fn unpack(n: &[u8; 32]) -> Gf {
    let mut o: Gf = [0; 16];
    for i in 0..16 {
        o[i] = i64::from(n[2 * i]) + (i64::from(n[2 * i + 1]) << 8);
    }
    o[15] &= 0x7fff;
    o
}

fn add(a: &Gf, b: &Gf) -> Gf {
    let mut o: Gf = [0; 16];
    for i in 0..16 {
        o[i] = a[i] + b[i];
    }
    o
}

fn sub(a: &Gf, b: &Gf) -> Gf {
    let mut o: Gf = [0; 16];
    for i in 0..16 {
        o[i] = a[i] - b[i];
    }
    o
}

fn mul(a: &Gf, b: &Gf) -> Gf {
    let mut t = [0i64; 31];
    for i in 0..16 {
        for j in 0..16 {
            t[i + j] += a[i] * b[j];
        }
    }
    for i in 0..15 {
        t[i] += 38 * t[i + 16];
    }
    let mut o: Gf = [0; 16];
    o.copy_from_slice(&t[..16]);
    carry(&mut o);
    carry(&mut o);
    o
}

fn square(a: &Gf) -> Gf {
    mul(a, a)
}

fn invert(i: &Gf) -> Gf {
    let mut c = *i;
    for a in (0..=253).rev() {
        c = square(&c);
        if a != 2 && a != 4 {
            c = mul(&c, i);
        }
    }
    c
}

/// X25519 as in RFC 7748: the scalar is clamped, then multiplied by the point `u`.
pub fn x25519(scalar: &[u8; 32], u: &[u8; 32]) -> [u8; 32] {
    let mut z = *scalar;
    z[31] = (z[31] & 127) | 64;
    z[0] &= 248;
    let x = unpack(u);
    let mut a: Gf = [0; 16];
    let mut b = x;
    let mut c: Gf = [0; 16];
    let mut d: Gf = [0; 16];
    a[0] = 1;
    d[0] = 1;
    for i in (0..=254usize).rev() {
        let bit = i64::from((z[i >> 3] >> (i & 7)) & 1);
        swap(&mut a, &mut b, bit);
        swap(&mut c, &mut d, bit);
        let mut e = add(&a, &c);
        a = sub(&a, &c);
        c = add(&b, &d);
        b = sub(&b, &d);
        d = square(&e);
        let f = square(&a);
        a = mul(&c, &a);
        c = mul(&b, &e);
        e = add(&a, &c);
        a = sub(&a, &c);
        b = square(&a);
        c = sub(&d, &f);
        a = mul(&c, &A24);
        a = add(&a, &d);
        c = mul(&c, &a);
        a = mul(&d, &f);
        d = mul(&b, &x);
        b = square(&e);
        swap(&mut a, &mut b, bit);
        swap(&mut c, &mut d, bit);
    }
    pack(&mul(&a, &invert(&c)))
}

/// The public key for a private key.
pub fn public_key(private: &[u8; 32]) -> [u8; 32] {
    x25519(private, &BASE_POINT)
}

// ----------------------------------------------------------------- base64

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// A 32-byte key as WireGuard writes it: 44 characters of base64, ending in `=`.
pub fn encode_key(key: &[u8; 32]) -> String {
    let mut out = String::with_capacity(44);
    for chunk in key.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[n as usize & 63] as char } else { '=' });
    }
    out
}

/// Reads a WireGuard key. `None` unless it is exactly 32 bytes of standard base64.
pub fn decode_key(text: &str) -> Option<[u8; 32]> {
    let bytes = text.as_bytes();
    if bytes.len() != 44 || bytes[43] != b'=' {
        return None;
    }
    let mut out = [0u8; 32];
    let mut acc: u32 = 0;
    let mut bits = 0;
    let mut n = 0;
    for &byte in &bytes[..43] {
        let value = ALPHABET.iter().position(|&c| c == byte)? as u32;
        acc = (acc << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out[n] = ((acc >> bits) & 0xff) as u8;
            n += 1;
        }
    }
    // 43 characters carry 258 bits: 32 bytes and two bits that must be zero.
    if n != 32 || acc & 0b11 != 0 {
        return None;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> [u8; 32] {
        let mut out = [0u8; 32];
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&text[2 * i..2 * i + 2], 16).expect("hex");
        }
        out
    }

    #[test]
    fn rfc_7748_scalar_multiplication_vectors() {
        // RFC 7748, section 5.2.
        assert_eq!(
            x25519(
                &hex("a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4"),
                &hex("e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c"),
            ),
            hex("c3da55379de9c6908e94ea4df28d084f32eccf03491c71f754b4075577a28552")
        );
        assert_eq!(
            x25519(
                &hex("4b66e9d4d1b4673c5ad22691957d6af5c11b6421e0ea01d42ca4169e7918ba0d"),
                &hex("e5210f12786811d3f4b7959d0538ae2c31dbe7106fc03c3efc4cd549c715a493"),
            ),
            hex("95cbde9476e8907d7aade45cb4b873f88b595a68799fa152e6f8f7647aac7957")
        );
    }

    #[test]
    fn rfc_7748_key_pairs_and_shared_secret() {
        // RFC 7748, section 6.1.
        let alice = hex("77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a");
        let bob = hex("5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb");
        let alice_public = hex("8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a");
        let bob_public = hex("de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f");
        assert_eq!(public_key(&alice), alice_public);
        assert_eq!(public_key(&bob), bob_public);
        let shared = hex("4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742");
        assert_eq!(x25519(&alice, &bob_public), shared);
        assert_eq!(x25519(&bob, &alice_public), shared);
    }

    #[test]
    fn keys_round_trip_through_base64() {
        let key = hex("8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a");
        let text = encode_key(&key);
        assert_eq!(text, "hSDwCYkwp1R0i33ctD73Wg2/Og0mOBr066SpjqqbTmo=");
        assert_eq!(decode_key(&text), Some(key));
    }

    #[test]
    fn anything_that_is_not_a_key_is_refused() {
        assert_eq!(decode_key(""), None);
        assert_eq!(decode_key("hSDwCYkwp1R0i33ctD73Wg2/Og0mOBr066SpjqqbTmo"), None, "padding missing");
        assert_eq!(decode_key("hSDwCYkwp1R0i33ctD73Wg2/Og0mOBr066SpjqqbTm=="), None, "31 bytes");
        assert_eq!(decode_key("hSDwCYkwp1R0i33ctD73Wg2/Og0mOBr066SpjqqbTmp="), None, "stray bits");
        assert_eq!(decode_key("hSDwCYkwp1R0i33ctD73Wg2_Og0mOBr066SpjqqbTmo="), None, "url alphabet");
        assert_eq!(decode_key(" hSDwCYkwp1R0i33ctD73Wg2/Og0mOBr066SpjqqbTmo"), None);
    }
}
