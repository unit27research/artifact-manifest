use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::{DirEntry, WalkDir};

pub const BOUNDARY_NOTE: &str = "This packet records supplied artifacts and declared boundaries. It does not prove that the declared claim is true. It does not verify truth, and it does not certify safety, compliance, or readiness.";

#[derive(Debug, Clone)]
pub struct CreatePacketOptions {
    pub artifact_dir: PathBuf,
    pub declared_claim: String,
    pub supported_scope: String,
    pub limitations: Vec<String>,
    pub allow_risky: bool,
    pub risk_review_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePacket {
    pub packet_version: String,
    pub created_unix_seconds: u64,
    pub declared_claim: String,
    pub supported_scope: String,
    pub limitations: Vec<String>,
    pub risk_review_note: Option<String>,
    pub boundary_note: String,
    pub artifacts: Vec<ArtifactRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub risk_flags: Vec<String>,
}

pub fn create_packet(options: CreatePacketOptions) -> Result<EvidencePacket> {
    if options.allow_risky
        && options
            .risk_review_note
            .as_ref()
            .map(|note| note.trim().is_empty())
            .unwrap_or(true)
    {
        return Err(anyhow!(
            "--risk-reviewed is required when --allow-risky is used"
        ));
    }

    let artifacts = scan_artifacts(&options.artifact_dir, options.allow_risky)?;

    Ok(EvidencePacket {
        packet_version: "0.1".to_string(),
        created_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock is before UNIX epoch")?
            .as_secs(),
        declared_claim: options.declared_claim,
        supported_scope: options.supported_scope,
        limitations: options.limitations,
        risk_review_note: options.risk_review_note,
        boundary_note: BOUNDARY_NOTE.to_string(),
        artifacts,
    })
}

pub fn write_packet(packet: &EvidencePacket, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create output directory {}", output_dir.display()))?;

    let json = serde_json::to_string_pretty(packet).context("failed to serialize packet JSON")?;
    fs::write(output_dir.join("evidence_packet.json"), format!("{json}\n"))
        .context("failed to write evidence_packet.json")?;

    fs::write(
        output_dir.join("EVIDENCE_PACKET.md"),
        render_markdown(packet),
    )
    .context("failed to write EVIDENCE_PACKET.md")?;

    Ok(())
}

pub fn render_markdown(packet: &EvidencePacket) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Evidence Packet\n\n");
    markdown.push_str("## Boundary Note\n\n");
    markdown.push_str(&packet.boundary_note);
    markdown.push_str("\n\n");
    markdown.push_str("## Declared Claim\n\n");
    markdown.push_str(&packet.declared_claim);
    markdown.push_str("\n\n");
    markdown.push_str("## Supported Scope\n\n");
    markdown.push_str(&packet.supported_scope);
    markdown.push_str("\n\n");

    markdown.push_str("## Limitations\n\n");
    if packet.limitations.is_empty() {
        markdown.push_str("- No limitations were declared.\n\n");
    } else {
        for limitation in &packet.limitations {
            markdown.push_str("- ");
            markdown.push_str(limitation);
            markdown.push('\n');
        }
        markdown.push('\n');
    }

    markdown.push_str("## Risk Review\n\n");
    match &packet.risk_review_note {
        Some(note) => {
            markdown.push_str(note);
            markdown.push_str("\n\n");
        }
        None => markdown.push_str("No risky artifacts were explicitly allowed.\n\n"),
    }

    markdown.push_str("## Artifacts\n\n");
    markdown.push_str("| Path | Size Bytes | SHA-256 | Risk Flags |\n");
    markdown.push_str("|---|---:|---|---|\n");
    for artifact in &packet.artifacts {
        let flags = if artifact.risk_flags.is_empty() {
            "none".to_string()
        } else {
            artifact.risk_flags.join(", ")
        };
        markdown.push_str(&format!(
            "| `{}` | {} | `{}` | {} |\n",
            escape_markdown_table_cell(&artifact.path),
            artifact.size_bytes,
            artifact.sha256,
            escape_markdown_table_cell(&flags)
        ));
    }

    markdown
}

pub fn scan_artifacts(artifact_dir: &Path, allow_risky: bool) -> Result<Vec<ArtifactRecord>> {
    if !artifact_dir.exists() {
        return Err(anyhow!(
            "artifact directory does not exist: {}",
            artifact_dir.display()
        ));
    }
    if !artifact_dir.is_dir() {
        return Err(anyhow!(
            "artifact path is not a directory: {}",
            artifact_dir.display()
        ));
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(artifact_dir)
        .into_iter()
        .filter_entry(should_scan_entry)
    {
        let entry = entry.with_context(|| {
            format!(
                "failed to read artifact directory entry under {}",
                artifact_dir.display()
            )
        })?;
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    files.sort();

    let mut artifacts = Vec::new();
    for path in files {
        let relative_path = relative_artifact_path(artifact_dir, &path)?;
        let risk_flags = risk_flags_for(&relative_path);
        if !allow_risky && !risk_flags.is_empty() {
            return Err(anyhow!(
                "risky artifact refused: {} ({})",
                relative_path,
                risk_flags.join(", ")
            ));
        }

        let metadata = fs::metadata(&path)
            .with_context(|| format!("failed to read metadata for {}", path.display()))?;
        artifacts.push(ArtifactRecord {
            path: relative_path,
            size_bytes: metadata.len(),
            sha256: sha256_file(&path)?,
            risk_flags,
        });
    }

    if artifacts.is_empty() {
        return Err(anyhow!(
            "artifact directory contains no files: {}",
            artifact_dir.display()
        ));
    }

    Ok(artifacts)
}

fn should_scan_entry(entry: &DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }

    let name = entry.file_name().to_string_lossy();
    !matches!(
        name.as_ref(),
        ".git" | "target" | "node_modules" | "__pycache__" | ".next"
    )
}

fn relative_artifact_path(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .with_context(|| format!("failed to calculate relative path for {}", path.display()))?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn risk_flags_for(relative_path: &str) -> Vec<String> {
    let lower = relative_path.to_ascii_lowercase();
    let file_name = Path::new(relative_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    let mut flags = Vec::new();
    if file_name == ".env"
        || file_name.starts_with(".env.")
        || file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".p12")
        || file_name.ends_with(".pfx")
        || file_name.ends_with(".crt")
        || file_name == "id_rsa"
        || file_name == "id_dsa"
        || file_name.contains("secret")
        || file_name.contains("token")
        || file_name.contains("credential")
        || lower.contains("private")
    {
        flags.push("possible_private_or_secret_file".to_string());
    }
    if lower.contains("cache")
        || lower.contains("/tmp/")
        || lower.contains("scratch")
        || file_name.ends_with(".sqlite")
        || file_name.ends_with(".db")
    {
        flags.push("scratch_or_cache_artifact".to_string());
    }
    if file_name == ".ds_store" || file_name.ends_with(".pyc") {
        flags.push("generated_or_local_machine_artifact".to_string());
    }
    flags
}

fn escape_markdown_table_cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('`', "\\`")
        .replace(['\n', '\r'], " ")
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file =
        fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
