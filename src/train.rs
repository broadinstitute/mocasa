use crate::data::load_training_data;
use crate::error::Error;
use crate::options::config::Config;

pub(crate) fn train(config: &Config) -> Result<(), Error> {
    let data = load_training_data(config)?;
    println!("Loaded data for {} variants", data.beta_se_lists.len());
    println!("Traits are {}", data.names.join(", "));
    for (var_id, beta_se_list) in data.beta_se_lists {
        print!("{}:", var_id);
        for (name, beta_se) in data.names.iter().zip(beta_se_list.iter()) {
            print!(" {}: beta={}, se={}", name, beta_se.beta, beta_se.se)
        }
        println!()
    }
    Ok(())
}