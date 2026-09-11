//! Windows Portable Executable (PE) import inspection utilities.
//!
//! Scans PE32 / PE32+ binaries in memory to detect dynamic dependencies,
//! specifically Microsoft Visual C++ Runtime libraries (`VCRUNTIME*.dll`, `MSVCP*.dll`).

#[derive(Debug, PartialEq, Eq)]
pub enum PeScanError {
    TooSmall,
    InvalidDosSignature,
    InvalidPeOffset,
    InvalidPeSignature,
    InvalidSectionTable,
    InvalidImportTable,
}

impl std::fmt::Display for PeScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeScanError::TooSmall => write!(f, "File is too small to be a PE binary"),
            PeScanError::InvalidDosSignature => write!(f, "Missing MZ DOS signature"),
            PeScanError::InvalidPeOffset => write!(f, "Invalid PE header offset in DOS header"),
            PeScanError::InvalidPeSignature => write!(f, "Missing PE\\0\\0 signature"),
            PeScanError::InvalidSectionTable => {
                write!(f, "Corrupted or out-of-bounds section table")
            }
            PeScanError::InvalidImportTable => write!(f, "Corrupted or out-of-bounds import table"),
        }
    }
}

impl std::error::Error for PeScanError {}

#[derive(Debug, Clone)]
struct SectionHeader {
    virtual_address: u32,
    virtual_size: u32,
    pointer_to_raw_data: u32,
    size_of_raw_data: u32,
}

fn rva_to_offset(rva: u32, sections: &[SectionHeader]) -> Option<usize> {
    for s in sections {
        let size = std::cmp::max(s.virtual_size, s.size_of_raw_data);
        if rva >= s.virtual_address && rva < s.virtual_address.saturating_add(size) {
            let offset_in_sec = rva - s.virtual_address;
            if offset_in_sec < s.size_of_raw_data {
                return Some((s.pointer_to_raw_data + offset_in_sec) as usize);
            }
        }
    }
    None
}

