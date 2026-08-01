use crate::app::AppContext;

/// Gate for the MCP writer tools. Writes must be explicitly enabled by a human in
/// the MyServiceBus UI; the window lasts 10 minutes (or until it is switched off).
/// Returns a tool-friendly error string while writes are disabled.
pub fn ensure_mcp_writes_enabled(app: &AppContext) -> Result<(), String> {
    if app.is_mcp_write_enabled() {
        return Ok(());
    }

    Err(
        "MCP write operations are currently DISABLED. Ask the user to open the \
         MyServiceBus UI and click \"Enable\" next to MCP W in the status bar (writes \
         stay on for 10 minutes, or until the user switches them off). Do not retry \
         until enabled."
            .into(),
    )
}
