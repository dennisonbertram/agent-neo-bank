use std::sync::Arc;

use crate::core::services::CoreServices;

pub struct AppState {
    pub core: Arc<CoreServices>,
}
