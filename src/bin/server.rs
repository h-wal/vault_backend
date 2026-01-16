use vault_backend::api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    api::run_server().await
}

