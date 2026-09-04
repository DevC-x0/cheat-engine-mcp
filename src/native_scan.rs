use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
}

impl ScanType {
    pub fn size(&self) -> usize {
        match self {
            ScanType::Int8 | ScanType::UInt8 => 1,
            ScanType::Int16 | ScanType::UInt16 => 2,
            ScanType::Int32 | ScanType::UInt32 | ScanType::Float32 => 4,
            ScanType::Int64 | ScanType::UInt64 | ScanType::Float64 => 8,
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            ScanType::Int8 | ScanType::UInt8 => 1,
            ScanType::Int16 | ScanType::UInt16 => 2,
            ScanType::Int32 | ScanType::UInt32 | ScanType::Float32 => 4,
            ScanType::Int64 | ScanType::UInt64 | ScanType::Float64 => 4,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "int8" | "i8" | "byte" => Some(ScanType::Int8),
            "uint8" | "u8" => Some(ScanType::UInt8),
            "int16" | "i16" | "short" => Some(ScanType::Int16),
            "uint16" | "u16" => Some(ScanType::UInt16),
            "int32" | "i32" | "int" | "number" => Some(ScanType::Int32),
            "uint32" | "u32" => Some(ScanType::UInt32),
            "int64" | "i64" | "long" => Some(ScanType::Int64),
            "uint64" | "u64" => Some(ScanType::UInt64),
            "float" | "f32" | "single" => Some(ScanType::Float32),
            "double" | "f64" => Some(ScanType::Float64),
            _ => None,
        }
    }

    pub fn detect(value: &str) -> Self {
        if value.contains('.') {
            ScanType::Float32
        } else if let Ok(n) = value.parse::<i64>() {
            if n >= i32::MIN as i64 && n <= i32::MAX as i64 {
                ScanType::Int32
            } else {
                ScanType::Int64
            }
        } else {
            ScanType::Int32
        }
    }
}

