pub mod no_args;
pub mod one_immediate;

// Re-export dispatcher functions for easy access
pub use no_args::dispatch_no_args;
pub use one_immediate::dispatch_one_immediate;
