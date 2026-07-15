// ---------------------------------------------------------------------------
// sha256.rs — Implementación SHA-256 desde cero (FIPS 180-4)
// ---------------------------------------------------------------------------
// Sin dependencias externas. Implementación completa del algoritmo.

pub struct Sha256 {
    estado: [u32; 8],
    msg_len: u64,
    buffer: [u8; 64],
    buffer_len: usize,
}

impl Sha256 {
    pub fn nuevo() -> Self {
        Sha256 {
            estado: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            msg_len: 0,
            buffer: [0u8; 64],
            buffer_len: 0,
        }
    }

    pub fn actualizar(&mut self, data: &[u8]) {
        let mut i = 0;
        self.msg_len += data.len() as u64;

        // Si hay datos en el buffer, intentar completar un bloque
        if self.buffer_len > 0 {
            while i < data.len() && self.buffer_len < 64 {
                self.buffer[self.buffer_len] = data[i];
                self.buffer_len += 1;
                i += 1;
            }
            if self.buffer_len == 64 {
                let bloque = self.buffer;
                self.procesar_bloque(bloque);
                self.buffer_len = 0;
            }
        }

        // Procesar bloques completos directamente
        while i + 64 <= data.len() {
            let mut bloque = [0u8; 64];
            bloque.copy_from_slice(&data[i..i + 64]);
            self.procesar_bloque(bloque);
            i += 64;
        }

        // Guardar el resto en el buffer
        while i < data.len() {
            self.buffer[self.buffer_len] = data[i];
            self.buffer_len += 1;
            i += 1;
        }
    }

    pub fn finalizar(mut self) -> [u8; 32] {
        // Padding: 1 byte 0x80 + ceros + longitud en bits (big-endian)
        let bit_len = self.msg_len * 8;
        let pad_len = if self.buffer_len < 56 {
            56 - self.buffer_len
        } else {
            120 - self.buffer_len
        };

        let mut padding = vec![0u8; pad_len + 8];
        padding[0] = 0x80;
        // Longitud en bits (big-endian, 8 bytes)
        padding[pad_len..pad_len + 8].copy_from_slice(&bit_len.to_be_bytes());
        self.actualizar(&padding);

        // Convertir estado a bytes (big-endian)
        let mut resultado = [0u8; 32];
        for i in 0..8 {
            resultado[i * 4..(i + 1) * 4].copy_from_slice(&self.estado[i].to_be_bytes());
        }
        resultado
    }

    fn procesar_bloque(&mut self, bloque: [u8; 64]) {
        // Constantes K (primeros 32 bits de raíces cúbicas de primeros 64 primos)
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
            0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
            0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
            0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
            0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
            0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
            0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
            0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
            0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
            0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
            0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
            0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
            0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
            0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
        ];

        // Preparar mensaje schedule W
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                bloque[i * 4], bloque[i * 4 + 1], bloque[i * 4 + 2], bloque[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        // Inicializar variables de trabajo
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (
            self.estado[0], self.estado[1], self.estado[2], self.estado[3],
            self.estado[4], self.estado[5], self.estado[6], self.estado[7],
        );

        // Ciclo principal
        for i in 0..64 {
            let big_s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(big_s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let big_s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = big_s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        // Sumar al estado
        self.estado[0] = self.estado[0].wrapping_add(a);
        self.estado[1] = self.estado[1].wrapping_add(b);
        self.estado[2] = self.estado[2].wrapping_add(c);
        self.estado[3] = self.estado[3].wrapping_add(d);
        self.estado[4] = self.estado[4].wrapping_add(e);
        self.estado[5] = self.estado[5].wrapping_add(f);
        self.estado[6] = self.estado[6].wrapping_add(g);
        self.estado[7] = self.estado[7].wrapping_add(h);
    }
}

/// Calcula SHA-256 de datos y retorna hex string de 64 caracteres.
pub fn sha256_hex(datos: &[u8]) -> String {
    let mut hasher = Sha256::nuevo();
    hasher.actualizar(datos);
    let hash = hasher.finalizar();
    hex_encode(&hash)
}

/// Calcula SHA-256 de un archivo.
pub fn sha256_archivo(path: &std::path::Path) -> Result<String, String> {
    let datos = std::fs::read(path).map_err(|e| format!("No se pudo leer {}: {}", path.display(), e))?;
    Ok(sha256_hex(&datos))
}

/// Convierte bytes a hex string.
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut resultado = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        resultado.push_str(&format!("{:02x}", b));
    }
    resultado
}

/// Convierte hex string a bytes.
pub fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string longitud impar".to_string());
    }
    let mut resultado = Vec::with_capacity(hex.len() / 2);
    let mut chars = hex.chars();
    while let Some(a) = chars.next() {
        let b = chars.next().ok_or("Hex string incompleta")?;
        let byte = u8::from_str_radix(&format!("{}{}", a, b), 16)
            .map_err(|_| format!("Hex inválido: {}{}", a, b))?;
        resultado.push(byte);
    }
    Ok(resultado)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_vacio() {
        let hash = sha256_hex(b"");
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn sha256_hola() {
        let hash = sha256_hex(b"hello");
        assert_eq!(hash, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn hex_roundtrip() {
        let datos = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let hex = hex_encode(&datos);
        assert_eq!(hex, "deadbeef");
        let decoded = hex_decode(&hex).unwrap();
        assert_eq!(decoded, datos);
    }
}
