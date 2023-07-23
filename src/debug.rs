use bevy::{prelude::*, reflect::TypePath};
use bevy_rapier2d::render::DebugRenderContext;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, TypePath)]
pub enum DebugAction {
    Click,
}

#[derive(Resource, PartialEq)]
pub enum DebugLevel {
    None,
    Basic,
}

pub fn switch_debug(
    action: Query<&ActionState<DebugAction>>,
    mut debug_level: ResMut<DebugLevel>,
    mut rapier_debug: ResMut<DebugRenderContext>
) {
    for action in &action {
        if action.just_pressed(DebugAction::Click) {
            *debug_level = match *debug_level {
                DebugLevel::None => DebugLevel::Basic,
                DebugLevel::Basic => DebugLevel::None,
            };
            rapier_debug.enabled = *debug_level == DebugLevel::Basic;
        }
    }
}

#[derive(Bundle)]
pub struct DebugBundle {
    pub name: Name,
    pub action: InputManagerBundle<DebugAction>,
}

pub fn debug_setup() -> DebugBundle {
    DebugBundle {
        name: Name::new("DebugManager"),
        action: InputManagerBundle::<DebugAction> {
            action_state: ActionState::<DebugAction>::default(),
            input_map: InputMap::new([(KeyCode::V, DebugAction::Click)]),
        },
    }
}
