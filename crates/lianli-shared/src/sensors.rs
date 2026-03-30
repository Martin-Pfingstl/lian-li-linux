use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TempSource {
    Hwmon {
        name: String,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        device_path: Option<String>,
    },
    NvidiaGpu {
        #[serde(default)]
        gpu_index: u32,
    },
    Command {
        cmd: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorInfo {
    pub source: TempSource,
    pub display_name: String,
    pub current_temp: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum ResolvedSensor {
    SysfsFile(PathBuf),
    NvidiaGpu(u32),
    ShellCommand(String),
}

pub fn enumerate_sensors() -> Vec<SensorInfo> {
    let mut sensors = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();
    let mut raw_entries: Vec<RawHwmonEntry> = Vec::new();

    // Scan hwmon devices
    let hwmon_dir = Path::new("/sys/class/hwmon");
    if let Ok(entries) = std::fs::read_dir(hwmon_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = match std::fs::read_to_string(path.join("name")) {
                Ok(n) => n.trim().to_string(),
                Err(_) => continue,
            };

            let device_path = std::fs::read_link(path.join("device"))
                .ok()
                .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()));

            // Find all temp*_input files
            if let Ok(files) = std::fs::read_dir(&path) {
                for file in files.flatten() {
                    let fname = file.file_name().to_string_lossy().to_string();
                    if fname.starts_with("temp") && fname.ends_with("_input") {
                        let prefix = fname.strip_suffix("_input").unwrap();
                        let label_path = path.join(format!("{prefix}_label"));
                        let label = std::fs::read_to_string(&label_path)
                            .map(|l| l.trim().to_string())
                            .unwrap_or_else(|_| prefix.to_string());

                        let temp = read_sysfs_temp(&file.path());

                        seen.push((name.clone(), label.clone()));
                        raw_entries.push(RawHwmonEntry {
                            name: name.clone(),
                            label,
                            device_path: device_path.clone(),
                            sysfs_path: file.path(),
                            temp,
                        });
                    }
                }
            }
        }
    }

    // Detect name+label collisions and include device_path only where needed
    for entry in &raw_entries {
        let collision = seen
            .iter()
            .filter(|(n, l)| n == &entry.name && l == &entry.label)
            .count()
            > 1;

        let dp = if collision {
            entry.device_path.clone()
        } else {
            None
        };

        let display = if collision {
            format!(
                "{} / {} ({})",
                entry.name,
                entry.label,
                dp.as_deref().unwrap_or("?")
            )
        } else {
            format!("{} / {}", entry.name, entry.label)
        };

        sensors.push(SensorInfo {
            source: TempSource::Hwmon {
                name: entry.name.clone(),
                label: entry.label.clone(),
                device_path: dp,
            },
            display_name: display,
            current_temp: entry.temp,
        });
    }

    // Check for NVIDIA GPU
    if let Ok(output) = Command::new("nvidia-smi")
        .args(["--query-gpu=index,name,temperature.gpu", "--format=csv,noheader,nounits"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(", ").collect();
                if parts.len() >= 3 {
                    let gpu_index: u32 = parts[0].trim().parse().unwrap_or(0);
                    let gpu_name = parts[1].trim();
                    let temp: Option<f32> = parts[2].trim().parse().ok();

                    sensors.push(SensorInfo {
                        source: TempSource::NvidiaGpu { gpu_index },
                        display_name: format!("{gpu_name} (GPU)"),
                        current_temp: temp,
                    });
                }
            }
        }
    }

    sensors
}

pub fn resolve_sensor(source: &TempSource) -> Option<ResolvedSensor> {
    match source {
        TempSource::Hwmon {
            name,
            label,
            device_path,
        } => {
            let hwmon_dir = Path::new("/sys/class/hwmon");
            let entries = std::fs::read_dir(hwmon_dir).ok()?;

            for entry in entries.flatten() {
                let path = entry.path();
                let hw_name = std::fs::read_to_string(path.join("name"))
                    .ok()
                    .map(|n| n.trim().to_string())?;
                if &hw_name != name {
                    continue;
                }

                if let Some(ref dp) = device_path {
                    let actual_dp = std::fs::read_link(path.join("device"))
                        .ok()
                        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()));
                    if actual_dp.as_deref() != Some(dp.as_str()) {
                        continue;
                    }
                }

                // Search temp*_input files for matching label
                if let Ok(files) = std::fs::read_dir(&path) {
                    for file in files.flatten() {
                        let fname = file.file_name().to_string_lossy().to_string();
                        if fname.starts_with("temp") && fname.ends_with("_input") {
                            let prefix = fname.strip_suffix("_input").unwrap();
                            let label_path = path.join(format!("{prefix}_label"));
                            let file_label = std::fs::read_to_string(&label_path)
                                .map(|l| l.trim().to_string())
                                .unwrap_or_else(|_| prefix.to_string());

                            if &file_label == label {
                                return Some(ResolvedSensor::SysfsFile(file.path()));
                            }
                        }
                    }
                }
            }
            None
        }
        TempSource::NvidiaGpu { gpu_index } => Some(ResolvedSensor::NvidiaGpu(*gpu_index)),
        TempSource::Command { cmd } => Some(ResolvedSensor::ShellCommand(cmd.clone())),
    }
}

pub fn read_sensor_temp(resolved: &ResolvedSensor) -> anyhow::Result<f32> {
    match resolved {
        ResolvedSensor::SysfsFile(path) => {
            let content = std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("reading {}: {e}", path.display()))?;
            let millidegrees: f32 = content
                .trim()
                .parse()
                .map_err(|e| anyhow::anyhow!("parsing {}: {e}", path.display()))?;
            Ok(millidegrees / 1000.0)
        }
        ResolvedSensor::NvidiaGpu(index) => {
            let output = Command::new("nvidia-smi")
                .args([
                    "--query-gpu=temperature.gpu",
                    "--format=csv,noheader,nounits",
                    "-i",
                    &index.to_string(),
                ])
                .output()
                .map_err(|e| anyhow::anyhow!("nvidia-smi: {e}"))?;
            if !output.status.success() {
                anyhow::bail!("nvidia-smi failed");
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let temp: f32 = stdout
                .trim()
                .parse()
                .map_err(|e| anyhow::anyhow!("parsing nvidia-smi output: {e}"))?;
            Ok(temp)
        }
        ResolvedSensor::ShellCommand(cmd) => {
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .map_err(|e| anyhow::anyhow!("executing command: {e}"))?;
            if !output.status.success() {
                anyhow::bail!("command failed with status {}", output.status);
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let temp_str = stdout
                .split_whitespace()
                .next()
                .ok_or_else(|| anyhow::anyhow!("empty output"))?;
            let temp: f32 = temp_str
                .parse()
                .map_err(|e| anyhow::anyhow!("parsing '{temp_str}': {e}"))?;
            if !temp.is_finite() {
                anyhow::bail!("value '{temp}' is not finite");
            }
            Ok(temp)
        }
    }
}

#[allow(dead_code)]
struct RawHwmonEntry {
    name: String,
    label: String,
    device_path: Option<String>,
    sysfs_path: PathBuf,
    temp: Option<f32>,
}

fn read_sysfs_temp(path: &Path) -> Option<f32> {
    let content = std::fs::read_to_string(path).ok()?;
    let millidegrees: f32 = content.trim().parse().ok()?;
    Some(millidegrees / 1000.0)
}
