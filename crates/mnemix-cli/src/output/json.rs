use crate::output::CommandOutput;

pub(crate) fn render_json(output: &CommandOutput) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(output).map(|json| format!("{json}\n"))
}
