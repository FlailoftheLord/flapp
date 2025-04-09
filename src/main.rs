use bevy::{prelude::*, window::PrimaryWindow};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use rand::{rngs::ThreadRng, thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins((
            EmbeddedAssetPlugin {
                mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
            },
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from(WINDOW_TITLE),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        resolution: Vec2::new(WIN_X, WIN_Y).into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        ))
        .add_systems(Startup, setup_level)
        .add_systems(
            Update,
            (
                show_pause_screen,
                update_bird,
                update_obstacles,
                update_score_text,
            ),
        )
        .run();
}
//game general
const WIN_X: f32 = 1280.;
const WIN_Y: f32 = 720.;
const WINDOW_TITLE: &str = "Flapp Birb";
const BACKGROUND_COLOR: Color = Color::srgb(0.5, 0.7, 0.8);
const PIXEL_RATIO: f32 = 4.5;

//pause screen
const PAUSE_TEXT_COLOR: Color = Color::srgb(1., 0.5, 0.2);
const PAUSE_TEXT_SIZE: f32 = 28.;
const PAUSE_TEXT_1: &str = "Flap Flap Away~";
const PAUSE_TEXT_2: &str = "press [space] to start.";

//score display
const SCORE_DISPLAY: &str = "Score: ";
const SCORE_TEXT_COLOR: Color = Color::srgb(1., 1., 0.);
const SCORE_TEXT_SIZE: f32 = 10.;
const SCORE_POS_PAD_X: f32 = 30.;
const SCORE_POS_PAD_Y: f32 = 15.;

//bird
const FLAP_KEY: KeyCode = KeyCode::Space;
const FLAP_FORCE: f32 = 400.;
const VELOCITY_ROT_RATIO: f32 = 7.2;
const GRAVITY: f32 = 1600.;

//obstacles and collision
const MERCY_ZONE: f32 = 5.;
const OBSTACLE_AMOUNT: i32 = 8;
const OBSTACLE_WIDTH: f32 = 32.;
const OBSTACLE_HEIGHT: f32 = 144.;
const OBSTACLE_VERTICAL_OFFSET: f32 = 30.;
const OBSTACLE_GAP: f32 = 16.;
const OBSTACLE_SPACING: f32 = 64.;
const OBSTACLE_SCROLL_SPEED: f32 = 120.;

#[derive(Resource)]
pub struct GameManager {
    pub bird_image: Handle<Image>,
    pub pipe_image: Handle<Image>,
    pub window_dimensions: Vec2,
}

