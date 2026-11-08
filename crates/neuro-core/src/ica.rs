//! Deterministic symmetric FastICA for EEG artifact pipelines.
//!
//! Whitening is PCA-based (Jacobi eigendecomposition of the channel
//! covariance); the unmixing iteration uses the `tanh` nonlinearity with a
//! fixed-seed deterministic initial matrix so results are reproducible.
//! `reconstruct` supports component rejection by zeroing selected sources
//! before the inverse transform.

use crate::CoreError;

/// Fitted ICA model: whitening + unmixing matrices and component sources.
#[derive(Clone, Debug)]
pub struct IcaModel {
    /// Per-channel mean removed before whitening.
    pub mean: Vec<f64>,
    /// `n_components x n_channels`: `Z = whitening * (X - mean)`.
    pub whitening: Vec<Vec<f64>>,
    /// `n_components x n_components` orthogonal unmixing: `S = unmixing * Z`.
    pub unmixing: Vec<Vec<f64>>,
    /// Whitening inverse factor `E * sqrt(D)` (`n_channels x n_components`),
    /// used for reconstruction.
    pub unwhitening: Vec<Vec<f64>>,
    /// Component time series, `n_components x n_samples`.
    pub sources: Vec<Vec<f64>>,
    pub n_components: usize,
}

