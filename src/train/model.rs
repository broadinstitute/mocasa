use crate::data::TrainData;
use crate::math::matrix::Matrix;
use crate::train::param_stats::ParamHessian;
use crate::train::params::Params;
use crate::train::vars::Vars;

pub(crate) struct TrainModel {
    data: TrainData,
}

impl TrainModel {
    pub(crate) fn new(data: TrainData) -> TrainModel {
        TrainModel { data }
    }
    pub(crate) fn initial_params(&self) -> Params {
        let meta = self.data.meta.clone();
        let mu = 0.0;
        let tau = 1.0;
        let betas = vec![1.0; self.data.n_traits()];
        let sigmas = vec![1.0; self.data.n_traits()];
        Params { meta, mu, tau, betas, sigmas }
    }
    pub(crate) fn initial_vars(&self, params: &Params) -> Vars {
        let meta = self.data.meta.clone();
        let es = vec![params.mu; self.data.n_data_points()];
        let element_gen = |i_data_point: usize, i_trait: usize| {
            es[i_data_point] * params.betas[i_trait]
        };
        let ts =
            Matrix::fill(self.data.n_data_points(), self.data.n_traits(),
                         element_gen);
        Vars { meta, es, ts }
    }
    pub(crate) fn f_quot_e<'a>(&self, params: &'a Params, vars: &'a Vars, i_data_point: &'a usize)
                               -> impl Fn(f64, f64) -> f64 + 'a {
        |e_new: f64, e_old: f64| {
            let mu = params.mu;
            let tau = params.tau;
            let e_term =
                ((e_new - mu).powi(2) - (e_old - mu).powi(2)) / tau.powi(2);
            let ts = &vars.ts[*i_data_point];
            let t_sum: f64 =
                (0..params.meta.n_traits()).map(|i_trait: usize| {
                    ((ts[i_trait] - params.betas[i_trait] * e_new).powi(2) -
                        (ts[i_trait] - params.betas[i_trait] * e_old).powi(2)) /
                        params.sigmas[i_trait].powi(2)
                }).sum();
            (-0.5 * (e_term + t_sum)).exp()
        }
    }
    pub(crate) fn f_quot_t<'a>(&'a self, params: &'a Params, vars: &'a Vars,
                               i_data_point: &'a usize, i_trait: &'a usize)
                               -> impl Fn(f64, f64) -> f64 + 'a{
        |t_new: f64, t_old: f64| {
            let e = vars.es[*i_data_point];
            let beta = params.betas[*i_trait];
            let sigma = params.sigmas[*i_trait];
            let t_term =
                ((t_new - beta * e).powi(2) - (t_old - beta * e).powi(2)) / sigma.powi(2);
            let o = self.data.betas[*i_data_point][*i_trait];
            let s = self.data.ses[*i_data_point][*i_trait];
            let o_term = ((t_new - o).powi(2) - (t_old - o).powi(2)) / s.powi(2);
            (-0.5 * (t_term + o_term)).exp()
        }
    }
    pub(crate) fn evaluate_params(&self, params: &Params, vars: &Vars) -> ParamHessian {
        todo!()
    }
}

