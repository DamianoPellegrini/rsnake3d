use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_fixed_timer, window::PrimaryWindow};
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    Forward,
    Backward,
}

impl From<IVec3> for Direction {
    fn from(value: IVec3) -> Self {
        match value {
            IVec3::Y => Direction::Up,
            IVec3::NEG_Y => Direction::Down,
            IVec3::X => Direction::Right,
            IVec3::NEG_X => Direction::Left,
            IVec3::Z => Direction::Forward,
            IVec3::NEG_Z => Direction::Backward,
            _ => panic!("Invalid direction"),
        }
    }
}

impl From<Direction> for IVec3 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => IVec3::Y,
            Direction::Down => IVec3::NEG_Y,
            Direction::Right => IVec3::X,
            Direction::Left => IVec3::NEG_X,
            Direction::Forward => IVec3::Z,
            Direction::Backward => IVec3::NEG_Z,
        }
    }
}

/// Stores the assets for the game
#[derive(Resource)]
struct SnakeAssets {
    snake_material: Handle<StandardMaterial>,
    food_material: Handle<StandardMaterial>,

    head_mesh: Handle<Mesh>,
    tail_mesh: Handle<Mesh>,
    // tail_angle_mesh: Handle<Mesh>,
    food_mesh: Handle<Mesh>,
}

/// Stores the position in a grid like fashion
#[derive(PartialEq, Component, Debug, Default, Clone, Copy, Reflect, FromReflect)]
struct Position(IVec3);

/// Tag for food
#[derive(Component, Debug, Default)]
struct Food;

/// Stores the direction the snake is moving in
#[derive(Component, Debug, Reflect)]
struct SnakeHead(Direction);

impl Default for SnakeHead {
    fn default() -> Self {
        SnakeHead(Direction::Up)
    }
}

/// Tag for snake segments
#[derive(Component, Debug, Default)]
struct SnakeSegment;

/// Stores the position of the last snake segment before it moved
#[derive(Component, Debug, Default, Reflect)]
struct LastSnakeSegment(Option<Position>);

#[derive(Bundle)]
struct SnakeSegmentBundle {
    _segment: SnakeSegment,
    _name: Name,
    position: Position,
    #[bundle]
    pbr: PbrBundle,
}

impl Default for SnakeSegmentBundle {
    fn default() -> Self {
        Self {
            _name: Name::new("Snake Segment"),
            _segment: SnakeSegment::default(),
            position: Position::default(),
            pbr: PbrBundle::default(),
        }
    }
}

#[derive(Bundle)]
struct SnakeHeadBundle {
    head: SnakeHead,
    #[bundle]
    segment: SnakeSegmentBundle,
}

impl Default for SnakeHeadBundle {
    fn default() -> Self {
        Self {
            head: SnakeHead::default(),
            segment: SnakeSegmentBundle {
                _name: Name::new("Snake Head"),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
struct SnakeLastSegmentBundle {
    last: LastSnakeSegment,
    #[bundle]
    segment: SnakeSegmentBundle,
}

impl Default for SnakeLastSegmentBundle {
    fn default() -> Self {
        Self {
            last: LastSnakeSegment::default(),
            segment: SnakeSegmentBundle::default(),
        }
    }
}

#[derive(Bundle)]
struct FoodBundle {
    _name: Name,
    _food: Food,
    position: Position,
    #[bundle]
    pbr: PbrBundle,
}

impl Default for FoodBundle {
    fn default() -> Self {
        Self {
            _name: Name::new("Food"),
            _food: Food::default(),
            position: Position::default(),
            pbr: PbrBundle::default(),
        }
    }
}

/// Notify that the food has been eaten
struct EatEvent;

fn load_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // mut asset_server: ResMut<AssetServer>,
) {
    let head_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.8 }));
    let tail_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.65 }));

    let food_mesh = meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 0.4,
            subdivisions: 2,
        })
        .unwrap(),
    );

    commands.insert_resource(SnakeAssets {
        snake_material: materials.add(StandardMaterial {
            base_color: Color::rgb(0., 0.7, 0.),
            // unlit: true,
            ..default()
        }),
        food_material: materials.add(StandardMaterial {
            base_color: Color::rgb(1., 0., 0.),
            // unlit: true,
            ..default()
        }),

        head_mesh,
        tail_mesh,
        // tail_angle_mesh: head_mesh,
        food_mesh,
    });
}

