pub mod error;
pub mod log;

pub fn greet(who: &str) -> String {
    format!("Ahoy, {who}!")
}
