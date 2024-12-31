//force once movement every tick


use bevy::{math::vec2, prelude::*, time::common_conditions::*};
use core::time::Duration;
use rand::prelude::random;


const SNAKE_HEAD_COLOR: Color = Color::srgb(0.2,0.6,0.2);
const SNAKE_SEGMENT_COLOR: Color = Color::srgb(0.1, 0.3, 0.1);
const FOOD_COLOR: Color = Color::srgb(1., 0., 0.);
const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;


#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Up,
    Right,
    Down
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Default, Resource)]
struct LastTailPositon(Option<Position>);

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Resource)]
struct SnakeSegments(Vec<Entity>);

#[derive(Event)]
struct GrowthEvent;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct Food;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPositon::default())
        .add_event::<GrowthEvent>()
        .add_systems(Startup, (setup_camera, spawn_snake))
        .add_systems(Update, snake_movement_input.before(snake_movement))
        .add_systems(FixedUpdate, (snake_movement, snake_eating.after(snake_movement), snake_growth.after(snake_eating)).chain().run_if(on_timer(Duration::from_secs_f32(0.15))))
        .add_systems(FixedUpdate, food_spawner.run_if(on_timer(Duration::from_secs(1))))
        .add_systems(PostUpdate,(position_translation, size_scaling).chain())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake!".to_string(), // <--
                resolution: (500.0, 500.0).into(),
                ..default()
            }),
            ..default()
        }))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn spawn_snake(mut commands: Commands, mut segments: ResMut<SnakeSegments>) {
    *segments = SnakeSegments(vec![
        commands
            .spawn(Sprite::from_color(SNAKE_HEAD_COLOR, vec2(1., 1.)))
            .insert(SnakeHead {
                direction: Direction::Up,
            })
            .insert(SnakeSegment)
            .insert(Position {x: 3, y: 3})
            .insert(Size::square(0.8))
            .id(),
        spawn_segment(commands, Position {x:3, y:2}),
    ])

    
}

fn snake_movement_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::ArrowLeft) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::ArrowUp) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
            Direction::Right
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    }
}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_position: ResMut<LastTailPositon>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0.iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        };
        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
        *last_tail_position = LastTailPositon(Some(*segment_positions.last().unwrap()));
    }
}

fn size_scaling(windows: Query<&Window>, mut q: Query<(&Size, &mut Transform)>) {
    let Ok(window) = windows.get_single() else {return};
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.physical_width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn position_translation(windows: Query<&Window>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window/bound_game;
        pos/bound_game*bound_window - (bound_window/2.) + (tile_size/2.)
    }
    let Ok(window) = windows.get_single() else {return};
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        )
    }
}

fn food_spawner(mut commands: Commands) {
    commands
        .spawn(Sprite::from_color(FOOD_COLOR, vec2(1., 1.)))
        .insert(Food)
        .insert(Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        })
        .insert(Size::square(0.8));
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn(Sprite::from_color(SNAKE_SEGMENT_COLOR, vec2(1., 1.)))
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                growth_writer.send(GrowthEvent);
            }
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPositon>,
    mut segments: ResMut<SnakeSegments>,
    growth_reader: EventReader<GrowthEvent>,
) {
    if !growth_reader.is_empty() {
        segments.0.push(spawn_segment(commands, last_tail_position.0.unwrap()))
    }
}