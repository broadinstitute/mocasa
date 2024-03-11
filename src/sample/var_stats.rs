use crate::data::Meta;
use crate::math::matrix::Matrix;
use crate::params::Params;
use crate::sample::vars::Vars;

pub(crate) struct VarStats {
    meta: Meta,
    n: usize,
    e_sums: Vec<f64>,
    e2_sums: Vec<f64>,
    e_t_sums: Matrix,
    t2_sums: Matrix,
}

pub(crate) struct MuSig {
    pub(crate) mu: f64,
    pub(crate) sig: f64
}

impl VarStats {
    pub(crate) fn new(meta: Meta) -> VarStats {
        let n: usize = 0;
        let n_data_points = meta.n_data_points();
        let n_traits = meta.n_traits();
        let e_sums: Vec<f64> = vec![0.0; n_data_points];
        let e2_sums: Vec<f64> = vec![0.0; n_data_points];
        let e_t_sums: Matrix = Matrix::fill(n_data_points, n_traits, |_, _| 0.0);
        let t2_sums: Matrix = Matrix::fill(n_data_points, n_traits, |_, _| 0.0);
        VarStats { meta, n, e_sums, e2_sums, e_t_sums, t2_sums }
    }
    pub(crate) fn add(&mut self, vars: &Vars) {
        self.n += 1;
        let n_data_points = self.meta.n_data_points();
        let n_traits = self.meta.n_traits();
        for j in 0..n_data_points {
            let e_j = vars.es[j];
            self.e_sums[j] += e_j;
            self.e2_sums[j] += e_j.powi(2);
            for i in 0..n_traits {
                let t_j_i = vars.ts[j][i];
                self.e_t_sums[j][i] += e_j * t_j_i;
                self.t2_sums[j][i] += t_j_i.powi(2);
            }
        }
    }
    pub(crate) fn compute_new_params(&self) -> Params {
        let meta = &self.meta;
        let n_f = self.n as f64;
        let n_data_points = meta.n_data_points();
        let n_data_points_f = n_data_points as f64;
        let n_traits = meta.n_traits();
        let mut sum_for_mu: f64 = 0.0;
        for j in 0..n_data_points {
            let mean_e_j = self.e_sums[j] / n_f;
            sum_for_mu += mean_e_j;
        }
        let mu = sum_for_mu / n_data_points_f;
        let mut sum_for_tau: f64 = 0.0;
        for j in 0..n_data_points {
            let mean_e2_j = self.e2_sums[j] / n_f;
            let mean_e_j = self.e_sums[j] / n_f;
            sum_for_tau += mean_e2_j - 2.0 * mu * mean_e_j + n_data_points_f * mu.powi(2);
        }
        let tau = (sum_for_tau / n_data_points_f).sqrt();
        let mut betas: Vec<f64> = Vec::with_capacity(n_traits);
        for i in 0..n_traits {
            let mut mean_e_t_sum: f64 = 0.0;
            let mut mean_e2_sum: f64 = 0.0;
            for j in 0..n_data_points {
                mean_e_t_sum += self.e_t_sums[j][i] / n_f;
                mean_e2_sum += self.e2_sums[j] / n_f;
            }
            betas.push(mean_e_t_sum / mean_e2_sum);
        }
        let mut sigmas: Vec<f64> = Vec::with_capacity(n_traits);
        for (i, beta) in betas.iter().enumerate() {
            let mut sum_for_sigma: f64 = 0.0;
            for j in 0..n_data_points {
                let mean_t2_j_i = self.t2_sums[j][i] / n_f;
                let mean_e_t_j_i = self.e_t_sums[j][i] / n_f;
                let mean_e2_j_i = self.e2_sums[j] / n_f;
                sum_for_sigma +=
                    mean_t2_j_i - 2.0 * betas[i] * mean_e_t_j_i + beta.powi(2) * mean_e2_j_i
            }
            let sigma = (sum_for_sigma / n_data_points_f).sqrt();
            sigmas.push(sigma)
        }
        let trait_names = meta.trait_names.clone();
        Params { trait_names, mu, tau, betas, sigmas }
    }
    pub(crate) fn calculate_mu_sig(&self) -> MuSig {
        let meta = &self.meta;
        let n_f = self.n as f64;
        let n_data_points = meta.n_data_points();
        let n_data_points_f = n_data_points as f64;
        let mut sum_for_mu: f64 = 0.0;
        let mut sum_for_mu_mu: f64 = 0.0;
        for j in 0..n_data_points {
            let mean_e_j = self.e_sums[j] / n_f;
            sum_for_mu += mean_e_j;
            let mean_e2_j = self.e2_sums[j] / n_f;
            sum_for_mu_mu += mean_e2_j;
        }
        let mu = sum_for_mu / n_data_points_f;
        let mu_mu = sum_for_mu_mu / n_data_points_f;
        let sig = (mu_mu - mu*mu).sqrt();
        MuSig { mu, sig }
    }
}