fn setup_window(mut primary_window_q: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = primary_window_q.get_single_mut() else {
        return;
    };
    window.title = "Snake DDD".to_string();
    // window.resolution = (500.0, 500.0).into();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_scene(
    mut commands: Commands,
    snake_assets: Res<SnakeAssets>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    // HEAD
    commands.spawn(SnakeHeadBundle {
        head: SnakeHead(Direction::Up),
        segment: SnakeSegmentBundle {
            position: Position(IVec3 { x: 0, y: 0, z: 0 }),
            pbr: PbrBundle {
                mesh: meshes.get_handle(&snake_assets.head_mesh),
                material: materials.get_handle(&snake_assets.snake_material),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        },
        ..default()
    });

    debug!(target: "bevypoco::setup_scene", "Spawned head");

    // Starting tail
    commands.spawn(SnakeLastSegmentBundle {
        segment: SnakeSegmentBundle {
            position: Position(IVec3 { x: 0, y: -1, z: 0 }),
            pbr: PbrBundle {
                mesh: meshes.get_handle(&snake_assets.tail_mesh),
                material: materials.get_handle(&snake_assets.snake_material),
                transform: Transform::from_xyz(0., -1., 0.),
                ..default()
            },
            ..default()
        },
        ..default()
    });

    debug!(target: "bevypoco::setup_scene", "Spawned tail");

    commands.spawn(FoodBundle {
        position: Position(IVec3 { x: 0, y: 1, z: 0 }),
        pbr: PbrBundle {
            mesh: meshes.get_handle(&snake_assets.food_mesh),
            material: materials.get_handle(&snake_assets.food_material),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
        ..default()
    });

    debug!(target: "bevypoco::setup_scene", "Spawned food");
}

fn position_translation(mut query: Query<(&Position, &mut Transform)>) {
    for (Position(pos), mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
    }
}

fn eat_food(
    mut commands: Commands,
    mut eat_writer: EventWriter<EatEvent>,
    food_position: Query<(Entity, &Position), With<Food>>,
    head_position: Query<&Position, With<SnakeHead>>,
) {
    let Ok(head_pos) = head_position.get_single() else {
        return;
    };

    let Ok((ent, food_pos)) = food_position.get_single() else {
        return;
    };

    if food_pos == head_pos {
        debug!(target: "bevypoco::eat_food", head = ?head_pos, food = ?food_pos);
        commands.entity(ent).despawn();
        eat_writer.send(EatEvent);
        debug!(target: "bevypoco::events", "Sent EatEvent");
    }
}

fn snake_growth(
    mut commands: Commands,
    mut eat_reader: EventReader<EatEvent>,
    last_segment: Query<(Entity, &LastSnakeSegment)>,
    snake_assets: Res<SnakeAssets>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((last_segment_ent, LastSnakeSegment(Some(last_segment_pos)))) = last_segment.get_single() else {
        return;
    };

    if eat_reader.iter().next().is_none() {
        return;
    }

    debug!(target: "bevypoco::snake_growth", "Received EatEvent");
    debug!(target: "bevypoco::snake_growth", ?last_segment_ent, ?last_segment_pos);
    // add new segment after last and move last component
    // to the new one
    commands
        .entity(last_segment_ent)
        .remove::<LastSnakeSegment>();

    debug!(target: "bevypoco::snake_growth", "Removed LastSnakeSegment from {:?}", last_segment_ent);

    commands.spawn((
        SnakeSegmentBundle {
            position: *last_segment_pos,
            pbr: PbrBundle {
                mesh: meshes.get_handle(&snake_assets.tail_mesh),
                material: materials.get_handle(&snake_assets.snake_material),
                transform: Transform::from_xyz(0., -1., 0.),
                ..default()
            },
            ..default()
        },
        LastSnakeSegment(None),
    ));

    debug!(target: "bevypoco::snake_growth", "Spawned new tail segment at {:?}", last_segment_pos);
}

fn food_spawner(
    mut commands: Commands,
    mut eat_reader: EventReader<EatEvent>,
    snake: Query<&Position, With<SnakeSegment>>,
    snake_assets: Res<SnakeAssets>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    if eat_reader.iter().next().is_none() {
        return;
    }

    let pos = loop {
        let x = 0;
        let z = 0;

        // let x = rand::thread_rng().gen_range(-5..=5);
        let y = rand::thread_rng().gen_range(-5..=5);
        // let z = rand::thread_rng().gen_range(-5..=5);

        let pos = Position(IVec3 { x, y, z });
        if snake.iter().all(|p| *p != pos) {
            break pos;
        }
    };

    commands.spawn(FoodBundle {
        position: pos,
        pbr: PbrBundle {
            mesh: meshes.get_handle(&snake_assets.food_mesh),
            material: materials.get_handle(&snake_assets.food_material),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        ..default()
    });

    debug!(target: "bevypoco::food_spawner", "Spawned new food at {:?}", pos);
}

fn snake_movement(
    mut query_head: Query<(&SnakeHead, &mut Position), Without<LastSnakeSegment>>,
    mut query_last: Query<(&mut LastSnakeSegment, &mut Position), Without<SnakeHead>>,
    mut snake_query: Query<
        &mut Position,
        (
            With<SnakeSegment>,
            Without<LastSnakeSegment>,
            Without<SnakeHead>,
        ),
    >,
) {
    let Ok((SnakeHead(snake_direction), mut head_position)) = query_head.get_single_mut() else {
        return;
    };

    let Ok(( mut last_segment, mut last_position)) = query_last.get_single_mut() else {
        return;
    };

    // save position of last segment before moving it
    last_segment.0 = Some(*last_position);
    debug!(target: "bevypoco::snake_movement", "Saving last segment at {:?}", last_segment.0.unwrap());

    // save position of head before moving it
    let mut old_position = *head_position;
    debug!(target: "bevypoco::snake_movement", "Saving head_position at {:?}", &old_position);

    // move head in direction
    head_position.0 += IVec3::from(*snake_direction);

    debug!(target: "bevypoco::snake_movement", "Moved Head to {:?}", head_position.0);

    // move all segments in snake to the next one based on direction
    for mut pos in snake_query.iter_mut() {
        debug!(target: "bevypoco::snake_movement", "Moved from {:?} to {:?}", *pos, old_position);
        let tmp = *pos;
        *pos = old_position;
        old_position = tmp;
    }

    // move last segment to old position
    *last_position = old_position;
}

/// This system set is used to tick the entitites at a fixed rate
#[derive(Default, SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
struct FixedSet;

fn main() {
    App::new()
        .register_type::<Position>()
        .register_type::<Direction>()
        .register_type::<SnakeHead>()
        .register_type::<LastSnakeSegment>()
        .configure_set(
            FixedSet::default()
                .run_if(on_fixed_timer(Duration::from_millis(1300)))
                .in_base_set(StartupSet::PostStartup),
        )
        .add_event::<EatEvent>()
        .insert_resource(AmbientLight {
            brightness: 1.,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.1)))
        .add_startup_systems((load_meshes, setup_window, setup_camera))
        .add_startup_system(setup_scene.in_base_set(StartupSet::PostStartup))
        .add_system(position_translation)
        .add_systems((snake_growth, food_spawner).chain())
        .add_systems((snake_movement, eat_food).chain().in_set(FixedSet))
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_editor_pls::EditorPlugin::new())
        .run();
}
