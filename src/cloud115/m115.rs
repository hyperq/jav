use rand::RngCore;

const RSA_N_HEX: &str = "8686980c0f5a24c4b9d43020cd2c22703ff3f450756529058b1cf88f09b8602136477198a6e2683149659bd122c33592fdb5ad47944ad1ea4d36c6b172aad6338c3bb6ac6227502d010993ac967d1aef00f0c8e038de2e4d3bc2ec368af2e9f10a6f1eda4f7262f136420c07c331b871bf139f74f3010e3c4fe57df3afb71683";
const RSA_E: u32 = 0x10001;

static XOR_KEY_SEED: [u8; 144] = [
    0xf0, 0xe5, 0x69, 0xae, 0xbf, 0xdc, 0xbf, 0x8a,
    0x1a, 0x45, 0xe8, 0xbe, 0x7d, 0xa6, 0x73, 0xb8,
    0xde, 0x8f, 0xe7, 0xc4, 0x45, 0xda, 0x86, 0xc4,
    0x9b, 0x64, 0x8b, 0x14, 0x6a, 0xb4, 0xf1, 0xaa,
    0x38, 0x01, 0x35, 0x9e, 0x26, 0x69, 0x2c, 0x86,
    0x00, 0x6b, 0x4f, 0xa5, 0x36, 0x34, 0x62, 0xa6,
    0x2a, 0x96, 0x68, 0x18, 0xf2, 0x4a, 0xfd, 0xbd,
    0x6b, 0x97, 0x8f, 0x4d, 0x8f, 0x89, 0x13, 0xb7,
    0x6c, 0x8e, 0x93, 0xed, 0x0e, 0x0d, 0x48, 0x3e,
    0xd7, 0x2f, 0x88, 0xd8, 0xfe, 0xfe, 0x7e, 0x86,
    0x50, 0x95, 0x4f, 0xd1, 0xeb, 0x83, 0x26, 0x34,
    0xdb, 0x66, 0x7b, 0x9c, 0x7e, 0x9d, 0x7a, 0x81,
    0x32, 0xea, 0xb6, 0x33, 0xde, 0x3a, 0xa9, 0x59,
    0x34, 0x66, 0x3b, 0xaa, 0xba, 0x81, 0x60, 0x48,
    0xb9, 0xd5, 0x81, 0x9c, 0xf8, 0x6c, 0x84, 0x77,
    0xff, 0x54, 0x78, 0x26, 0x5f, 0xbe, 0xe8, 0x1e,
    0x36, 0x9f, 0x34, 0x80, 0x5c, 0x45, 0x2c, 0x9b,
    0x76, 0xd5, 0x1b, 0x8f, 0xcc, 0xc3, 0xb8, 0xf5,
];

static XOR_CLIENT_KEY: [u8; 12] = [
    0x78, 0x06, 0xad, 0x4c, 0x33, 0x86, 0x5d, 0x18,
    0x4c, 0x01, 0x3f, 0x46,
];

pub type Key = [u8; 16];

pub fn generate_key() -> Key {
    let mut key = [0u8; 16];
    rand::rng().fill_bytes(&mut key);
    key
}

pub fn encode(input: &[u8], key: &Key) -> String {
    use base64::Engine;
    let mut buf = vec![0u8; 16 + input.len()];
    buf[..16].copy_from_slice(key);
    buf[16..].copy_from_slice(input);

    xor_transform(&mut buf[16..], &xor_derive_key(key, 4));
    buf[16..].reverse();
    xor_transform(&mut buf[16..], &XOR_CLIENT_KEY);

    let encrypted = rsa_encrypt(&buf);
    base64::engine::general_purpose::STANDARD.encode(encrypted)
}

pub fn decode(input: &str, key: &Key) -> anyhow::Result<Vec<u8>> {
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD.decode(input)?;
    let data = rsa_decrypt(&data);
    if data.len() < 16 {
        anyhow::bail!("decrypted data too short");
    }

    let mut output = data[16..].to_vec();
    xor_transform(&mut output, &xor_derive_key(&data[..16], 12));
    output.reverse();
    xor_transform(&mut output, &xor_derive_key(key, 4));
    Ok(output)
}

fn xor_derive_key(seed: &[u8], size: usize) -> Vec<u8> {
    let mut key = vec![0u8; size];
    for i in 0..size {
        key[i] = seed[i % seed.len()].wrapping_add(XOR_KEY_SEED[size * i]);
        key[i] ^= XOR_KEY_SEED[size * (size - i - 1)];
    }
    key
}

fn xor_transform(data: &mut [u8], key: &[u8]) {
    let data_size = data.len();
    let key_size = key.len();
    let modv = data_size % 4;
    for i in 0..modv {
        data[i] ^= key[i % key_size];
    }
    for i in modv..data_size {
        data[i] ^= key[(i - modv) % key_size];
    }
}

