// ---------------------------------------------------------------------------
// App extension for registering Saveable resources
// ---------------------------------------------------------------------------

use bevy::prelude::*;
use simulation::{Saveable, SaveableRegistry};

/// Extension trait on `App` for one-line saveable registration.
///
/// # Example
///
/// ```ignore
/// use save::SaveableAppExt;
///
/// fn build(&self, app: &mut App) {
///     app.init_resource::<MyFeatureState>()
///        .register_saveable::<MyFeatureState>();
/// }
/// ```
pub trait SaveableAppExt {
    fn register_saveable<T: Saveable>(&mut self) -> &mut Self;
}

impl SaveableAppExt for App {
    fn register_saveable<T: Saveable>(&mut self) -> &mut Self {
        // Ensure the registry exists (idempotent).
        self.init_resource::<SaveableRegistry>();
        // Register the type.
        self.world_mut()
            .resource_mut::<SaveableRegistry>()
            .register::<T>();
        self
    }
}