/// Extract names of imported DLLs from an in-memory Windows PE executable.
pub fn scan_pe_imports(bytes: &[u8]) -> Result<Vec<String>, PeScanError> {
    if bytes.len() < 0x40 {
        return Err(PeScanError::TooSmall);
    }
    if &bytes[0..2] != b"MZ" {
        return Err(PeScanError::InvalidDosSignature);
    }

    let e_lfanew = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap()) as usize;
    if e_lfanew + 24 > bytes.len() {
        return Err(PeScanError::InvalidPeOffset);
    }
    if &bytes[e_lfanew..e_lfanew + 4] != b"PE\0\0" {
        return Err(PeScanError::InvalidPeSignature);
    }

    let file_header_offset = e_lfanew + 4;
    let num_sections = u16::from_le_bytes(
        bytes[file_header_offset + 2..file_header_offset + 4]
            .try_into()
            .unwrap(),
    ) as usize;
    let size_of_optional_header = u16::from_le_bytes(
        bytes[file_header_offset + 16..file_header_offset + 18]
            .try_into()
            .unwrap(),
    ) as usize;

    let opt_header_offset = file_header_offset + 20;
    if opt_header_offset + size_of_optional_header > bytes.len() {
        return Err(PeScanError::InvalidPeOffset);
    }

    let magic = u16::from_le_bytes(
        bytes[opt_header_offset..opt_header_offset + 2]
            .try_into()
            .unwrap(),
    );
    let import_dir_offset = match magic {
        0x010b => opt_header_offset + 96 + 8,  // PE32: DataDirectory[1]
        0x020b => opt_header_offset + 112 + 8, // PE32+: DataDirectory[1]
        _ => return Err(PeScanError::InvalidPeSignature),
    };

    if import_dir_offset + 8 > opt_header_offset + size_of_optional_header {
        return Ok(Vec::new()); // No import directory
    }

    let import_rva = u32::from_le_bytes(
        bytes[import_dir_offset..import_dir_offset + 4]
            .try_into()
            .unwrap(),
    );
    let import_size = u32::from_le_bytes(
        bytes[import_dir_offset + 4..import_dir_offset + 8]
            .try_into()
            .unwrap(),
    );
    if import_rva == 0 || import_size == 0 {
        return Ok(Vec::new());
    }

    let section_table_offset = opt_header_offset + size_of_optional_header;
    let mut sections = Vec::with_capacity(num_sections);
    for i in 0..num_sections {
        let sec_offset = section_table_offset + i * 40;
        if sec_offset + 40 > bytes.len() {
            return Err(PeScanError::InvalidSectionTable);
        }
        let virtual_size =
            u32::from_le_bytes(bytes[sec_offset + 8..sec_offset + 12].try_into().unwrap());
        let virtual_address =
            u32::from_le_bytes(bytes[sec_offset + 12..sec_offset + 16].try_into().unwrap());
        let size_of_raw_data =
            u32::from_le_bytes(bytes[sec_offset + 16..sec_offset + 20].try_into().unwrap());
        let pointer_to_raw_data =
            u32::from_le_bytes(bytes[sec_offset + 20..sec_offset + 24].try_into().unwrap());

        sections.push(SectionHeader {
            virtual_address,
            virtual_size,
            pointer_to_raw_data,
            size_of_raw_data,
        });
    }

    let import_file_offset = match rva_to_offset(import_rva, &sections) {
        Some(offset) => offset,
        None => return Err(PeScanError::InvalidImportTable),
    };

    let mut dll_names = Vec::new();
    let mut curr_desc = import_file_offset;

    loop {
        if curr_desc + 20 > bytes.len() {
            break;
        }
        let is_null_desc = bytes[curr_desc..curr_desc + 20].iter().all(|&b| b == 0);
        if is_null_desc {
            break;
        }

        let name_rva =
            u32::from_le_bytes(bytes[curr_desc + 12..curr_desc + 16].try_into().unwrap());
        if let Some(name_offset) = rva_to_offset(name_rva, &sections) {
            if name_offset < bytes.len() {
                let name_bytes = &bytes[name_offset..];
                let len = name_bytes.iter().position(|&b| b == 0).unwrap_or(0);
                if let Ok(name) = std::str::from_utf8(&name_bytes[..len]) {
                    if !name.is_empty() {
                        dll_names.push(name.to_string());
                    }
                }
            }
        }

        curr_desc += 20;
    }

    Ok(dll_names)
}

/// Filter a list of imported DLL names to only those matching dynamic MSVC runtime libraries.
pub fn detect_dynamic_msvc_imports(imports: &[String]) -> Vec<String> {
    imports
        .iter()
        .filter(|dll| {
            let lower = dll.to_ascii_lowercase();
            lower.starts_with("vcruntime") || lower.starts_with("msvcp")
        })
        .cloned()
        .collect()
}

