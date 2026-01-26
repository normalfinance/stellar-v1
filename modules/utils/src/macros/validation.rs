use soroban_sdk::panic_with_error;

// A macro that validates a condition and panics with a specific error if the condition is false
#[macro_export]
macro_rules! validate {
    ($env:expr, $condition:expr, $error:expr) => {
        if !$condition {
            panic_with_error!($env, $error) // Panic with the specified error
        }
    };
}