#[derive(Resource)]
struct Score {
    value: u32,
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct Bird {
    pub velocity: f32,
}

#[derive(Component)]
struct PauseText;

#[derive(Component)]
pub struct Obstacle {
    pipe_direction: f32,
}

fn setup_level(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let bird_image = asset_server.load("bird.png");
    let pipe_image = asset_server.load("pipe.png");
    let window = window_query.get_single().expect("Window not queryable");
    commands.insert_resource(GameManager {
        bird_image: bird_image.clone(),
        pipe_image: pipe_image.clone(),
        window_dimensions: Vec2::new(window.width(), window.height()),
    });

    //score
    commands.insert_resource(Score { value: 0 });

    //background color
    commands.insert_resource(ClearColor(BACKGROUND_COLOR));

    //camera
    commands.spawn(Camera2d::default());

    //bird
    spawn_bird(&mut commands, &bird_image, 1.);

    //obstacles
    let mut rand = thread_rng();
    spawn_obstacles(&mut commands, &mut rand, window.width(), &pipe_image);

    //score
    commands.spawn((
        Text2d::new(SCORE_DISPLAY),
        TextFont {
            font_size: SCORE_TEXT_SIZE * PIXEL_RATIO,
            ..Default::default()
        },
        TextColor(SCORE_TEXT_COLOR),
        Transform::from_xyz(
            -window.width() / 2. + (SCORE_POS_PAD_X * PIXEL_RATIO),
            window.height() / 2. - (SCORE_POS_PAD_Y * PIXEL_RATIO),
            1.,
        ),
        ScoreText,
    ));
}

fn get_centered_pos() -> f32 {
    return (OBSTACLE_HEIGHT / 2. + OBSTACLE_GAP) * PIXEL_RATIO;
}

fn generate_offset(rand: &mut ThreadRng) -> f32 {
    return rand.gen_range(-OBSTACLE_VERTICAL_OFFSET..OBSTACLE_VERTICAL_OFFSET) * PIXEL_RATIO;
}

fn spawn_bird(commands: &mut Commands, bird_image: &Handle<Image>, scale: f32) {
    commands.spawn((
        Sprite {
            image: bird_image.clone(),
            ..Default::default()
        },
        Transform::IDENTITY.with_scale(Vec3::splat(PIXEL_RATIO * scale)),
        Bird { velocity: 0. },
    ));
}

fn spawn_obstacles(
    commands: &mut Commands,
    rand: &mut ThreadRng,
    window_width: f32,
    pipe_image: &Handle<Image>,
) {
    for i in 0..OBSTACLE_AMOUNT {
        let y_offset: f32 = generate_offset(rand);
        let x_pos: f32 = (window_width / 2.) + (OBSTACLE_SPACING * PIXEL_RATIO * i as f32);
        //top
        obstacle(
            Vec3::X * x_pos + Vec3::Y * (get_centered_pos() + y_offset),
            1.,
            commands,
            pipe_image,
        );
        //bottom
        obstacle(
            Vec3::X * x_pos + Vec3::Y * (-get_centered_pos() + y_offset),
            -1.,
            commands,
            pipe_image,
        );
    }
}

//spawn singular pipe
fn obstacle(
    translation: Vec3,
    pipe_direction: f32,
    commands: &mut Commands,
    pipe_image: &Handle<Image>,
) {
    commands.spawn((
        Sprite {
            image: pipe_image.clone(),
            ..Default::default()
        },
        Transform::from_translation(translation).with_scale(Vec3::new(
            PIXEL_RATIO,
            PIXEL_RATIO * -pipe_direction,
            PIXEL_RATIO,
        )),
        Obstacle { pipe_direction },
    ));
}

fn update_obstacles(
    time: Res<Time>,
    game_manager: Res<GameManager>,
    mut obstacle_query: Query<(&mut Obstacle, &mut Transform)>,
) {
    let mut rand = thread_rng();
    let y_offset = generate_offset(&mut rand);
    for (obstacle, mut transform) in obstacle_query.iter_mut() {
        transform.translation.x -= time.delta_secs() * OBSTACLE_SCROLL_SPEED;
        if transform.translation.x + OBSTACLE_WIDTH * PIXEL_RATIO / 2.
            < -game_manager.window_dimensions.x / 2.
        {
            transform.translation.x += OBSTACLE_AMOUNT as f32 * OBSTACLE_SPACING * PIXEL_RATIO;
            transform.translation.y = get_centered_pos() * obstacle.pipe_direction + y_offset;
        }
    }
}

fn update_bird(
    mut commands: Commands,
    mut bird_query: Query<(&mut Bird, &mut Transform), Without<Obstacle>>,
    obstacle_query: Query<(&Transform, Entity), With<Obstacle>>,
    mut time: ResMut<Time<Virtual>>,
    keys: Res<ButtonInput<KeyCode>>,
    game_manager: Res<GameManager>,
    mut score: ResMut<Score>,
) {
    let mut dead = false;
    if let Ok((mut bird, mut transform)) = bird_query.get_single_mut() {
        if !time.is_paused() && !dead {
            if keys.just_pressed(FLAP_KEY) {
                bird.velocity = FLAP_FORCE;
            }

            bird.velocity -= time.delta_secs() * GRAVITY;
            transform.translation.y += bird.velocity * time.delta_secs();
            transform.rotation = Quat::from_axis_angle(
                Vec3::Z,
                f32::clamp(bird.velocity / VELOCITY_ROT_RATIO, -90., 90.).to_radians(),
            );

            if transform.translation.y <= -game_manager.window_dimensions.y / 2. {
                dead = true;
            } else {
                for (pipe_transform, _entity) in obstacle_query.iter() {
                    if pipe_transform.translation.x - transform.translation.x > 0.
                        && pipe_transform.translation.x - transform.translation.x
                            < OBSTACLE_SCROLL_SPEED * time.delta_secs()
                        && pipe_transform.translation.y > 0.
                    {
                        score.value += 1;
                    }
                    //collision check
                    if (pipe_transform.translation.y - transform.translation.y).abs()
                        < (OBSTACLE_HEIGHT - MERCY_ZONE) * PIXEL_RATIO / 2.
                        && (pipe_transform.translation.x - transform.translation.x).abs()
                            < (OBSTACLE_WIDTH - MERCY_ZONE) * PIXEL_RATIO / 2.
                    {
                        dead = true;
                        break;
                    }
                }
            }
        } else {
            if keys.just_pressed(FLAP_KEY) {
                dead = false;
                reset_game(
                    commands,
                    bird,
                    transform,
                    obstacle_query,
                    game_manager,
                    score,
                );
                time.unpause();
            }
        }

        if dead && !time.is_paused() {
            time.pause();
        }
    } else {
        if !dead {
            spawn_bird(&mut commands, &game_manager.bird_image, 1.);
        }
    }
}

fn score_text(score: u32) -> String {
    String::from(SCORE_DISPLAY) + format!("{}", score).as_str()
}

fn update_score_text(score: ResMut<Score>, mut query: Query<&mut Text2d, With<ScoreText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.0 = score_text(score.value);
    }
}

