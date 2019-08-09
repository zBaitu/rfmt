fn main() {
    let dnlml = Array1::from_shape_fn(ln_hyperparams.len(), |i| {
        0.5 * (self
            .covfunc
            .deriv_covariance(ln_hyperparams, train_inputs, i)
            * &W)
            .scalar_sum()
    });
}
