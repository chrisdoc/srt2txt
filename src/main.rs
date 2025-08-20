use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, ArgAction};
use rayon::prelude::*;
use regex::Regex;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Cli {
    /// SRT input file(s) or directory(ies)
    inputs: Vec<PathBuf>,

    /// Output directory (created if missing). If omitted and --stdout not set, writes next to source.
    #[arg(short, long)]
    output_dir: Option<PathBuf>,

    /// Write combined output of all inputs to stdout
    #[arg(long, action=ArgAction::SetTrue)]
    stdout: bool,

    /// Join all processed captions into a single output file instead of per-file outputs (implies not stdout)
    #[arg(long)]
    join: bool,

    /// Output file name when using --join (default: combined.txt)
    #[arg(long, default_value = "combined.txt")]
    join_name: String,

    /// Overwrite existing output files
    #[arg(short, long, action=ArgAction::SetTrue)]
    force: bool,

    /// Collapse multiple blank lines into one
    #[arg(long, action=ArgAction::SetTrue)]
    collapse_blank: bool,

    /// Remove immediately repeated lines (after trimming)
    #[arg(long, action=ArgAction::SetTrue)]
    remove_duplicates: bool,

    /// Join all captions into one continuous block (sentences) removing blank lines
    #[arg(long, action=ArgAction::SetTrue)]
    join_sentences: bool,
}

#[derive(thiserror::Error, Debug)]
enum SrtError {
    #[error("Malformed timestamp line: {0}")]
    BadTimestamp(String),
}

#[derive(Debug, Clone)]
struct Caption { start_ms: u64, end_ms: u64, text: String }

fn parse_timestamp(ts: &str) -> Option<u64> {
    // format: HH:MM:SS,mmm
    let mut parts = ts.split(|c| c == ':' || c == ',');
    let h: u64 = parts.next()?.parse().ok()?;
    let m: u64 = parts.next()?.parse().ok()?;
    let s: u64 = parts.next()?.parse().ok()?;
    let ms: u64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() { return None; }
    Some(h*3600_000 + m*60_000 + s*1000 + ms)
}

fn parse_srt(path: &Path) -> Result<Vec<Caption>> {
    let file = File::open(path).with_context(|| format!("Opening {:?}", path))?;
    let reader = BufReader::new(file);
    let mut captions = Vec::new();

    let mut lines = reader.lines().peekable();
    while let Some(line) = lines.next() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        // sequence number (may ignore parse errors)
        if line.trim().parse::<u32>().is_err() { continue; }
        let ts_line = match lines.next() { Some(l)=>l?, None=>break };
        if let Some((a,b)) = ts_line.split_once(" --> ") {
            if let (Some(start), Some(end)) = (parse_timestamp(a.trim()), parse_timestamp(b.trim())) {
                // collect text lines until blank
                let mut text_lines = Vec::new();
                while let Some(peek) = lines.peek() {
                    if peek.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) { break; }
                    let t = lines.next().unwrap()?; // safe unwrap
                    text_lines.push(t);
                }
                // consume blank
                if let Some(peek) = lines.peek() { if peek.as_ref().ok().map(|s| s.trim().is_empty()).unwrap_or(false) { lines.next(); } }

                let text = clean_text(&text_lines.join("\n"));
                captions.push(Caption { start_ms: start, end_ms: end, text });
            } else {
                // malformed timestamp line, skip block
                continue;
            }
        } else {
            continue; // malformed
        }
    }
    Ok(captions)
}

fn clean_text(raw: &str) -> String {
    // Remove HTML tags
    lazy_static::lazy_static! {
        static ref TAG_RE: Regex = Regex::new(r"<[^>]+>").unwrap();
        static ref SPACE_RE: Regex = Regex::new(r"\s+").unwrap();
    }
    let without_tags = TAG_RE.replace_all(raw, "");
    // Replace line breaks inside a caption with space
    let flattened = without_tags.replace('\n', " ");
    // Collapse multiple spaces
    let collapsed = SPACE_RE.replace_all(&flattened, " ");
    collapsed.trim().to_string()
}

fn post_process(texts: Vec<String>, collapse_blank: bool, dedup: bool) -> String {
    let mut out = Vec::new();
    let mut last_line: Option<String> = None;
    for t in texts.into_iter() {
        if t.is_empty() { continue; }
        if dedup {
            if let Some(ref last) = last_line { if last == &t { continue; } }
        }
        out.push(t.clone());
        last_line = Some(t);
    }
    let joined = out.join("\n\n");
    if collapse_blank {
        let re = Regex::new(r"\n{3,}").unwrap();
        re.replace_all(&joined, "\n\n").to_string()
    } else { joined }
}

