pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub fn decode(data: &[u8]) -> Result<Picture, String> {
    const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if data.len() < 8 || data[..8] != SIGNATURE {
        return Err("to nie jest PNG".into());
    }
    let mut at = 8;
    let (mut width, mut height, mut channels) = (0u32, 0u32, 0usize);
    let mut compressed = Vec::new();
    while at + 8 <= data.len() {
        let length = u32::from_be_bytes(data[at..at + 4].try_into().unwrap()) as usize;
        let kind = &data[at + 4..at + 8];
        let body = data.get(at + 8..at + 8 + length).ok_or("ucięty PNG")?;
        match kind {
            b"IHDR" => {
                width = u32::from_be_bytes(body[0..4].try_into().unwrap());
                height = u32::from_be_bytes(body[4..8].try_into().unwrap());
                let (depth, color, interlace) = (body[8], body[9], body[12]);
                channels = match (depth, color, interlace) {
                    (8, 6, 0) => 4,
                    (8, 2, 0) => 3,
                    _ => return Err(format!("nieobsługiwany PNG: głębia {depth}, kolor {color}, przeplot {interlace}")),
                };
            }
            b"IDAT" => compressed.extend_from_slice(body),
            b"IEND" => break,
            _ => {}
        }
        at += 12 + length;
    }
    if width == 0 || height == 0 || channels == 0 {
        return Err("PNG bez nagłówka".into());
    }
    let stride = width as usize * channels;
    let raw = inflate(compressed.get(2..).ok_or("pusty PNG")?)?;
    if raw.len() < (stride + 1) * height as usize {
        return Err("za mało danych w PNG".into());
    }
    let mut rows = vec![0u8; stride * height as usize];
    for y in 0..height as usize {
        let filter = raw[y * (stride + 1)];
        let line = &raw[y * (stride + 1) + 1..(y + 1) * (stride + 1)];
        let (done, rest) = rows.split_at_mut(y * stride);
        let previous = if y > 0 { &done[(y - 1) * stride..] } else { &[][..] };
        let current = &mut rest[..stride];
        for x in 0..stride {
            let a = if x >= channels { current[x - channels] as i32 } else { 0 };
            let b = if y > 0 { previous[x] as i32 } else { 0 };
            let c = if y > 0 && x >= channels { previous[x - channels] as i32 } else { 0 };
            let predicted = match filter {
                0 => 0,
                1 => a,
                2 => b,
                3 => (a + b) / 2,
                4 => {
                    let p = a + b - c;
                    let (pa, pb, pc) = ((p - a).abs(), (p - b).abs(), (p - c).abs());
                    if pa <= pb && pa <= pc {
                        a
                    } else if pb <= pc {
                        b
                    } else {
                        c
                    }
                }
                _ => return Err(format!("nieznany filtr PNG {filter}")),
            };
            current[x] = (line[x] as i32 + predicted) as u8;
        }
    }
    let pixels = if channels == 4 {
        rows
    } else {
        rows.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect()
    };
    Ok(Picture { width, height, pixels })
}

struct Bits<'a> {
    data: &'a [u8],
    at: usize,
    bit: u32,
}

impl Bits<'_> {
    fn take(&mut self, count: u32) -> Result<u32, String> {
        let mut value = 0;
        for i in 0..count {
            let byte = *self.data.get(self.at).ok_or("ucięte dane")?;
            value |= (((byte >> self.bit) & 1) as u32) << i;
            self.bit += 1;
            if self.bit == 8 {
                self.bit = 0;
                self.at += 1;
            }
        }
        Ok(value)
    }

    fn align(&mut self) {
        if self.bit != 0 {
            self.bit = 0;
            self.at += 1;
        }
    }
}

struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Huffman {
        let mut counts = [0u16; 16];
        for &l in lengths {
            counts[l as usize] += 1;
        }
        counts[0] = 0;
        let mut offsets = [0u16; 16];
        for i in 1..16 {
            offsets[i] = offsets[i - 1] + counts[i - 1];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (symbol, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[offsets[l as usize] as usize] = symbol as u16;
                offsets[l as usize] += 1;
            }
        }
        Huffman { counts, symbols }
    }

    fn decode(&self, bits: &mut Bits) -> Result<u16, String> {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for length in 1..16 {
            code |= bits.take(1)? as i32;
            let count = self.counts[length] as i32;
            if code - first < count {
                return Ok(self.symbols[(index + code - first) as usize]);
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        Err("zły kod Huffmana".into())
    }
}

const LENGTH_BASE: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LENGTH_EXTRA: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const DISTANCE_BASE: [u16; 30] =
    [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
const DISTANCE_EXTRA: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];

pub fn inflate(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut bits = Bits { data, at: 0, bit: 0 };
    let mut out = Vec::new();
    loop {
        let last = bits.take(1)?;
        match bits.take(2)? {
            0 => {
                bits.align();
                let header = data.get(bits.at..bits.at + 4).ok_or("ucięty blok")?;
                let length = u16::from_le_bytes([header[0], header[1]]) as usize;
                bits.at += 4;
                out.extend_from_slice(data.get(bits.at..bits.at + length).ok_or("ucięty blok")?);
                bits.at += length;
            }
            1 => {
                let mut lengths = [0u8; 288];
                lengths[..144].fill(8);
                lengths[144..256].fill(9);
                lengths[256..280].fill(7);
                lengths[280..].fill(8);
                block(&mut bits, &mut out, &Huffman::new(&lengths), &Huffman::new(&[5; 30]))?;
            }
            2 => {
                let literals = bits.take(5)? as usize + 257;
                let distances = bits.take(5)? as usize + 1;
                let codes = bits.take(4)? as usize + 4;
                const ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
                let mut code_lengths = [0u8; 19];
                for &i in &ORDER[..codes] {
                    code_lengths[i] = bits.take(3)? as u8;
                }
                let code = Huffman::new(&code_lengths);
                let mut lengths = Vec::with_capacity(literals + distances);
                while lengths.len() < literals + distances {
                    let symbol = code.decode(&mut bits)?;
                    match symbol {
                        0..=15 => lengths.push(symbol as u8),
                        16 => {
                            let previous = *lengths.last().ok_or("powtórka bez poprzednika")?;
                            for _ in 0..3 + bits.take(2)? {
                                lengths.push(previous);
                            }
                        }
                        17 => lengths.extend(std::iter::repeat_n(0, 3 + bits.take(3)? as usize)),
                        _ => lengths.extend(std::iter::repeat_n(0, 11 + bits.take(7)? as usize)),
                    }
                }
                if lengths.len() != literals + distances {
                    return Err("złe długości kodów".into());
                }
                block(&mut bits, &mut out, &Huffman::new(&lengths[..literals]), &Huffman::new(&lengths[literals..]))?;
            }
            _ => return Err("zły typ bloku".into()),
        }
        if last == 1 {
            return Ok(out);
        }
    }
}

fn block(bits: &mut Bits, out: &mut Vec<u8>, literals: &Huffman, distances: &Huffman) -> Result<(), String> {
    loop {
        let symbol = literals.decode(bits)? as usize;
        if symbol < 256 {
            out.push(symbol as u8);
        } else if symbol == 256 {
            return Ok(());
        } else {
            let i = symbol - 257;
            if i >= 29 {
                return Err("zła długość".into());
            }
            let length = LENGTH_BASE[i] as usize + bits.take(LENGTH_EXTRA[i] as u32)? as usize;
            let d = distances.decode(bits)? as usize;
            if d >= 30 {
                return Err("zła odległość".into());
            }
            let distance = DISTANCE_BASE[d] as usize + bits.take(DISTANCE_EXTRA[d] as u32)? as usize;
            if distance > out.len() {
                return Err("odległość poza danymi".into());
            }
            let start = out.len() - distance;
            for k in 0..length {
                out.push(out[start + k]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_halloween_pictures() {
        for (name, data) in crate::render::PICTURES_FOR_TESTS {
            let picture = decode(data).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(picture.pixels.len(), (picture.width * picture.height * 4) as usize, "{name}");
            assert!(picture.pixels.chunks_exact(4).any(|p| p[3] > 200), "{name} ma coś widocznego");
        }
    }

    #[test]
    fn inflates_stored_and_fixed_blocks() {
        assert_eq!(inflate(&[0x01, 0x03, 0x00, 0xfc, 0xff, b'a', b'b', b'c']).unwrap(), b"abc");
        assert_eq!(inflate(&[0x4b, 0x4c, 0x4a, 0x06, 0x00]).unwrap(), b"abc");
    }
}
