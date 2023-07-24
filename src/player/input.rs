use bevy::{prelude::*, reflect::TypePath};
use leafwing_input_manager::{prelude::*, Actionlike};

use crate::{
    animations::{AnimationFlip, AnimationState},
    debug::DebugLevel,
    mouse::Mouse,
    rendering::{Angle, Position},
};

use super::{
    stats::PlayerStats,
    weapon::{GunEntity, GunStats},
};

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, TypePath)]
pub enum PlayerActions {
    ControllerMove,
    ControllerLook,
    ControllerShoot,
    Left,
    Right,
    Up,
    Down,
    Shoot,
}

#[derive(Component, Debug, Reflect, Default)]
pub enum PlayerState {
    #[default]
    Idle,
    LeftFront,
    LeftBack,
    RightFront,
    RightBack,
    Front,
    Back,
}

pub fn update_gun_angle(
    debug_level: DebugLevel,
    lines: &mut bevy_prototype_debug_lines::DebugLines,
    gun_pos: Vec2,
    cursor_position: Vec2,
    gun_stats: &GunStats,
    gun_angle: &mut Angle,
    flip: &mut AnimationFlip
) {
    let direction = (cursor_position - gun_pos).normalize();
    let mut barrel_position = gun_pos + direction.perp() * gun_stats.barrel_height;
    let mut barrel_to_cursor = cursor_position - barrel_position;
    *gun_angle = Angle(barrel_to_cursor.y.atan2(barrel_to_cursor.x));
    *flip = if gun_angle.0.abs().to_degrees() > 90. {
        barrel_position = gun_pos + direction.perp() * -gun_stats.barrel_height;
        barrel_to_cursor = cursor_position - barrel_position;
        *gun_angle = Angle(barrel_to_cursor.y.atan2(barrel_to_cursor.x));
        AnimationFlip::YAxis
    } else {
        AnimationFlip::False
    };
    if debug_level == DebugLevel::Basic {
        lines.line_colored((gun_pos).extend(0.), barrel_position.extend(0.), 0.0, Color::RED);
        lines.line_colored(barrel_position.extend(0.), cursor_position.extend(0.), 0.0, Color::GREEN);
        lines.line_colored(cursor_position.extend(0.), (gun_pos).extend(0.), 0.0, Color::GOLD);
    }
}

pub fn calculate_cursor_position(
    stats: &PlayerStats,
    player_actions: &ActionState<PlayerActions>,
    camera_transform: &GlobalTransform,
    camera: &Camera,
    player_pos: &Vec2,
    mouse: Option<&ActionState<Mouse>>,
) -> Option<Vec2> {
    if stats.controller {
        if player_actions.pressed(PlayerActions::ControllerLook) {
            let axis_pair = player_actions.clamped_axis_pair(PlayerActions::ControllerLook).unwrap();
            return Some(*player_pos + axis_pair.xy().normalize() * 30.);
        }
    } else {
        if let Some(mouse_action_state) = mouse {
            let mouse_ray = mouse_action_state
            .axis_pair(Mouse::MousePosition)
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor.xy()));

            if let Some(mouse_pos) = mouse_ray {
                return Some(mouse_pos.origin.truncate());
            }
        }
    }
    None
}

