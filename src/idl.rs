use anchor_client::idl::Idl;
use std::{fs::File, path::Path};

pub fn load_idl() -> anyhow::Result<Idl> {
    let path = Path::new("../assignment/target/idl/collateral_vault.json");
    let file = File::open(path)?;
    let idl: Idl = serde_json::from_reader(file)?;
    Ok(idl)
}
