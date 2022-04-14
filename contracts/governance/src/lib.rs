pub mod entrypoints;

mod error;
mod executions;
mod queries;
mod staking;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;
mod validators;
