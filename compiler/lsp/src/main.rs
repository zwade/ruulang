use server::RuuLangServer;
use tower_lsp::{LspService, Server};

pub mod server;
mod utils;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| RuuLangServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