impl fmt::Display for ScanType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ScanType::Int8 => "int8",
            ScanType::UInt8 => "uint8",
            ScanType::Int16 => "int16",
            ScanType::UInt16 => "uint16",
            ScanType::Int32 => "int32",
            ScanType::UInt32 => "uint32",
            ScanType::Int64 => "int64",
            ScanType::UInt64 => "uint64",
            ScanType::Float32 => "float",
            ScanType::Float64 => "double",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedValue {
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl ParsedValue {
    pub fn parse(s: &str, scan_type: ScanType) -> Result<Self, String> {
        let s = s.trim();
        match scan_type {
            ScanType::Int8 => s
                .parse::<i8>()
                .map(ParsedValue::I8)
                .map_err(|e| e.to_string()),
            ScanType::UInt8 => s
                .parse::<u8>()
                .map(ParsedValue::U8)
                .map_err(|e| e.to_string()),
            ScanType::Int16 => s
                .parse::<i16>()
                .map(ParsedValue::I16)
                .map_err(|e| e.to_string()),
            ScanType::UInt16 => s
                .parse::<u16>()
                .map(ParsedValue::U16)
                .map_err(|e| e.to_string()),
            ScanType::Int32 => s
                .parse::<i32>()
                .map(ParsedValue::I32)
                .map_err(|e| e.to_string()),
            ScanType::UInt32 => s
                .parse::<u32>()
                .map(ParsedValue::U32)
                .map_err(|e| e.to_string()),
            ScanType::Int64 => s
                .parse::<i64>()
                .map(ParsedValue::I64)
                .map_err(|e| e.to_string()),
            ScanType::UInt64 => s
                .parse::<u64>()
                .map(ParsedValue::U64)
                .map_err(|e| e.to_string()),
            ScanType::Float32 => s
                .parse::<f32>()
                .map(ParsedValue::F32)
                .map_err(|e| e.to_string()),
            ScanType::Float64 => s
                .parse::<f64>()
                .map(ParsedValue::F64)
                .map_err(|e| e.to_string()),
        }
    }

    pub fn from_bytes(bytes: &[u8], scan_type: ScanType) -> Option<Self> {
        if bytes.len() < scan_type.size() {
            return None;
        }
        match scan_type {
            ScanType::Int8 => Some(ParsedValue::I8(bytes[0] as i8)),
            ScanType::UInt8 => Some(ParsedValue::U8(bytes[0])),
            ScanType::Int16 => {
                let arr: [u8; 2] = bytes[0..2].try_into().ok()?;
                Some(ParsedValue::I16(i16::from_ne_bytes(arr)))
            }
            ScanType::UInt16 => {
                let arr: [u8; 2] = bytes[0..2].try_into().ok()?;
                Some(ParsedValue::U16(u16::from_ne_bytes(arr)))
            }
            ScanType::Int32 => {
                let arr: [u8; 4] = bytes[0..4].try_into().ok()?;
                Some(ParsedValue::I32(i32::from_ne_bytes(arr)))
            }
            ScanType::UInt32 => {
                let arr: [u8; 4] = bytes[0..4].try_into().ok()?;
                Some(ParsedValue::U32(u32::from_ne_bytes(arr)))
            }
            ScanType::Int64 => {
                let arr: [u8; 8] = bytes[0..8].try_into().ok()?;
                Some(ParsedValue::I64(i64::from_ne_bytes(arr)))
            }
            ScanType::UInt64 => {
                let arr: [u8; 8] = bytes[0..8].try_into().ok()?;
                Some(ParsedValue::U64(u64::from_ne_bytes(arr)))
            }
            ScanType::Float32 => {
                let arr: [u8; 4] = bytes[0..4].try_into().ok()?;
                Some(ParsedValue::F32(f32::from_ne_bytes(arr)))
            }
            ScanType::Float64 => {
                let arr: [u8; 8] = bytes[0..8].try_into().ok()?;
                Some(ParsedValue::F64(f64::from_ne_bytes(arr)))
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            ParsedValue::I8(v) => vec![*v as u8],
            ParsedValue::U8(v) => vec![*v],
            ParsedValue::I16(v) => v.to_ne_bytes().to_vec(),
            ParsedValue::U16(v) => v.to_ne_bytes().to_vec(),
            ParsedValue::I32(v) => v.to_ne_bytes().to_vec(),
            ParsedValue::U32(v) => v.to_ne_bytes().to_vec(),
            ParsedValue::I64(v) => v.to_ne_bytes().to_vec(),
            ParsedValue::U64(v) => v.to_ne_bytes().to_vec(),
            ParsedValue::F32(v) => v.to_ne_bytes().to_vec(),
            ParsedValue::F64(v) => v.to_ne_bytes().to_vec(),
        }
    }

    pub fn matches_exact(&self, target: &ParsedValue) -> bool {
        match (self, target) {
            (ParsedValue::I8(a), ParsedValue::I8(b)) => a == b,
            (ParsedValue::U8(a), ParsedValue::U8(b)) => a == b,
            (ParsedValue::I16(a), ParsedValue::I16(b)) => a == b,
            (ParsedValue::U16(a), ParsedValue::U16(b)) => a == b,
            (ParsedValue::I32(a), ParsedValue::I32(b)) => a == b,
            (ParsedValue::U32(a), ParsedValue::U32(b)) => a == b,
            (ParsedValue::I64(a), ParsedValue::I64(b)) => a == b,
            (ParsedValue::U64(a), ParsedValue::U64(b)) => a == b,
            (ParsedValue::F32(a), ParsedValue::F32(b)) => (a - b).abs() < 0.001,
            (ParsedValue::F64(a), ParsedValue::F64(b)) => (a - b).abs() < 0.00001,
            _ => false,
        }
    }

    pub fn is_greater_than(&self, other: &ParsedValue) -> bool {
        match (self, other) {
            (ParsedValue::I8(a), ParsedValue::I8(b)) => a > b,
            (ParsedValue::U8(a), ParsedValue::U8(b)) => a > b,
            (ParsedValue::I16(a), ParsedValue::I16(b)) => a > b,
            (ParsedValue::U16(a), ParsedValue::U16(b)) => a > b,
            (ParsedValue::I32(a), ParsedValue::I32(b)) => a > b,
            (ParsedValue::U32(a), ParsedValue::U32(b)) => a > b,
            (ParsedValue::I64(a), ParsedValue::I64(b)) => a > b,
            (ParsedValue::U64(a), ParsedValue::U64(b)) => a > b,
            (ParsedValue::F32(a), ParsedValue::F32(b)) => a > b,
            (ParsedValue::F64(a), ParsedValue::F64(b)) => a > b,
            _ => false,
        }
    }

    pub fn is_less_than(&self, other: &ParsedValue) -> bool {
        match (self, other) {
            (ParsedValue::I8(a), ParsedValue::I8(b)) => a < b,
            (ParsedValue::U8(a), ParsedValue::U8(b)) => a < b,
            (ParsedValue::I16(a), ParsedValue::I16(b)) => a < b,
            (ParsedValue::U16(a), ParsedValue::U16(b)) => a < b,
            (ParsedValue::I32(a), ParsedValue::I32(b)) => a < b,
            (ParsedValue::U32(a), ParsedValue::U32(b)) => a < b,
            (ParsedValue::I64(a), ParsedValue::I64(b)) => a < b,
            (ParsedValue::U64(a), ParsedValue::U64(b)) => a < b,
            (ParsedValue::F32(a), ParsedValue::F32(b)) => a < b,
            (ParsedValue::F64(a), ParsedValue::F64(b)) => a < b,
            _ => false,
        }
    }

    pub fn is_changed(&self, other: &ParsedValue) -> bool {
        !self.matches_exact(other)
    }

    pub fn is_unchanged(&self, other: &ParsedValue) -> bool {
        self.matches_exact(other)
    }

    pub fn is_in_range(&self, low: &ParsedValue, high: &ParsedValue) -> bool {
        match (self, low, high) {
            (ParsedValue::I8(v), ParsedValue::I8(l), ParsedValue::I8(h)) => v >= l && v <= h,
            (ParsedValue::U8(v), ParsedValue::U8(l), ParsedValue::U8(h)) => v >= l && v <= h,
            (ParsedValue::I16(v), ParsedValue::I16(l), ParsedValue::I16(h)) => v >= l && v <= h,
            (ParsedValue::U16(v), ParsedValue::U16(l), ParsedValue::U16(h)) => v >= l && v <= h,
            (ParsedValue::I32(v), ParsedValue::I32(l), ParsedValue::I32(h)) => v >= l && v <= h,
            (ParsedValue::U32(v), ParsedValue::U32(l), ParsedValue::U32(h)) => v >= l && v <= h,
            (ParsedValue::I64(v), ParsedValue::I64(l), ParsedValue::I64(h)) => v >= l && v <= h,
            (ParsedValue::U64(v), ParsedValue::U64(l), ParsedValue::U64(h)) => v >= l && v <= h,
            (ParsedValue::F32(v), ParsedValue::F32(l), ParsedValue::F32(h)) => v >= l && v <= h,
            (ParsedValue::F64(v), ParsedValue::F64(l), ParsedValue::F64(h)) => v >= l && v <= h,
            _ => false,
        }
    }
}

impl fmt::Display for ParsedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsedValue::I8(v) => write!(f, "{v}"),
            ParsedValue::U8(v) => write!(f, "{v}"),
            ParsedValue::I16(v) => write!(f, "{v}"),
            ParsedValue::U16(v) => write!(f, "{v}"),
            ParsedValue::I32(v) => write!(f, "{v}"),
            ParsedValue::U32(v) => write!(f, "{v}"),
            ParsedValue::I64(v) => write!(f, "{v}"),
            ParsedValue::U64(v) => write!(f, "{v}"),
            ParsedValue::F32(v) => write!(f, "{v}"),
            ParsedValue::F64(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScanMatch {
    pub address: u64,
    pub value: ParsedValue,
}

pub const MAX_MATCHES_SAVED: usize = 20000;

pub fn scan_buffer_exact(
    base_address: u64,
    buffer: &[u8],
    scan_type: ScanType,
    target: &ParsedValue,
    step: usize,
    results: &mut Vec<ScanMatch>,
) {
    let size = scan_type.size();
    if buffer.len() < size {
        return;
    }
    let step = if step == 0 {
        scan_type.alignment()
    } else {
        step
    };
    let max_offset = buffer.len() - size;
    let mut offset = 0;
    while offset <= max_offset {
        if results.len() >= MAX_MATCHES_SAVED {
            break;
        }
        if let Some(val) = ParsedValue::from_bytes(&buffer[offset..offset + size], scan_type) {
            if val.matches_exact(target) {
                results.push(ScanMatch {
                    address: base_address + offset as u64,
                    value: val,
                });
            }
        }
        offset += step;
    }
}

pub fn scan_buffer_range(
    base_address: u64,
    buffer: &[u8],
    scan_type: ScanType,
    low: &ParsedValue,
    high: &ParsedValue,
    step: usize,
    results: &mut Vec<ScanMatch>,
) {
    let size = scan_type.size();
    if buffer.len() < size {
        return;
    }
    let step = if step == 0 {
        scan_type.alignment()
    } else {
        step
    };
    let max_offset = buffer.len() - size;
    let mut offset = 0;
    while offset <= max_offset {
        if results.len() >= MAX_MATCHES_SAVED {
            break;
        }
        if let Some(val) = ParsedValue::from_bytes(&buffer[offset..offset + size], scan_type) {
            if val.is_in_range(low, high) {
                results.push(ScanMatch {
                    address: base_address + offset as u64,
                    value: val,
                });
            }
        }
        offset += step;
    }
}

pub fn scan_buffer_any(
    base_address: u64,
    buffer: &[u8],
    scan_type: ScanType,
    step: usize,
    results: &mut Vec<ScanMatch>,
) {
    let size = scan_type.size();
    if buffer.len() < size {
        return;
    }
    let step = if step == 0 {
        scan_type.alignment()
    } else {
        step
    };
    let max_offset = buffer.len() - size;
    let mut offset = 0;
    while offset <= max_offset {
        if results.len() >= MAX_MATCHES_SAVED {
            break;
        }
        if let Some(val) = ParsedValue::from_bytes(&buffer[offset..offset + size], scan_type) {
            results.push(ScanMatch {
                address: base_address + offset as u64,
                value: val,
            });
        }
        offset += step;
    }
}

pub fn format_scan_output(
    matches: &[ScanMatch],
    total_count: usize,
    scan_type: ScanType,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("{} matches found.\n", total_count));
    let preview_count = matches.len().min(10);
    for (i, m) in matches.iter().take(preview_count).enumerate() {
        out.push_str(&format!(
            "[{:>2}] 0x{:08x}, {}, {}\n",
            i + 1,
            m.address,
            m.value,
            scan_type
        ));
    }
    if total_count > preview_count {
        out.push_str(&format!(
            "... and {} more matches\n",
            total_count - preview_count
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_type_detect() {
        assert_eq!(ScanType::detect("100"), ScanType::Int32);
        assert_eq!(ScanType::detect("100.5"), ScanType::Float32);
        assert_eq!(ScanType::detect("5000000000"), ScanType::Int64);
    }

    #[test]
    fn test_parse_and_compare() {
        let val1 = ParsedValue::parse("100", ScanType::Int32).unwrap();
        let val2 = ParsedValue::parse("100", ScanType::Int32).unwrap();
        let val3 = ParsedValue::parse("150", ScanType::Int32).unwrap();

        assert!(val1.matches_exact(&val2));
        assert!(val3.is_greater_than(&val1));
        assert!(val1.is_less_than(&val3));
        assert!(val3.is_changed(&val1));
        assert!(val1.is_unchanged(&val2));
    }

    #[test]
    fn test_scan_buffer_exact() {
        let mut buffer = vec![0u8; 32];
        let val: i32 = 42;
        buffer[8..12].copy_from_slice(&val.to_ne_bytes());
        buffer[20..24].copy_from_slice(&val.to_ne_bytes());

        let target = ParsedValue::I32(42);
        let mut matches = Vec::new();
        scan_buffer_exact(0x1000, &buffer, ScanType::Int32, &target, 4, &mut matches);

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].address, 0x1008);
        assert_eq!(matches[1].address, 0x1014);
    }

    #[test]
    fn test_scan_buffer_range() {
        let mut buffer = vec![0u8; 32];
        let val1: i32 = 50;
        let val2: i32 = 150;
        let val3: i32 = 300;
        buffer[4..8].copy_from_slice(&val1.to_ne_bytes());
        buffer[12..16].copy_from_slice(&val2.to_ne_bytes());
        buffer[20..24].copy_from_slice(&val3.to_ne_bytes());

        let low = ParsedValue::I32(40);
        let high = ParsedValue::I32(200);
        let mut matches = Vec::new();
        scan_buffer_range(
            0x2000,
            &buffer,
            ScanType::Int32,
            &low,
            &high,
            4,
            &mut matches,
        );

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].address, 0x2004);
        assert_eq!(matches[1].address, 0x200c);
    }

    #[test]
    fn test_scan_buffer_any() {
        let mut buffer = vec![0u8; 16];
        let val: i32 = 999;
        buffer[0..4].copy_from_slice(&val.to_ne_bytes());
        buffer[8..12].copy_from_slice(&val.to_ne_bytes());

        let mut matches = Vec::new();
        scan_buffer_any(0x3000, &buffer, ScanType::Int32, 4, &mut matches);
        assert_eq!(matches.len(), 4);
    }

    #[test]
    fn test_format_scan_output() {
        let matches = vec![
            ScanMatch {
                address: 0x1000,
                value: ParsedValue::I32(100),
            },
            ScanMatch {
                address: 0x1004,
                value: ParsedValue::I32(200),
            },
        ];
        let out = format_scan_output(&matches, 2, ScanType::Int32);
        assert!(out.contains("2 matches found."));
        assert!(out.contains("0x00001000, 100, int32"));
        assert!(out.contains("0x00001004, 200, int32"));
    }
}