/// Scans a binary in memory and returns any dynamic MSVC runtime imports detected.
pub fn has_dynamic_msvc_imports(bytes: &[u8]) -> Result<Vec<String>, PeScanError> {
    let imports = scan_pe_imports(bytes)?;
    Ok(detect_dynamic_msvc_imports(&imports))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Build a minimal synthetic PE32+ file buffer with a custom list of imported DLL names.
    pub fn build_synthetic_pe(dll_names: &[&str]) -> Vec<u8> {
        let mut buf = vec![0u8; 1024];

        // DOS Header
        buf[0..2].copy_from_slice(b"MZ");
        let e_lfanew: u32 = 0x80;
        buf[0x3c..0x40].copy_from_slice(&e_lfanew.to_le_bytes());

        // PE Header at 0x80
        let pe_offset = 0x80;
        buf[pe_offset..pe_offset + 4].copy_from_slice(b"PE\0\0");

        // COFF File Header (20 bytes)
        let file_header = pe_offset + 4;
        buf[file_header..file_header + 2].copy_from_slice(&0x8664u16.to_le_bytes()); // AMD64
        buf[file_header + 2..file_header + 4].copy_from_slice(&1u16.to_le_bytes()); // 1 section
        let opt_hdr_size: u16 = 240;
        buf[file_header + 16..file_header + 18].copy_from_slice(&opt_hdr_size.to_le_bytes());

        // Optional Header (PE32+)
        let opt_header = file_header + 20;
        buf[opt_header..opt_header + 2].copy_from_slice(&0x020Bu16.to_le_bytes()); // PE32+ magic

        // Data Directory: Import Table (Index 1) at opt_header + 112 + 8
        let import_dir_entry = opt_header + 120;
        let rdata_rva = 0x1000u32;
        let import_rva = rdata_rva;
        let import_size = (dll_names.len() as u32 + 1) * 20;
        buf[import_dir_entry..import_dir_entry + 4].copy_from_slice(&import_rva.to_le_bytes());
        buf[import_dir_entry + 4..import_dir_entry + 8].copy_from_slice(&import_size.to_le_bytes());

        // Section Table (1 section: .rdata)
        let section_table = opt_header + opt_hdr_size as usize;
        buf[section_table..section_table + 6].copy_from_slice(b".rdata");
        let sec_raw_ptr = 0x200u32; // raw file offset 512
        let sec_raw_size = 512u32;
        buf[section_table + 8..section_table + 12].copy_from_slice(&sec_raw_size.to_le_bytes()); // VirtualSize
        buf[section_table + 12..section_table + 16].copy_from_slice(&rdata_rva.to_le_bytes()); // VirtualAddress
        buf[section_table + 16..section_table + 20].copy_from_slice(&sec_raw_size.to_le_bytes()); // SizeOfRawData
        buf[section_table + 20..section_table + 24].copy_from_slice(&sec_raw_ptr.to_le_bytes()); // PointerToRawData

        // Populate Import Descriptors and Strings in .rdata (starting at offset 512)
        let mut desc_offset = sec_raw_ptr as usize;
        let mut string_offset = desc_offset + (dll_names.len() + 1) * 20;

        for dll in dll_names {
            let str_rva = rdata_rva + (string_offset - sec_raw_ptr as usize) as u32;
            // Write descriptor Name RVA at desc_offset + 12
            buf[desc_offset + 12..desc_offset + 16].copy_from_slice(&str_rva.to_le_bytes());

            // Write string at string_offset
            let dll_bytes = dll.as_bytes();
            buf[string_offset..string_offset + dll_bytes.len()].copy_from_slice(dll_bytes);
            buf[string_offset + dll_bytes.len()] = 0; // null terminator

            string_offset += dll_bytes.len() + 1;
            desc_offset += 20;
        }
        // Null descriptor is already all zeroes

        buf
    }

    #[test]
    fn pe_import_scanner_detects_dynamic_msvc_imports() {
        let pe_bytes = build_synthetic_pe(&[
            "KERNEL32.dll",
            "VCRUNTIME140.dll",
            "MSVCP140.dll",
            "USER32.dll",
        ]);
        let imports = scan_pe_imports(&pe_bytes).expect("synthetic PE must scan successfully");
        assert_eq!(imports.len(), 4);
        assert_eq!(imports[0], "KERNEL32.dll");
        assert_eq!(imports[1], "VCRUNTIME140.dll");
        assert_eq!(imports[2], "MSVCP140.dll");
        assert_eq!(imports[3], "USER32.dll");

        let dynamic_crt = detect_dynamic_msvc_imports(&imports);
        assert_eq!(dynamic_crt, vec!["VCRUNTIME140.dll", "MSVCP140.dll"]);
    }

    #[test]
    fn pe_import_scanner_passes_clean_static_binary() {
        let pe_bytes = build_synthetic_pe(&["KERNEL32.dll", "USER32.dll", "ADVAPI32.dll"]);
        let msvc_imports = has_dynamic_msvc_imports(&pe_bytes).expect("scan must succeed");
        assert!(
            msvc_imports.is_empty(),
            "static binary must have zero dynamic MSVC CRT imports, found: {:?}",
            msvc_imports
        );
    }
}
