#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

pub fn start() -> Result<(), Box<dyn std::error::Error>> {
    trace!("Starting");
    Ok(())
}
