use sanos::calibration::calibrate_with_stats;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to resolve repository root")
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .expect("failed to list directory")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    out.sort();
    out
}

#[test]
fn repo_configs_parse_and_calibrate_on_reference_snapshot() {
    let root = repo_root();
    let snapshot_path = root.join("data/snapshots/tv_equity_like.json");
    let config_dir = root.join("data/configs");

    let snapshot = sanos_io::load_iv_snapshot_v1(&snapshot_path).unwrap_or_else(|err| {
        panic!(
            "failed to load snapshot {}: {err:#}",
            snapshot_path.display()
        )
    });
    let book = sanos_io::snapshot_v1_to_option_book(&snapshot).unwrap_or_else(|err| {
        panic!(
            "failed to convert snapshot {}: {err:#}",
            snapshot_path.display()
        )
    });

    let configs = json_files(&config_dir);
    assert!(
        !configs.is_empty(),
        "no config files found in {}",
        config_dir.display()
    );

    for cfg_path in configs {
        let cfg = sanos_io::load_calibration_config(&cfg_path)
            .unwrap_or_else(|err| panic!("failed to parse config {}: {err:#}", cfg_path.display()));

        calibrate_with_stats(&book, &cfg)
            .unwrap_or_else(|err| panic!("calibration failed for {}: {err:?}", cfg_path.display()));
    }
}
