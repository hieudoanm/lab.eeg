use crate::MegWindow;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct BandPower {
    pub band: String,
    pub low_hz: f64,
    pub high_hz: f64,
    pub power: f64,
}

pub fn welch_band_powers(
    window: &MegWindow,
    bands: &BTreeMap<String, (f64, f64)>,
    n_fft: usize,
) -> Vec<BandPower> {
    if window.data.is_empty() || window.times_s.len() < 2 || n_fft == 0 {
        return Vec::new();
    }
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n_fft);
    let mut spectra = Vec::with_capacity(window.data.len());
    for channel in &window.data {
        let mut spectrum = Vec::new();
        let mut offset = 0usize;
        while offset < channel.len() {
            let mut input = vec![Complex::<f32>::new(0.0, 0.0); n_fft];
            let end = (offset + n_fft).min(channel.len());
            for (index, value) in channel[offset..end].iter().enumerate() {
                input[index] = Complex::new(*value as f32, 0.0);
            }
            fft.process(&mut input);
            spectrum.push(input);
            if end == channel.len() {
                break;
            }
            offset += n_fft / 2;
        }
        spectra.push(spectrum);
    }

    let frequency_resolution = window.effective_sampling_frequency_hz / n_fft as f64;
    let mut result = Vec::new();
    for (name, (low, high)) in bands {
        let mut total = 0.0;
        let mut count = 0usize;
        for spectra_for_channel in &spectra {
            for spectrum in spectra_for_channel {
                for (index, value) in spectrum.iter().enumerate() {
                    let frequency = index as f64 * frequency_resolution;
                    if frequency >= *low && frequency <= *high {
                        total += (value.norm_sqr() as f64) / (n_fft * n_fft) as f64;
                        count += 1;
                    }
                }
            }
        }
        result.push(BandPower {
            band: name.clone(),
            low_hz: *low,
            high_hz: *high,
            power: if count == 0 {
                0.0
            } else {
                total / count as f64
            },
        });
    }
    result
}
