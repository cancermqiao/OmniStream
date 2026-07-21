use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::recording;

pub(crate) const MIN_RECORDING_FREE_SPACE_PERCENT: f64 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DiskSpaceSnapshot {
    pub(crate) path: PathBuf,
    pub(crate) total_kb: u64,
    pub(crate) available_kb: u64,
    pub(crate) free_percent: f64,
}

pub(crate) async fn recording_storage_below_min_free_percent()
-> Result<Option<DiskSpaceSnapshot>, String> {
    let root = recording::recording_root_dir();
    let snapshot = disk_space_snapshot(&root).await?;
    if snapshot.free_percent < MIN_RECORDING_FREE_SPACE_PERCENT {
        Ok(Some(snapshot))
    } else {
        Ok(None)
    }
}

pub(crate) async fn recording_disk_space_snapshot() -> Result<DiskSpaceSnapshot, String> {
    disk_space_snapshot(&recording::recording_root_dir()).await
}

async fn disk_space_snapshot(path: &Path) -> Result<DiskSpaceSnapshot, String> {
    let inspect_path = nearest_existing_path(path);
    let output = Command::new("df")
        .arg("-Pk")
        .arg(&inspect_path)
        .output()
        .await
        .map_err(|e| format!("failed to execute df for {}: {e}", inspect_path.display()))?;

    if !output.status.success() {
        return Err(format!(
            "df failed for {} with status {}: {}",
            inspect_path.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    parse_df_output(&String::from_utf8_lossy(&output.stdout), inspect_path)
}

fn nearest_existing_path(path: &Path) -> PathBuf {
    let mut current = path;
    loop {
        if current.exists() {
            return current.to_path_buf();
        }
        let Some(parent) = current.parent() else {
            return PathBuf::from(".");
        };
        current = parent;
    }
}

fn parse_df_output(output: &str, path: PathBuf) -> Result<DiskSpaceSnapshot, String> {
    let line = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .nth(1)
        .ok_or_else(|| format!("df output missing data line: {output:?}"))?;
    let fields = line.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 4 {
        return Err(format!("df output data line is invalid: {line:?}"));
    }

    let total_kb = fields[1]
        .parse::<u64>()
        .map_err(|e| format!("failed to parse df total blocks from {line:?}: {e}"))?;
    let available_kb = fields[3]
        .parse::<u64>()
        .map_err(|e| format!("failed to parse df available blocks from {line:?}: {e}"))?;
    let free_percent =
        if total_kb == 0 { 0.0 } else { (available_kb as f64 / total_kb as f64) * 100.0 };

    Ok(DiskSpaceSnapshot { path, total_kb, available_kb, free_percent })
}

#[cfg(test)]
mod tests {
    use super::{MIN_RECORDING_FREE_SPACE_PERCENT, parse_df_output};
    use std::path::PathBuf;

    #[test]
    fn parse_df_output_calculates_free_percent() {
        let output = "\
Filesystem     1024-blocks    Used Available Capacity Mounted on
/dev/vda1         10000000 9900000    100000      99% /
";

        let snapshot = parse_df_output(output, PathBuf::from("/data")).expect("valid df output");

        assert_eq!(snapshot.total_kb, 10_000_000);
        assert_eq!(snapshot.available_kb, 100_000);
        assert!((snapshot.free_percent - 1.0).abs() < f64::EPSILON);
        assert!(snapshot.free_percent < MIN_RECORDING_FREE_SPACE_PERCENT);
    }

    #[test]
    fn parse_df_output_rejects_missing_data_line() {
        let err = parse_df_output(
            "Filesystem 1024-blocks Used Available Capacity Mounted on\n",
            PathBuf::from("/data"),
        )
        .expect_err("missing data line should fail");

        assert!(err.contains("missing data line"));
    }
}
