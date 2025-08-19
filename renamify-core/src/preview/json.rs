use crate::scanner::Plan;

/// Render plan as JSON
pub fn render_json(plan: &Plan) -> String {
    serde_json::to_string_pretty(plan).unwrap_or_else(|_| "null".to_string())
}
