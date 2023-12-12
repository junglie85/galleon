pub mod error;
pub mod logger;

pub fn greet(who: &str) -> String {
    format!("Ahoy, {who}!")
}