fn reset_game(
    mut commands: Commands,
    mut bird: Mut<Bird>,
    mut transform: Mut<Transform>,
    mut obstacle_query: Query<(&Transform, Entity), With<Obstacle>>,
    game_manager: Res<GameManager>,
    mut score: ResMut<Score>,
) {
    transform.translation = Vec3::ZERO;
    bird.velocity = 0.;
    score.value = 0;
    for (_pipe_transform, entity) in obstacle_query.iter_mut() {
        commands.entity(entity).despawn();
    }
    let mut rand = thread_rng();
    spawn_obstacles(
        &mut commands,
        &mut rand,
        game_manager.window_dimensions.x,
        &game_manager.pipe_image,
    );
}

//show pause screen when the game time is paused
fn show_pause_screen(
    score: Res<Score>,
    time: Res<Time<Virtual>>,
    mut commands: Commands,
    game_manager: Res<GameManager>,
    mut bird_query: Query<Entity, With<Bird>>,
    mut text_query: Query<Entity, With<PauseText>>,
) {
    if time.is_paused() {
        if let Ok(entity) = bird_query.get_single_mut() {
            commands.entity(entity).despawn();
        }

        //pause text
        let window_dimensions = game_manager.window_dimensions;
        commands.spawn_batch(vec![
            (
                Text2d::new(PAUSE_TEXT_1),
                TextFont {
                    font_size: PAUSE_TEXT_SIZE * PIXEL_RATIO,
                    ..Default::default()
                },
                TextColor(PAUSE_TEXT_COLOR),
                Transform::from_xyz(0., window_dimensions.y / 6., 1.),
                PauseText,
            ),
            (
                Text2d::new(PAUSE_TEXT_2),
                TextFont {
                    font_size: (PAUSE_TEXT_SIZE / 3.) * PIXEL_RATIO,
                    ..Default::default()
                },
                TextColor(PAUSE_TEXT_COLOR),
                Transform::from_xyz(0., -window_dimensions.y / 6., 1.),
                PauseText,
            ),
            (
                Text2d::new(score_text(score.value)),
                TextFont {
                    font_size: (PAUSE_TEXT_SIZE / 1.5) * PIXEL_RATIO,
                    ..Default::default()
                },
                TextColor(PAUSE_TEXT_COLOR),
                Transform::from_xyz(0., 0., 1.),
                PauseText,
            ),
        ]);
    } else {
        for t in text_query.iter_mut() {
            commands.entity(t).despawn();
        }
    }
}