/// Jacobi eigendecomposition of a symmetric matrix.
/// Returns `(eigenvalues desc, eigenvectors as columns)`.
/// Jacobi eigendecomposition of a symmetric matrix.
/// Returns `(eigenvalues descending, eigenvectors as columns)`.
pub fn jacobi_eigen(matrix: &[Vec<f64>]) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = matrix.len();
    let mut a: Vec<Vec<f64>> = matrix.to_vec();
    let mut v = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        v[i][i] = 1.0;
    }
    for _sweep in 0..50 {
        let mut max_off = 0.0f64;
        for p in 0..n {
            for q in p + 1..n {
                let apq = a[p][q];
                max_off = max_off.max(apq.abs());
                if apq.abs() < 1e-14 {
                    continue;
                }
                let app = a[p][p];
                let aqq = a[q][q];
                let theta = 0.5 * (aqq - app) / apq;
                // sign(0) must behave as +1: theta == 0 needs t = 1.
                let t = if theta >= 0.0 {
                    1.0 / (theta + (theta * theta + 1.0).sqrt())
                } else {
                    -1.0 / (-theta + (theta * theta + 1.0).sqrt())
                };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for k in 0..n {
                    let akp = a[k][p];
                    let akq = a[k][q];
                    a[k][p] = c * akp - s * akq;
                    a[k][q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let apk = a[p][k];
                    let aqk = a[q][k];
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
                for k in 0..n {
                    let vkp = v[k][p];
                    let vkq = v[k][q];
                    v[k][p] = c * vkp - s * vkq;
                    v[k][q] = s * vkp + c * vkq;
                }
            }
        }
        if max_off < 1e-12 {
            break;
        }
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| {
        a[j][j]
            .partial_cmp(&a[i][i])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let eigenvalues: Vec<f64> = order.iter().map(|&i| a[i][i]).collect();
    // eigenvectors as columns -> return row-major n x n with column j = vec j
    let mut vectors = vec![vec![0.0f64; n]; n];
    for (new_j, &old_j) in order.iter().enumerate() {
        for i in 0..n {
            vectors[i][new_j] = v[i][old_j];
        }
    }
    (eigenvalues, vectors)
}

/// Deterministic xorshift64 PRNG for the FastICA initial matrix.
struct Rng(u64);
impl Rng {
    fn next_f64(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        // Map to approximately standard normal via sum of uniforms.
        let mut sum = 0.0;
        for _ in 0..12 {
            sum += (x & 0xFFFF) as f64 / 65536.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
        }
        sum - 6.0
    }
}

/// Symmetric decorrelation: `W = (W W^T)^{-1/2} W`.
fn symmetric_decorrelate(w: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = w.len();
    // cov = W W^T
    let mut cov = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            cov[i][j] = w[i].iter().zip(w[j].iter()).map(|(a, b)| a * b).sum();
        }
    }
    let (evals, evecs) = jacobi_eigen(&cov);
    // (WW^T)^{-1/2} = E diag(1/sqrt(eval)) E^T
    let mut inv_sqrt = vec![vec![0.0f64; n]; n];
    for k in 0..n {
        let scale = if evals[k] > 1e-12 {
            1.0 / evals[k].sqrt()
        } else {
            0.0
        };
        for i in 0..n {
            for j in 0..n {
                inv_sqrt[i][j] += evecs[i][k] * scale * evecs[j][k];
            }
        }
    }
    // out = inv_sqrt * W
    let mut out = vec![vec![0.0f64; w[0].len()]; n];
    for i in 0..n {
        for j in 0..w[0].len() {
            for k in 0..n {
                out[i][j] += inv_sqrt[i][k] * w[k][j];
            }
        }
    }
    out
}

/// Run symmetric FastICA on a `(channels, samples)` matrix.
/// `n_components` defaults to the channel count (full-rank whitening).
pub fn fastica(
    matrix: &[Vec<f64>],
    n_components: Option<usize>,
    max_iter: usize,
    tol: f64,
) -> Result<IcaModel, CoreError> {
    let n_channels = matrix.len();
    let n_samples = matrix.first().map(Vec::len).unwrap_or(0);
    if n_channels == 0 || n_samples < 2 {
        return Err(CoreError::InvalidInput(
            "ICA needs channels and samples".into(),
        ));
    }
    let n_components = n_components.unwrap_or(n_channels).min(n_channels);
    if n_components == 0 {
        return Err(CoreError::InvalidInput(
            "n_components must be positive".into(),
        ));
    }
    let max_iter = max_iter.clamp(1, 2000);
    // 1. Center.
    let mut mean = vec![0.0f64; n_channels];
    for (ch, row) in matrix.iter().enumerate() {
        mean[ch] = row.iter().sum::<f64>() / n_samples as f64;
    }
    // 2. Covariance C = Xc Xc^T / n.
    let mut cov = vec![vec![0.0f64; n_channels]; n_channels];
    for i in 0..n_channels {
        for j in i..n_channels {
            let mut acc = 0.0;
            for t in 0..n_samples {
                acc += (matrix[i][t] - mean[i]) * (matrix[j][t] - mean[j]);
            }
            cov[i][j] = acc / n_samples as f64;
            cov[j][i] = cov[i][j];
        }
    }
    let (evals, evecs) = jacobi_eigen(&cov);
    // Whitening over top-n_components eigenpairs: Z_k = e_k^T Xc / sqrt(eval_k).
    let mut whitening = vec![vec![0.0f64; n_channels]; n_components];
    let mut unwhitening = vec![vec![0.0f64; n_components]; n_channels];
    let mut z = vec![vec![0.0f64; n_samples]; n_components];
    for k in 0..n_components {
        let scale = if evals[k] > 1e-12 {
            1.0 / evals[k].sqrt()
        } else {
            0.0
        };
        for ch in 0..n_channels {
            whitening[k][ch] = evecs[ch][k] * scale;
            unwhitening[ch][k] = evecs[ch][k] * evals[k].max(0.0).sqrt();
        }
        for t in 0..n_samples {
            let mut acc = 0.0;
            for ch in 0..n_channels {
                acc += evecs[ch][k] * (matrix[ch][t] - mean[ch]);
            }
            z[k][t] = acc * scale;
        }
    }
    // 3. Symmetric FastICA on whitened data.
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut w = vec![vec![0.0f64; n_components]; n_components];
    for i in 0..n_components {
        for j in 0..n_components {
            w[i][j] = rng.next_f64() * 0.1 + if i == j { 1.0 } else { 0.0 };
        }
    }
    let mut w = symmetric_decorrelate(&w);
    for _iter in 0..max_iter {
        // W_new = E[tanh(WZ)] Z^T / n - diag(E[1-tanh^2]) W
        let mut y = vec![vec![0.0f64; n_samples]; n_components];
        for i in 0..n_components {
            for t in 0..n_samples {
                let mut acc = 0.0;
                for k in 0..n_components {
                    acc += w[i][k] * z[k][t];
                }
                y[i][t] = acc.tanh();
            }
        }
        let mut w_new = vec![vec![0.0f64; n_components]; n_components];
        for i in 0..n_components {
            for j in 0..n_components {
                let mut acc = 0.0;
                for t in 0..n_samples {
                    acc += y[i][t] * z[j][t];
                }
                w_new[i][j] = acc / n_samples as f64;
            }
            let mut gp_mean = 0.0;
            for t in 0..n_samples {
                gp_mean += 1.0 - y[i][t] * y[i][t];
            }
            gp_mean /= n_samples as f64;
            for j in 0..n_components {
                w_new[i][j] -= gp_mean * w[i][j];
            }
        }
        let w_new = symmetric_decorrelate(&w_new);
        // Convergence: rows of W_new W_old^T ~ identity.
        let mut min_diag = f64::INFINITY;
        for i in 0..n_components {
            let mut dot = 0.0;
            for j in 0..n_components {
                dot += w_new[i][j] * w[i][j];
            }
            // |dot| close to 1 for converged row pairs (same ordering).
            min_diag = min_diag.min(dot.abs());
        }
        w = w_new;
        if min_diag > 1.0 - tol {
            break;
        }
    }
    // Sources S = W Z.
    let mut sources = vec![vec![0.0f64; n_samples]; n_components];
    for i in 0..n_components {
        for t in 0..n_samples {
            let mut acc = 0.0;
            for k in 0..n_components {
                acc += w[i][k] * z[k][t];
            }
            sources[i][t] = acc;
        }
    }
    Ok(IcaModel {
        mean,
        whitening,
        unmixing: w,
        unwhitening,
        sources,
        n_components,
    })
}

/// Reconstruct the channel-space signal, optionally dropping components.
/// `excluded` lists zero-based component indices to reject.
pub fn reconstruct(model: &IcaModel, excluded: &[usize]) -> Vec<Vec<f64>> {
    let n_channels = model.mean.len();
    let n_samples = model.sources.first().map(Vec::len).unwrap_or(0);
    // Z_hat = W^T S_keep (orthogonal unmixing inverse).
    let n_components = model.n_components;
    let mut z_hat = vec![vec![0.0f64; n_samples]; n_components];
    for j in 0..n_components {
        if excluded.contains(&j) {
            continue;
        }
        for t in 0..n_samples {
            z_hat[j][t] = model.sources[j][t];
        }
    }
    // Xc_hat = unwhitening * (W^T applied): Z_hat already in whitened space;
    // rotate back through unmixing transpose first.
    let mut z_back = vec![vec![0.0f64; n_samples]; n_components];
    for i in 0..n_components {
        for t in 0..n_samples {
            let mut acc = 0.0;
            for j in 0..n_components {
                acc += model.unmixing[j][i] * z_hat[j][t];
            }
            z_back[i][t] = acc;
        }
    }
    let mut out = vec![vec![0.0f64; n_samples]; n_channels];
    for ch in 0..n_channels {
        for t in 0..n_samples {
            let mut acc = 0.0;
            for k in 0..n_components {
                acc += model.unwhitening[ch][k] * z_back[k][t];
            }
            out[ch][t] = acc + model.mean[ch];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jacobi_recovers_known_eigenpairs() {
        // Dense symmetric matrix A = Q D Q^T with Q a rotation product.
        let d = [9.0f64, 4.0, 1.0];
        let c = 0.6f64;
        let s = 0.8f64;
        let q = [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]];
        let mut matrix = vec![vec![0.0f64; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    matrix[i][j] += q[i][k] * d[k] * q[j][k];
                }
            }
        }
        let (evals, evecs) = jacobi_eigen(&matrix);
        assert!((evals[0] - 9.0).abs() < 1e-10);
        assert!((evals[1] - 4.0).abs() < 1e-10);
        assert!((evals[2] - 1.0).abs() < 1e-10);
        // eigenvector for eval 1 is e3 = [0,0,1] -> column 2
        assert!(evecs[2][2].abs() > 0.999);
        // Reconstruction A v = lambda v for each pair.
        for k in 0..3 {
            for i in 0..3 {
                let mut acc = 0.0;
                for j in 0..3 {
                    acc += matrix[i][j] * evecs[j][k];
                }
                assert!((acc - evals[k] * evecs[i][k]).abs() < 1e-8);
            }
        }
    }

    #[test]
    fn fastica_whitening_roundtrip_is_exact() {
        // Rank-2 data in 3 channels: x = a*s1 + b*s2.
        let n = 500usize;
        let s1: Vec<f64> = (0..n).map(|i| (i as f64 * 0.05).sin()).collect();
        let s2: Vec<f64> = (0..n).map(|i| (i as f64 * 0.11).cos()).collect();
        let matrix = vec![
            s1.iter().zip(&s2).map(|(a, b)| a + 0.5 * b + 2.0).collect(),
            s1.iter().zip(&s2).map(|(a, b)| -a + b - 1.0).collect(),
            s1.iter().map(|v| 0.3 * v).collect(),
        ];
        let model = fastica(&matrix, Some(2), 100, 1e-6).unwrap();
        // Whitening roundtrip: unwhitening @ whitening @ Xc == Xc (rank 2).
        let t = 11usize;
        let mut z = vec![0.0; 2];
        for k in 0..2 {
            for ch in 0..3 {
                z[k] += model.whitening[k][ch] * (matrix[ch][t] - model.mean[ch]);
            }
        }
        for ch in 0..3 {
            let mut xh = model.mean[ch];
            for k in 0..2 {
                xh += model.unwhitening[ch][k] * z[k];
            }
            assert!(
                (xh - matrix[ch][t]).abs() < 1e-8,
                "ch{ch}: {xh} vs {}",
                matrix[ch][t]
            );
        }
    }
}

/// Variance summary for one ICA component (for artifact review).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IcaComponentInfo {
    pub index: usize,
    pub variance: f64,
    /// Fraction of total component variance (sums to ~1 across components).
    pub variance_fraction: f64,
}

/// Topography for one ICA component: per-channel mixing weights plus an
/// optional interpolated scalp grid.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IcaTopography {
    pub component: usize,
    /// Unit-norm mixing weights, one per input channel.
    pub channel_weights: Vec<f64>,
    /// Flattened azimuth×elevation scalp grid (row-major), present only when
    /// electrode positions were supplied.
    #[serde(default)]
    pub grid: Option<Vec<f64>>,
    #[serde(default)]
    pub grid_cols: usize,
    #[serde(default)]
    pub grid_rows: usize,
}