// --- RSA with big integer (no external crate, using simple bignum) ---

fn rsa_encrypt(input: &[u8]) -> Vec<u8> {
    let n = BigUint::from_hex(RSA_N_HEX);
    let e = BigUint::from_u32(RSA_E);
    let key_len = RSA_N_HEX.len() / 2; // 128 bytes

    let mut result = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        let slice_size = (key_len - 11).min(remaining.len());
        let slice = &remaining[..slice_size];
        remaining = &remaining[slice_size..];

        let encrypted = rsa_encrypt_slice(slice, key_len, &n, &e);
        result.extend_from_slice(&encrypted);
    }
    result
}

fn rsa_encrypt_slice(input: &[u8], key_len: usize, n: &BigUint, e: &BigUint) -> Vec<u8> {
    let pad_size = key_len - input.len() - 3;
    let mut buf = vec![0u8; key_len];
    buf[0] = 0;
    buf[1] = 2;

    let mut pad_data = vec![0u8; pad_size];
    rand::rng().fill_bytes(&mut pad_data);
    for i in 0..pad_size {
        buf[2 + i] = (pad_data[i] % 0xff) + 1;
    }
    buf[pad_size + 2] = 0;
    buf[pad_size + 3..].copy_from_slice(input);

    let msg = BigUint::from_bytes_be(&buf);
    let ret = msg.mod_pow(e, n);
    let ret_bytes = ret.to_bytes_be();

    let mut out = vec![0u8; key_len];
    let start = key_len - ret_bytes.len();
    out[start..].copy_from_slice(&ret_bytes);
    out
}

fn rsa_decrypt(input: &[u8]) -> Vec<u8> {
    let n = BigUint::from_hex(RSA_N_HEX);
    let e = BigUint::from_u32(RSA_E);
    let key_len = RSA_N_HEX.len() / 2;

    let mut result = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        let slice_size = key_len.min(remaining.len());
        let slice = &remaining[..slice_size];
        remaining = &remaining[slice_size..];

        let decrypted = rsa_decrypt_slice(slice, &n, &e);
        result.extend_from_slice(&decrypted);
    }
    result
}

fn rsa_decrypt_slice(input: &[u8], n: &BigUint, e: &BigUint) -> Vec<u8> {
    let msg = BigUint::from_bytes_be(input);
    let ret = msg.mod_pow(e, n);
    let ret_bytes = ret.to_bytes_be();

    for (i, &b) in ret_bytes.iter().enumerate() {
        if b == 0 && i != 0 {
            return ret_bytes[i + 1..].to_vec();
        }
    }
    ret_bytes
}

// --- Minimal BigUint for RSA (only what we need: from_hex, mod_pow, to_bytes_be) ---

#[derive(Clone)]
struct BigUint {
    digits: Vec<u64>, // little-endian base-2^64
}

impl BigUint {
    fn from_hex(hex: &str) -> Self {
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();
        Self::from_bytes_be(&bytes)
    }

    fn from_u32(v: u32) -> Self {
        Self { digits: vec![v as u64] }
    }

    fn from_bytes_be(bytes: &[u8]) -> Self {
        let mut digits = Vec::new();
        let mut i = bytes.len();
        while i > 0 {
            let start = if i >= 8 { i - 8 } else { 0 };
            let mut val = 0u64;
            for &b in &bytes[start..i] {
                val = (val << 8) | (b as u64);
            }
            digits.push(val);
            i = start;
        }
        while digits.len() > 1 && *digits.last().unwrap() == 0 {
            digits.pop();
        }
        Self { digits }
    }

    fn to_bytes_be(&self) -> Vec<u8> {
        if self.is_zero() {
            return vec![0];
        }
        let mut bytes = Vec::new();
        for &d in self.digits.iter().rev() {
            for i in (0..8).rev() {
                bytes.push((d >> (i * 8)) as u8);
            }
        }
        while bytes.first() == Some(&0) && bytes.len() > 1 {
            bytes.remove(0);
        }
        bytes
    }

    fn is_zero(&self) -> bool {
        self.digits.iter().all(|&d| d == 0)
    }

    fn mod_pow(&self, exp: &BigUint, modulus: &BigUint) -> BigUint {
        let mut result = BigUint::from_u32(1);
        let mut base = self.mod_op(modulus);
        let exp_bits = exp.to_bits();

        for bit in exp_bits {
            if bit {
                result = result.mul(&base).mod_op(modulus);
            }
            base = base.mul(&base).mod_op(modulus);
        }
        result
    }

    fn to_bits(&self) -> Vec<bool> {
        let mut bits = Vec::new();
        for &d in &self.digits {
            for i in 0..64 {
                bits.push((d >> i) & 1 == 1);
            }
        }
        while bits.last() == Some(&false) && !bits.is_empty() {
            bits.pop();
        }
        bits
    }

