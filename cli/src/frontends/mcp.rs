pub struct McpFrontend {
}

impl McpFrontend {
    pub fn new() -> Self {
        Self {
        }
    }

    pub async fn handle_request(
        &self,
        _request: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        todo!("MCP frontend not yet implemented")
    }
}

impl Default for McpFrontend {
    fn default() -> Self {
        Self::new()
    }
}
