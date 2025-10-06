use std::fs::File;
use std::io::{Read};
use std::path::Path;

/// Convert FIT "semicircles" to degrees.
#[inline]
fn semicircles_to_deg(v: i32) -> f64 {
    // Treat FIT "invalid" sentinel as missing.
    if v == i32::MAX { return f64::NAN; }
    // 180 / 2^31
    const K: f64 = 180.0 / 2147483648.0;
    (v as f64) * K
}

/// A compact in-memory definition for a local message.
#[derive(Copy, Clone)]
struct MsgDef {
    endian_big: bool,
    global_num: u16,
    // Up to 15 standard fields is common; keep lengths small and fixed.
    // We only need: (field_num, size), and the total data_len for fast skipping.
    fields: [(u8, u8); 32], // includes developer fields (if any)
    field_count: u8,
    data_len: u16,
}

impl MsgDef {
    #[inline] fn empty() -> Self {
        Self { endian_big: false, global_num: 0, fields: [(0,0); 32], field_count: 0, data_len: 0 }
    }
}

/// Parse coordinates (lat, lon) in degrees from FIT bytes.
/// - Fast path: assumes well-formed files; does bounds checks but no CRC validation.
/// - Extracts from "record" messages (global #20), fields #0 (position_lat) and #1 (position_long) as SINT32.
pub fn parse_fit_coords(buf: &[u8]) -> Vec<[f64; 2]> {
    let mut out: Vec<[f64;2]> = Vec::with_capacity(4096); // typical ride/run scale; grows if needed

    // ---- Header ------------------------------------------------------------
    if buf.len() < 12 { return out; }
    let hdr_sz = buf[0] as usize;
    if !(hdr_sz == 12 || hdr_sz == 14) || buf.len() < hdr_sz { return out; }
    // bytes 8..12 must be b".FIT"
    if &buf[8..12] != b".FIT" { return out; }
    // data size (little-endian, per FIT spec) at bytes 4..8
    let data_size = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
    let mut p = hdr_sz; // cursor at start of record stream
    let end_of_data = hdr_sz.saturating_add(data_size).min(buf.len());

    // ---- State: local message definitions (0..15) --------------------------
    let mut defs: [MsgDef; 16] = [MsgDef::empty(); 16];

    // ---- Helpers -----------------------------------------------------------
    #[inline] fn read_u16_le(b: &[u8]) -> u16 {
        u16::from_le_bytes([b[0], b[1]])
    }
    #[inline] fn read_u16_be(b: &[u8]) -> u16 {
        u16::from_be_bytes([b[0], b[1]])
    }
    #[inline] fn read_i32_le(b: &[u8]) -> i32 {
        i32::from_le_bytes([b[0], b[1], b[2], b[3]])
    }
    #[inline] fn read_i32_be(b: &[u8]) -> i32 {
        i32::from_be_bytes([b[0], b[1], b[2], b[3]])
    }

    // ---- Record Stream -----------------------------------------------------
    while p < end_of_data {
        let header = buf[p]; p += 1;

        // Compressed timestamp header? (bit7==1)
        if (header & 0x80) != 0 {
            // local msg: bits 5-6
            let local_id = ((header >> 5) & 0x03) as usize;
            let d = defs[local_id];
            let dl = d.data_len as usize;
            if dl == 0 || p.checked_add(dl).map(|q| q <= end_of_data).unwrap_or(false) == false {
                break; // malformed or missing definition
            }

            // Fast path: only handle global==20, and only read field#0 & #1 if they exist and size==4
            if d.global_num == 20 {
                // We need to find offsets for fields 0 and 1 once. Do it linearly; small counts.
                let mut off = 0usize;
                let mut lat_off = usize::MAX;
                let mut lon_off = usize::MAX;
                for i in 0..(d.field_count as usize) {
                    let (fnum, sz) = (d.fields[i].0, d.fields[i].1);
                    if fnum == 0 && sz == 4 { lat_off = off; }
                    if fnum == 1 && sz == 4 { lon_off = off; }
                    off += sz as usize;
                }
                // If both present, decode directly; otherwise, just skip.
                if lat_off != usize::MAX && lon_off != usize::MAX {
                    let rec = &buf[p..p+dl];
                    let (lat_raw, lon_raw) = if d.endian_big {
                        (read_i32_be(&rec[lat_off..lat_off+4]), read_i32_be(&rec[lon_off..lon_off+4]))
                    } else {
                        (read_i32_le(&rec[lat_off..lat_off+4]), read_i32_le(&rec[lon_off..lon_off+4]))
                    };
                    let lat = semicircles_to_deg(lat_raw);
                    let lon = semicircles_to_deg(lon_raw);
                    if lat.is_finite() && lon.is_finite() {
                        out.push([lat, lon]);
                    }
                }
            }
            p += dl;
            continue;
        }

        // Normal header:
        let local_id = (header & 0x0F) as usize;
        let is_definition = (header & 0x40) != 0;
        let has_dev_fields = (header & 0x20) != 0;

        if is_definition {
            // Definition message
            if p + 5 > end_of_data { break; }
            let _reserved = buf[p]; // always 0
            let arch = buf[p+1]; // 0=little, 1=big
            let endian_big = arch == 1;
            let global_num = if endian_big {
                read_u16_be(&buf[p+2..p+4])
            } else {
                read_u16_le(&buf[p+2..p+4])
            };
            let field_count = buf[p+4] as usize;
            p += 5;

            let need = field_count.checked_mul(3).unwrap_or(usize::MAX);
            if need == usize::MAX || p + need > end_of_data { break; }

            let mut def = MsgDef::empty();
            def.endian_big = endian_big;
            def.global_num = global_num;

            let mut total_len: usize = 0;
            let mut fc = 0usize;

            // Standard fields
            for _ in 0..field_count {
                let fnum = buf[p];
                let fsize = buf[p+1];
                let _btype = buf[p+2] & 0x1F; // base type id; we only care about size here
                p += 3;
                if fc < def.fields.len() {
                    def.fields[fc] = (fnum, fsize);
                    fc += 1;
                }
                total_len += fsize as usize;
            }

            // Developer fields (optional)
            if has_dev_fields {
                if p >= end_of_data { break; }
                let dev_cnt = buf[p] as usize;
                p += 1;
                let need_dev = dev_cnt.checked_mul(3).unwrap_or(usize::MAX);
                if need_dev == usize::MAX || p + need_dev > end_of_data { break; }
                for _ in 0..dev_cnt {
                    let fnum = buf[p];        // developer field number
                    let fsize = buf[p+1];     // size in bytes
                    let _dev_idx = buf[p+2];  // developer data index
                    p += 3;
                    if fc < def.fields.len() {
                        def.fields[fc] = (fnum, fsize);
                        fc += 1;
                    }
                    total_len += fsize as usize;
                }
            }

            def.field_count = fc as u8;
            def.data_len = (total_len as u16).min(u16::MAX);
            defs[local_id] = def;
        } else {
            // Data message
            let d = defs[local_id];
            let dl = d.data_len as usize;
            if dl == 0 || p.checked_add(dl).map(|q| q <= end_of_data).unwrap_or(false) == false {
                break; // malformed or unknown definition
            }

            if d.global_num == 20 {
                // Find lat/lon offsets (field#0 and #1, size 4)
                let mut off = 0usize;
                let mut lat_off = usize::MAX;
                let mut lon_off = usize::MAX;
                for i in 0..(d.field_count as usize) {
                    let (fnum, sz) = (d.fields[i].0, d.fields[i].1);
                    if fnum == 0 && sz == 4 { lat_off = off; }
                    if fnum == 1 && sz == 4 { lon_off = off; }
                    off += sz as usize;
                }
                if lat_off != usize::MAX && lon_off != usize::MAX {
                    let rec = &buf[p..p+dl];
                    let (lat_raw, lon_raw) = if d.endian_big {
                        (read_i32_be(&rec[lat_off..lat_off+4]), read_i32_be(&rec[lon_off..lon_off+4]))
                    } else {
                        (read_i32_le(&rec[lat_off..lat_off+4]), read_i32_le(&rec[lon_off..lon_off+4]))
                    };
                    let lat = semicircles_to_deg(lat_raw);
                    let lon = semicircles_to_deg(lon_raw);
                    if lat.is_finite() && lon.is_finite() {
                        out.push([lat, lon]);
                    }
                }
            }
            p += dl;
        }
    }

    out
}

// ------------------------- Convenience I/O -------------------------

/// Read a FIT file from disk and return (lat, lon) in degrees.
pub fn parse_fit_coords_from_path(path: impl AsRef<Path>) -> std::io::Result<Vec<[f64;2]>> {
    let mut f = File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(parse_fit_coords(&buf))
}

// ------------------------- Example (comment out in library) -------------------------
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smoke_empty() {
        let v = parse_fit_coords(&[]);
        assert!(v.is_empty());
    }
}


