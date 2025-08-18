use rmcp::serve_server;
use std::error::Error;
mod json_validator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let validator_service = json_validator::JsonValidator::new();

    println!("Starting JSON Validator MCP server, connect to standard input/output");

    let io = (tokio::io::stdin(), tokio::io::stdout());

    serve_server(validator_service, io).await?;
    Ok(())
}