fn process_file(path: &Path, cli: &Cli) -> Result<String> {
    let captions = parse_srt(path)?;
    let texts: Vec<String> = captions.into_iter().map(|c| c.text).collect();
    if cli.join_sentences {
        Ok(join_sentences(texts))
    } else {
        Ok(post_process(texts, cli.collapse_blank, cli.remove_duplicates))
    }
}

fn join_sentences(texts: Vec<String>) -> String {
    let mut out = String::new();
    for t in texts {
        if t.is_empty() { continue; }
        if !out.is_empty() {
            let last = out.chars().rev().find(|c| !c.is_whitespace());
            if let Some(ch) = last {
                if !ch.is_whitespace() { out.push(' '); }
            }
        }
        out.push_str(t.trim());
    }
    out
}

fn write_output(path: &Path, content: &str, cli: &Cli, base_input: &Path) -> Result<()> {
    if cli.stdout { println!("{}", content); return Ok(()); }
    let out_dir = if let Some(dir) = &cli.output_dir { dir.clone() } else { base_input.parent().unwrap_or(Path::new(".")).to_path_buf() };
    if !out_dir.exists() { fs::create_dir_all(&out_dir)?; }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let out_path = out_dir.join(format!("{stem}.txt"));
    if out_path.exists() && !cli.force { anyhow::bail!("Refusing to overwrite {:?} (use --force)", out_path); }
    fs::write(&out_path, content)?;
    eprintln!("Wrote {:?}", out_path);
    Ok(())
}

fn gather_input_files(inputs: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for input in inputs {
        if input.is_file() {
            if input.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("srt")).unwrap_or(false) { files.push(input.clone()); }
        } else if input.is_dir() {
            for entry in WalkDir::new(input).into_iter().filter_map(Result::ok) {
                if entry.file_type().is_file() {
                    let p = entry.path();
                    if p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("srt")).unwrap_or(false) { files.push(p.to_path_buf()); }
                }
            }
        }
    }
    files
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.inputs.is_empty() { anyhow::bail!("Provide at least one input file or directory"); }
    if cli.stdout && cli.join { anyhow::bail!("--stdout and --join are mutually exclusive"); }
    if cli.join_sentences && cli.join { anyhow::bail!("--join-sentences and --join cannot be combined"); }
    if cli.join_sentences && cli.collapse_blank { eprintln!("Warning: --collapse-blank ignored with --join-sentences"); }

    let files = gather_input_files(&cli.inputs);
    if files.is_empty() { anyhow::bail!("No .srt files found"); }

    if cli.join {
        let contents: Vec<_> = files.par_iter().map(|f| process_file(f, &cli).with_context(|| format!("Processing {:?}", f))).collect::<Result<_>>()?;
    let joined = if cli.join_sentences { join_sentences(contents) } else { post_process(contents, cli.collapse_blank, cli.remove_duplicates) };
        let out_dir = cli.output_dir.clone().unwrap_or_else(|| PathBuf::from("."));
        if !out_dir.exists() { fs::create_dir_all(&out_dir)?; }
        let out_path = out_dir.join(&cli.join_name);
        if out_path.exists() && !cli.force { anyhow::bail!("Refusing to overwrite {:?} (use --force)", out_path); }
        fs::write(&out_path, joined)?;
        eprintln!("Wrote {:?}", out_path);
        return Ok(());
    }

    files.par_iter().try_for_each(|f| {
        let content = process_file(f, &cli)?;
        write_output(f, &content, &cli, f)
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write as _;

    const SIMPLE_SRT: &str = "1\n00:00:00,000 --> 00:00:01,000\n<i>Hello</i> world!\n\n2\n00:00:01,500 --> 00:00:03,000\nSecond  line.\nLine continued.\n\n2\n00:00:04,000 --> 00:00:05,000\nSecond  line.\nLine continued.\n"; // duplicate block intentional

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(parse_timestamp("00:00:10,250"), Some(10_250));
        assert_eq!(parse_timestamp("01:01:01,001"), Some(3_661_001));
        assert_eq!(parse_timestamp("bad"), None);
    }

    #[test]
    fn test_parse_and_clean() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "{}", SIMPLE_SRT).unwrap();
        let captions = parse_srt(tmp.path()).unwrap();
        assert_eq!(captions.len(), 3); // duplicate seq number still counted if timestamp valid
        assert_eq!(captions[0].text, "Hello world!");
        assert_eq!(captions[1].text, "Second line. Line continued.");
    }

    #[test]
    fn test_post_process_dedup() {
        let texts = vec![
            "A first".to_string(),
            "A first".to_string(),
            "Different".to_string(),
            "Different".to_string(),
        ];
        let out = post_process(texts, true, true);
        // Expect only one blank line separator and duplicates removed
        assert!(out.contains("A first"));
        assert!(out.contains("Different"));
        assert!(!out.contains("A first\n\nA first"));
    }
}