    fn mul(&self, other: &BigUint) -> BigUint {
        let n = self.digits.len() + other.digits.len();
        let mut result = vec![0u64; n + 1];
        for (i, &a) in self.digits.iter().enumerate() {
            let mut carry = 0u128;
            for (j, &b) in other.digits.iter().enumerate() {
                let prod = (a as u128) * (b as u128) + (result[i + j] as u128) + carry;
                result[i + j] = prod as u64;
                carry = prod >> 64;
            }
            let mut k = i + other.digits.len();
            while carry > 0 {
                let v = (result[k] as u128) + carry;
                result[k] = v as u64;
                carry = v >> 64;
                k += 1;
            }
        }
        while result.len() > 1 && *result.last().unwrap() == 0 {
            result.pop();
        }
        BigUint { digits: result }
    }

    fn mod_op(&self, modulus: &BigUint) -> BigUint {
        self.div_mod(modulus).1
    }

    fn div_mod(&self, divisor: &BigUint) -> (BigUint, BigUint) {
        if divisor.is_zero() {
            panic!("division by zero");
        }
        if self.cmp(divisor) == std::cmp::Ordering::Less {
            return (BigUint::from_u32(0), self.clone());
        }

        let shift = self.bit_len() - divisor.bit_len();
        let mut rem = self.clone();
        let mut quotient_digits = vec![0u64; shift / 64 + 1];

        for i in (0..=shift).rev() {
            let shifted = divisor.shl(i);
            if rem.cmp(&shifted) != std::cmp::Ordering::Less {
                rem = rem.sub(&shifted);
                quotient_digits[i / 64] |= 1u64 << (i % 64);
            }
        }

        while quotient_digits.len() > 1 && *quotient_digits.last().unwrap() == 0 {
            quotient_digits.pop();
        }
        (BigUint { digits: quotient_digits }, rem)
    }

    fn sub(&self, other: &BigUint) -> BigUint {
        let mut digits = vec![0u64; self.digits.len()];
        let mut borrow = 0i128;
        for i in 0..self.digits.len() {
            let a = self.digits[i] as i128;
            let b = if i < other.digits.len() { other.digits[i] as i128 } else { 0 };
            let v = a - b - borrow;
            if v < 0 {
                digits[i] = (v + (1i128 << 64)) as u64;
                borrow = 1;
            } else {
                digits[i] = v as u64;
                borrow = 0;
            }
        }
        while digits.len() > 1 && *digits.last().unwrap() == 0 {
            digits.pop();
        }
        BigUint { digits }
    }

    fn shl(&self, shift: usize) -> BigUint {
        let word_shift = shift / 64;
        let bit_shift = shift % 64;
        let mut digits = vec![0u64; self.digits.len() + word_shift + 1];
        for (i, &d) in self.digits.iter().enumerate() {
            digits[i + word_shift] |= d.checked_shl(bit_shift as u32).unwrap_or(0);
            if bit_shift > 0 {
                digits[i + word_shift + 1] |= d.checked_shr((64 - bit_shift) as u32).unwrap_or(0);
            }
        }
        while digits.len() > 1 && *digits.last().unwrap() == 0 {
            digits.pop();
        }
        BigUint { digits }
    }

    fn bit_len(&self) -> usize {
        if self.is_zero() { return 0; }
        let top = self.digits.len() - 1;
        top * 64 + (64 - self.digits[top].leading_zeros() as usize)
    }

    fn cmp(&self, other: &BigUint) -> std::cmp::Ordering {
        match self.digits.len().cmp(&other.digits.len()) {
            std::cmp::Ordering::Equal => {
                for i in (0..self.digits.len()).rev() {
                    match self.digits[i].cmp(&other.digits[i]) {
                        std::cmp::Ordering::Equal => continue,
                        ord => return ord,
                    }
                }
                std::cmp::Ordering::Equal
            }
            ord => ord,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bigint_basic() {
        let a = BigUint::from_u32(12345);
        let b = BigUint::from_u32(67890);
        let prod = a.mul(&b);
        // 12345 * 67890 = 838102050
        assert_eq!(prod.digits[0], 838102050);
    }

    #[test]
    fn bigint_mod_pow() {
        // 3^7 mod 13 = 2187 mod 13 = 3
        let base = BigUint::from_u32(3);
        let exp = BigUint::from_u32(7);
        let modulus = BigUint::from_u32(13);
        let result = base.mod_pow(&exp, &modulus);
        assert_eq!(result.digits[0], 3);
    }

    #[test]
    fn encode_produces_output() {
        let key = generate_key();
        let payload = br#"{"pickcode":"test"}"#;
        let encoded = encode(payload, &key);
        assert!(!encoded.is_empty());
        // should be valid base64
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD.decode(&encoded);
        assert!(decoded.is_ok());
    }
}
