use neuro_core::CoreError;
use serde_json::{json, Value};
use std::f64::consts::PI;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct SampleConfig {
    pub magnetometers: usize,
    pub gradiometers: usize,
    pub auxiliary: usize,
    pub sampling_frequency_hz: f64,
    pub duration_seconds: f64,
    pub stimulus_interval_s: f64,
    pub seed: u64,
}

impl Default for SampleConfig {
    fn default() -> Self {
        Self {
            magnetometers: 32,
            gradiometers: 64,
            auxiliary: 9,
            sampling_frequency_hz: 250.0,
            duration_seconds: 30.0,
            stimulus_interval_s: 1.5,
            seed: 42,
        }
    }
}

struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn normal(&mut self) -> f64 {
        let u1 = self.uniform().max(1e-12);
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
    }
}

/// Generates a deterministic synthetic OPM-style recording: magnetometer and
/// gradiometer channels with evoked responses plus auxiliary EOG/ECG/stim
/// channels, written as interleaved f32 `.bin` with a JSON sidecar.
pub fn generate_sample_recording(
    directory: &Path,
    config: &SampleConfig,
) -> Result<Value, CoreError> {
    if config.magnetometers == 0 && config.gradiometers == 0 {
        return Err(CoreError::InvalidInput(
            "recording must contain MEG channels".into(),
        ));
    }
    if config.sampling_frequency_hz <= 0.0 || config.duration_seconds <= 0.0 {
        return Err(CoreError::InvalidInput(
            "sampling frequency and duration must be positive".into(),
        ));
    }
    std::fs::create_dir_all(directory)?;

    let sfreq = config.sampling_frequency_hz;
    let n_times = (config.duration_seconds * sfreq).round() as usize;
    let n_meg = config.magnetometers + config.gradiometers;
    let n_channels = n_meg + config.auxiliary;
    let mut rng = Rng(config.seed);

    let mut data = vec![vec![0.0f64; n_times]; n_channels];
    for ch in 0..config.magnetometers {
        let scale = 2e-13;
        for (i, v) in data[ch].iter_mut().enumerate() {
            *v = scale * (i as f64 * 0.001).sin() + scale * rng.normal();
        }
    }
    for ch in config.magnetometers..n_meg {
        let scale = 3e-14;
        for (i, v) in data[ch].iter_mut().enumerate() {
            *v = scale * rng.normal() + scale * 0.5 * (i as f64 * 0.002).cos();
        }
    }

    let n_events = (config.duration_seconds / config.stimulus_interval_s).floor() as usize;
    let response_len = (0.25 * sfreq).round() as usize;
    for k in 0..n_events {
        let onset = (k as f64 * config.stimulus_interval_s * sfreq).round() as usize;
        if onset + response_len > n_times {
            break;
        }
        let amplitude = if k % 2 == 0 { 8e-13 } else { 4e-13 };
        for ch in 0..n_meg {
            for j in 0..response_len {
                let envelope = (-(j as f64 / response_len as f64) * 6.0).exp()
                    * (4.0 * PI * j as f64 / response_len as f64).sin();
                data[ch][onset + j] += amplitude * envelope;
            }
        }
    }

    // Auxiliary channels: EOG drift, ECG pulses, stimulus marker, misc noise.
    let aux_base = n_meg;
    if config.auxiliary > 0 {
        for i in 0..n_times {
            let t = i as f64 / sfreq;
            data[aux_base][i] = 200e-6 * (2.0 * PI * 0.3 * t).sin() + 20e-6 * rng.normal();
        }
    }
    if config.auxiliary > 1 {
        for i in 0..n_times {
            let t = i as f64 / sfreq;
            data[aux_base + 1][i] =
                150e-6 * (2.0 * PI * 0.25 * t + 1.0).sin() + 20e-6 * rng.normal();
        }
    }
    if config.auxiliary > 2 {
        for i in 0..n_times {
            data[aux_base + 2][i] = 100e-6 * rng.normal();
        }
    }
    if config.auxiliary > 3 {
        let mut beat = 0usize;
        while beat as f64 / 1.2 * sfreq + 20.0 <= n_times as f64 {
            let centre = (beat as f64 / 1.2 * sfreq).round() as usize;
            for j in 0..20.min(n_times - centre) {
                let envelope = (-(j as f64 / 20.0) * 5.0).exp();
                data[aux_base + 3][centre + j] += 1.2e-3 * envelope;
            }
            beat += 1;
        }
        for i in 0..n_times {
            data[aux_base + 3][i] += 10e-6 * rng.normal();
        }
    }
    if config.auxiliary > 4 {
        for k in 0..n_events {
            let onset = (k as f64 * config.stimulus_interval_s * sfreq).round() as usize;
            let code = if k % 2 == 0 { 1.0 } else { 2.0 };
            for j in 0..50.min(n_times.saturating_sub(onset)) {
                data[aux_base + 4][onset + j] = code;
            }
        }
    }
    for extra in 5..config.auxiliary.min(8) {
        for i in 0..n_times {
            data[aux_base + extra][i] = 0.5 * rng.normal();
        }
    }

    // Interleaved f32 little-endian payload.
    let mut bytes = Vec::with_capacity(n_times * n_channels * 4);
    for sample in 0..n_times {
        for ch in 0..n_channels {
            bytes.extend_from_slice(&(data[ch][sample] as f32).to_le_bytes());
        }
    }
    let bin_path = directory.join("data.bin");
    std::fs::write(&bin_path, &bytes)?;

    let mut channels = Vec::with_capacity(n_channels);
    let mut push = |name: String, kind: &str, unit: &str| {
        channels.push(json!({
            "name": name,
            "type": kind,
            "unit": unit,
        }));
    };
    for i in 0..config.magnetometers {
        push(format!("MEG {:04}", i + 1), "mag", "tesla");
    }
    for i in 0..config.gradiometers {
        push(
            format!("MEG {:04}", config.magnetometers + i + 1),
            "grad",
            "tesla_per_metre",
        );
    }
    let aux_names = [
        "EOG 001", "EOG 002", "EOG 003", "ECG 001", "STI 014", "MISC 001", "MISC 002", "MISC 003",
        "STI 015",
    ];
    let aux_kinds = [
        "eog", "eog", "eog", "ecg", "stim", "misc", "misc", "misc", "stim",
    ];
    let aux_units = [
        "volt",
        "volt",
        "volt",
        "volt",
        "dimensionless",
        "other",
        "other",
        "other",
        "dimensionless",
    ];
    for i in 0..config.auxiliary {
        let idx = i.min(aux_names.len() - 1);
        push(aux_names[idx].to_string(), aux_kinds[idx], aux_units[idx]);
    }
    let events: Vec<Value> = (0..n_events)
        .map(|k| {
            json!({
                "sample": (k as f64 * config.stimulus_interval_s * sfreq).round() as usize,
                "time_s": k as f64 * config.stimulus_interval_s,
                "code": if k % 2 == 0 { "auditory" } else { "visual" },
                "value": if k % 2 == 0 { 1 } else { 2 },
            })
        })
        .collect();

    let header = json!({
        "name": "Synthetic OPM sample",
        "format": "ucl_opm_bin",
        "sampling_frequency_hz": sfreq,
        "duration_seconds": config.duration_seconds,
        "n_channels": n_channels,
        "dtype": "float32",
        "sensor_type": "opm_vector",
        "coordinate_frame": "head",
        "channels": channels,
        "event_sets": [{
            "name": "stimulus_events",
            "source": "synthetic",
            "sample_convention": "onset",
            "events": events,
            "metadata": {
                "event_id_mapping": {"1": "auditory tone", "2": "visual flash"},
                "stim_channel": "STI 014",
            },
        }],
    });
    std::fs::write(
        directory.join("header.json"),
        serde_json::to_vec_pretty(&header)?,
    )?;

    Ok(json!({
        "directory": directory.to_string_lossy(),
        "channels": n_channels,
        "samples": n_times,
        "size_bytes": bytes.len(),
    }))
}