#[allow(clippy::type_complexity)]
pub fn shooting_system(
    time: Res<Time>,
    mouse: Query<&ActionState<Mouse>>,
    mut players: Query<(&Position, &GunEntity, &ActionState<PlayerActions>, &mut PlayerStats)>,
    mut gun: Query<(
        &mut Position,
        &mut Angle,
        &mut AnimationFlip,
        &mut GunStats,
        Without<PlayerStats>,
    )>,
    camera: Query<(&Camera, &GlobalTransform)>,
    debug_level: Res<DebugLevel>,
    mut lines: ResMut<bevy_prototype_debug_lines::DebugLines>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if let Some((camera, camera_transform)) =
        camera.into_iter().find(|(camera, _)| camera.is_active)
    {
        for (Position(player_pos), gun_id, player_actions, mut stats) in &mut players {
            let mouse_maybe = mouse.get_single();
            let cursor_position: Option<Vec2> = calculate_cursor_position(&stats, player_actions, camera_transform, camera, player_pos, mouse_maybe.ok(),);

            if let Ok((mut gun_pos, mut gun_angle, mut flip, mut gun_stats, _)) = gun.get_mut(gun_id.0) {
                gun_pos.0 = *player_pos;
                gun_pos.0.x += 6.;

                if let Some(cursor_position) = cursor_position {
                    update_gun_angle((*debug_level).clone(), &mut lines, gun_pos.0, cursor_position, &gun_stats, &mut gun_angle, &mut flip)
                }
                let angle = (*gun_angle).0;
                let direction = Vec2::from_angle(angle).normalize();
                let barrel_position: Vec2;
                if *flip == AnimationFlip::False {
                    barrel_position = gun_pos.0 + direction.perp() * gun_stats.barrel_height;
                } else {
                    barrel_position = gun_pos.0 + direction.perp() * -gun_stats.barrel_height;
                }
                let barrel_end = barrel_position + Vec2::from_angle(angle) * gun_stats.barrel_length;
                if *debug_level == DebugLevel::Basic && cursor_position.is_none() {
                    lines.line_colored((gun_pos.0).extend(0.), (barrel_position).extend(0.), 0.0, Color::AZURE);
                    lines.line_colored((gun_pos.0).extend(0.), (gun_pos.0 + Vec2::from_angle(angle) * gun_stats.barrel_length).extend(0.), 0.0, Color::CYAN);
                    lines.line_colored((barrel_position).extend(0.), (barrel_end).extend(0.), 0.0, Color::PURPLE);
                }

                gun_stats.timer.tick(time.delta());
                if (player_actions.pressed(PlayerActions::Shoot) && !stats.controller) || (player_actions.pressed(PlayerActions::ControllerShoot) && stats.controller) {
                    (gun_stats.shoot)(&mut commands, &asset_server, &mut gun_stats, &mut stats, barrel_end, angle);
                }
            }
        }
    }
}

pub fn player_input_setup() -> InputManagerBundle::<PlayerActions> {
    let mut input_map = InputMap::new([
        (KeyCode::Q, PlayerActions::Left),
        (KeyCode::D, PlayerActions::Right),
        (KeyCode::Z, PlayerActions::Up),
        (KeyCode::S, PlayerActions::Down),
    ]);
    input_map.insert(DualAxis::left_stick(), PlayerActions::ControllerMove)
        .insert(DualAxis::right_stick(), PlayerActions::ControllerLook)
        .insert(MouseButton::Left, PlayerActions::Shoot)
        .insert(GamepadButtonType::South, PlayerActions::ControllerShoot);

    InputManagerBundle::<PlayerActions> {
        action_state: ActionState::default(),
        input_map
    }
}

pub fn move_players(
    time: Res<Time>,
    mut query: Query<(
        &PlayerStats,
        &ActionState<PlayerActions>,
        &mut Position,
        &mut AnimationState,
    )>,
) {
    for (stats, actions, mut position, mut state) in &mut query {
        let mut direction = Vec2::ZERO;

        if stats.controller {
            if actions.pressed(PlayerActions::ControllerMove) {
                let axis_pair = actions.clamped_axis_pair(PlayerActions::ControllerMove).unwrap();
                direction.x += axis_pair.x();
                direction.y += axis_pair.y();
            }
        } else {
            if actions.pressed(PlayerActions::Left) {
                direction.x -= 1.;
            }
            if actions.pressed(PlayerActions::Right) {
                direction.x += 1.;
            }
            if actions.pressed(PlayerActions::Up) {
                direction.y += 1.;
            }
            if actions.pressed(PlayerActions::Down) {
                direction.y -= 1.;
            }
        }

        if direction == Vec2::ZERO {
            *state = AnimationState::new(&PlayerState::Idle);
        } else {
            let mut angle = direction.angle_between(Vec2::NEG_Y).to_degrees();
            if angle < 0. {
                angle += 360.
            }
            *state = match angle {
                n if (n < 30. + 60. * 0.) => AnimationState::new(&PlayerState::Front),
                n if (n <= 30. + 60. * 1.) => AnimationState::new(&PlayerState::LeftFront),
                n if (n < 30. + 60. * 2.) => AnimationState::new(&PlayerState::LeftBack),
                n if (n < 30. + 60. * 3.) => AnimationState::new(&PlayerState::Back),
                n if (n < 30. + 60. * 4.) => AnimationState::new(&PlayerState::RightBack),
                n if (n < 30. + 60. * 5.) => AnimationState::new(&PlayerState::RightFront),
                n if (n < 30. + 60. * 6.) => AnimationState::new(&PlayerState::Front),
                _ => {
                    panic!("IMPOSSIBLE ANGLE!")
                }
            };
            position.0 += direction.clamp_length(0., 1.) * stats.speed * time.delta_seconds();
        }
    }
}