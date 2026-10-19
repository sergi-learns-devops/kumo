use std::fs;
use std::io::Read;

const REPO: &str = "sergi-learns-devops/kumo";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn self_update() {
    eprintln!("  \x1b[2m⟳ Checking for updates...\x1b[0m");

    // 1. Get latest release from GitHub API
    let latest = match get_latest_version() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("  \x1b[31mError checking for updates: {}\x1b[0m", e);
            std::process::exit(1);
        }
    };

    // 2. Compare versions
    if latest.trim_start_matches('v') == CURRENT_VERSION {
        eprintln!("  \x1b[32m✓ Already up to date (v{})\x1b[0m", CURRENT_VERSION);
        return;
    }

    eprintln!(
        "  \x1b[36m✓ Update available: v{} → v{}\x1b[0m",
        CURRENT_VERSION,
        latest.trim_start_matches('v')
    );

    // 3. Detect platform
    let platform = detect_platform();
    eprintln!("  \x1b[2m⟳ Downloading kumo-{}...\x1b[0m", platform);

    // 4. Download the new binary
    let download_url = format!(
        "https://github.com/{}/releases/download/{}/kumo-{}.tar.gz",
        REPO, latest, platform
    );

    let tar_gz = match download_binary(&download_url) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("  \x1b[31mError downloading update: {}\x1b[0m", e);
            eprintln!("  You can update manually:");
            eprintln!("    curl -fsSL https://raw.githubusercontent.com/{}/main/install.sh | bash", REPO);
            std::process::exit(1);
        }
    };

    // 5. Extract the binary from tar.gz
    let new_binary = match extract_tar_gz(&tar_gz) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("  \x1b[31mError extracting update: {}\x1b[0m", e);
            std::process::exit(1);
        }
    };

    // 6. Get the path of the current executable
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  \x1b[31mError locating current binary: {}\x1b[0m", e);
            std::process::exit(1);
        }
    };

    // Resolve symlinks to get the actual binary path
    let actual_path = match fs::canonicalize(&current_exe) {
        Ok(p) => p,
        Err(_) => current_exe.clone(),
    };

    // 7. Replace the binary atomically: write to tmp, then rename
    let tmp_path = actual_path.with_extension("tmp");

    if let Err(e) = fs::write(&tmp_path, &new_binary) {
        eprintln!("  \x1b[31mError writing update: {}\x1b[0m", e);
        eprintln!("  You may need to run with sudo: sudo kumo --update");
        let _ = fs::remove_file(&tmp_path);
        std::process::exit(1);
    }

    // Set executable permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o755));
    }

    // Rename tmp to actual (atomic on same filesystem)
    if let Err(e) = fs::rename(&tmp_path, &actual_path) {
        eprintln!("  \x1b[31mError replacing binary: {}\x1b[0m", e);
        eprintln!("  You may need to run with sudo: sudo kumo --update");
        let _ = fs::remove_file(&tmp_path);
        std::process::exit(1);
    }

    eprintln!(
        "  \x1b[32m✓ Updated to v{}! Run 'kumo --version' to verify.\x1b[0m",
        latest.trim_start_matches('v')
    );
}

fn get_latest_version() -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);

    let response = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| format!("Failed to reach GitHub: {}", e))?;

    let body = response
        .into_string()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Parse tag_name from JSON response
    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse response: {}", e))?;

    json["tag_name"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No tag_name in release".to_string())
}

fn detect_platform() -> String {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };

    format!("{}-{}", os, arch)
}

fn download_binary(url: &str) -> Result<Vec<u8>, String> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(30))
        .call()
        .map_err(|e| format!("Download failed: {}", e))?;

    let mut buf = Vec::new();
    response
        .into_reader()
        .take(50 * 1024 * 1024) // 50MB max
        .read_to_end(&mut buf)
        .map_err(|e| format!("Failed to read download: {}", e))?;

    Ok(buf)
}

fn extract_tar_gz(data: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Cursor;

    // Decompress gzip
    let cursor = Cursor::new(data);
    let mut decoder = flate2_decode(cursor)?;

    // Read the tar archive and find the "kumo" binary
    let mut tar_data = Vec::new();
    decoder
        .read_to_end(&mut tar_data)
        .map_err(|e| format!("Failed to decompress: {}", e))?;

    // Simple tar extraction: find the file content after the header
    // tar format: 512-byte header blocks, then file data padded to 512 bytes
    let mut offset = 0;
    while offset + 512 <= tar_data.len() {
        let header = &tar_data[offset..offset + 512];

        // Check if this is an empty block (end of archive)
        if header.iter().all(|&b| b == 0) {
            break;
        }

        // Get filename from header (first 100 bytes, null-terminated)
        let name_end = header[..100].iter().position(|&b| b == 0).unwrap_or(100);
        let name = String::from_utf8_lossy(&header[..name_end]);

        // Get file size from header (octal, bytes 124-135)
        let size_str = String::from_utf8_lossy(&header[124..136]);
        let size = usize::from_str_radix(size_str.trim().trim_matches('\0'), 8).unwrap_or(0);

        offset += 512; // Move past header

        if name.contains("kumo") && size > 0 {
            if offset + size <= tar_data.len() {
                return Ok(tar_data[offset..offset + size].to_vec());
            }
        }

        // Skip to next header (size padded to 512 boundary)
        offset += (size + 511) / 512 * 512;
    }

    Err("Could not find kumo binary in archive".to_string())
}

/// Simple gzip decompression using miniz_oxide (no external dependency)
fn flate2_decode(mut reader: impl Read) -> Result<impl Read, String> {
    // Read all compressed data
    let mut compressed = Vec::new();
    reader
        .read_to_end(&mut compressed)
        .map_err(|e| format!("Read error: {}", e))?;

    // Skip gzip header (simplified)
    if compressed.len() < 10 || compressed[0] != 0x1f || compressed[1] != 0x8b {
        return Err("Not a valid gzip file".to_string());
    }

    let mut pos = 10;

    // Check flags
    let flags = compressed[3];
    if flags & 0x04 != 0 {
        // FEXTRA
        if pos + 2 > compressed.len() { return Err("Invalid gzip".to_string()); }
        let xlen = compressed[pos] as usize | (compressed[pos + 1] as usize) << 8;
        pos += 2 + xlen;
    }
    if flags & 0x08 != 0 {
        // FNAME
        while pos < compressed.len() && compressed[pos] != 0 { pos += 1; }
        pos += 1;
    }
    if flags & 0x10 != 0 {
        // FCOMMENT
        while pos < compressed.len() && compressed[pos] != 0 { pos += 1; }
        pos += 1;
    }
    if flags & 0x02 != 0 {
        // FHCRC
        pos += 2;
    }

    if pos >= compressed.len() {
        return Err("Invalid gzip header".to_string());
    }

    // Decompress the DEFLATE data (strip 4-byte CRC + 4-byte size at end)
    let deflate_end = if compressed.len() >= 8 { compressed.len() - 8 } else { compressed.len() };
    let deflate_data = &compressed[pos..deflate_end];

    // Use miniz_oxide for decompression (it's already a transitive dep of ureq/ring)
    let decompressed = miniz_oxide::inflate::decompress_to_vec(deflate_data)
        .map_err(|e| format!("Decompression failed: {:?}", e))?;

    Ok(std::io::Cursor::new(decompressed))
}
