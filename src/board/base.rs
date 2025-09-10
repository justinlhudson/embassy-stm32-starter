// Base board configuration module - defines the common interface for all board implementations

/// Stub trait for board configuration
pub trait BoardConfiguration {
	fn board_name() -> &'static str;
}

/// Stub trait for interrupt handlers
pub trait InterruptHandlers {
	fn setup();
}
