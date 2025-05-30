// Bevy Tower Defense Game with Boids Flocking Simulation
// This game combines a menu interface with a tower defense mechanic where
// turrets shoot at flocking boids (bird-like entities that move in groups)

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::prelude::*;

fn main() {
    App::new()
        // Configure the main window with title and resolution
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Game Menu".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        // Set background color to dark gray
        .insert_resource(ClearColor(Color::srgb(0.15, 0.15, 0.15)))
        // Initialize all game systems on startup
        .add_systems(Startup, (setup_camera, setup_menu, setup_boids, setup_turrets))
        // Systems that run every frame
        .add_systems(Update, (
            button_system,        // Handle menu button interactions
            update_boids,         // Update boid movement and flocking behavior
            draw_boids,          // Render boids with proper orientation and colors
            bounce_boids,        // Handle screen wrapping for boids
            update_turrets,      // Turret targeting and laser creation
            update_lasers,       // Update laser beam positions and lengths
            apply_laser_damage,  // Apply damage to targeted boids
            respawn_boids,       // Maintain boid population
        ))
        .run();
}

// ===== COMPONENT DEFINITIONS =====

/// Marker component for the main menu UI
#[derive(Component)]
struct MainMenu;

/// Core boid component containing movement and health data
#[derive(Component)]
struct Boid {
    velocity: Vec2,              // Current movement direction and speed
    acceleration: Vec2,          // Forces applied this frame
    health: f32,                // Health from 0.0 to 1.0
    damage_flash_timer: Timer,   // Timer for red damage flash effect
}

/// Marker component for boid visual representations (triangular meshes)
#[derive(Component)]
struct BoidVisual;

/// Turret component for defensive structures
#[derive(Component)]
struct Turret {
    target: Option<Entity>,      // Currently targeted boid entity
    range: f32,                  // Maximum targeting range
    cooldown_timer: Timer,       // Delay between target acquisitions
}

/// Laser beam component linking beams to their source turrets
#[derive(Component)]
struct LaserBeam {
    turret: Entity,              // Which turret owns this laser
}

/// Enum defining different menu button types
#[derive(Component)]
enum MenuButton {
    SinglePlayer,
    Multiplayer,
    Settings,
    Quit,
    Character,
}

// ===== SETUP SYSTEMS =====

/// Initialize the 2D camera for the game
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Create the main menu UI with buttons and title
fn setup_menu(mut commands: Commands) {
    // Root UI container taking full screen
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,  // Space items apart
                align_items: AlignItems::FlexEnd,               // Align to bottom
                padding: UiRect::all(Val::Px(40.0)),           // 40px padding on all sides
                ..default()
            },
            MainMenu,
        ))
        .with_children(|parent| {
            // Left side menu container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,      // Stack buttons vertically
                    align_items: AlignItems::FlexStart,         // Align to left
                    row_gap: Val::Px(20.0),                    // 20px gap between buttons
                    ..default()
                })
                .with_children(|parent| {
                    // Character button (special placement at top)
                    spawn_menu_button(parent, "Character", MenuButton::Character);
                    
                    // Visual separator line
                    parent.spawn((
                        Node {
                            width: Val::Px(250.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(10.0)),  // 10px margin top/bottom
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),  // Semi-transparent white
                    ));
                    
                    // Main menu buttons
                    spawn_menu_button(parent, "Single Player", MenuButton::SinglePlayer);
                    spawn_menu_button(parent, "Multiplayer", MenuButton::Multiplayer);
                    spawn_menu_button(parent, "Settings", MenuButton::Settings);
                    spawn_menu_button(parent, "Quit", MenuButton::Quit);
                });

            // Game title positioned in top right corner
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,  // Absolute positioning
                    top: Val::Px(40.0),                    // 40px from top
                    right: Val::Px(40.0),                  // 40px from right
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("TITLE HERE"),
                        TextFont {
                            font_size: 72.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Helper function to create individual menu buttons
fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    button_type: MenuButton,
) {
    parent
        .spawn((
            Button,                                      // Bevy button component
            Node {
                width: Val::Px(250.0),                  // Fixed width
                height: Val::Px(50.0),                  // Fixed height
                justify_content: JustifyContent::Center, // Center text horizontally
                align_items: AlignItems::Center,         // Center text vertically
                ..default()
            },
            BackgroundColor(Color::NONE),               // Transparent background
            button_type,                                // Button type for identification
        ))
        .with_children(|parent| {
            // Button text child
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

// ===== UI INTERACTION SYSTEM =====

/// Handle button interactions (hover, click effects)
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>),  // Only run when interaction changes
    >,
    mut text_query: Query<&mut TextColor>,
    mut exit: EventWriter<AppExit>,            // For quitting the application
) {
    for (interaction, button_type, mut color, children) in &mut interaction_query {
        // Determine text color based on interaction state
        let text_color_value = match *interaction {
            Interaction::Pressed => {
                // Handle button actions
                match button_type {
                    MenuButton::Quit => {
                        exit.write(AppExit::Success);  // Exit application
                    }
                    _ => {}  // Other buttons don't have actions yet
                }
                Color::srgb(0.6, 0.6, 0.6)  // Dark gray when pressed
            }
            Interaction::Hovered => Color::srgb(0.8, 0.8, 0.8),  // Light gray when hovered
            Interaction::None => Color::WHITE,                     // White when normal
        };

        // Keep button background transparent
        *color = BackgroundColor(Color::NONE);
        
        // Update text color for all child text elements
        for child in children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                text_color.0 = text_color_value;
            }
        }
    }
}

// ===== BOID SETUP AND SIMULATION =====

/// Initialize the boid population with different types
fn setup_boids(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else { return; };
    let mut rng = rand::rng();
    
    // Spawn main flock of 150 white boids with random positions and velocities
    for _ in 0..150 {
        // Random position within window bounds
        let position = Vec2::new(
            rng.random_range(-window.width() / 2.0..window.width() / 2.0),
            rng.random_range(-window.height() / 2.0..window.height() / 2.0),
        );
        
        // Start with varied but consistent velocities for natural movement
        let angle = rng.random_range(0.0..std::f32::consts::TAU);  // TAU = 2π
        let speed = rng.random_range(300.0..500.0);
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
        
        commands.spawn((
            Boid {
                velocity,
                acceleration: Vec2::ZERO,
                health: 1.0,  // Full health
                damage_flash_timer: Timer::from_seconds(0.5, TimerMode::Once),
            },
            Transform::from_translation(position.extend(0.0)),  // Convert Vec2 to Vec3
        ));
    }
    
    // Spawn special colored boids for visual variety
    
    // Single pink boid at specific position
    commands.spawn((
        Boid {
            velocity: Vec2::new(rng.random_range(-150.0..150.0), rng.random_range(-150.0..150.0)),
            acceleration: Vec2::ZERO,
            health: 1.0,
            damage_flash_timer: Timer::from_seconds(0.5, TimerMode::Once),
        },
        Transform::from_translation(Vec3::new(200.0, 100.0, 1.0)),  // Z=1.0 marks special boids
    ));
    
    // Three red boids in bottom right corner
    for i in 0..3 {
        commands.spawn((
            Boid {
                velocity: Vec2::new(rng.random_range(-100.0..100.0), rng.random_range(-100.0..100.0)),
                acceleration: Vec2::ZERO,
                health: 1.0,
                damage_flash_timer: Timer::from_seconds(0.1, TimerMode::Once),  // Faster flash
            },
            Transform::from_translation(Vec3::new(
                window.width() / 2.0 - 100.0 - i as f32 * 30.0,  // Spaced 30px apart
                -window.height() / 2.0 + 100.0,                   // Near bottom
                1.0,                                               // Mark as special
            )),
        ));
    }
}

/// Update boid movement using flocking algorithm (separation, alignment, cohesion)
fn update_boids(
    mut boids: Query<(&mut Boid, &mut Transform, Entity)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let Ok(window) = window_query.single() else { return; };
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    
    // Collect all boid positions and velocities for flocking calculations
    // This prevents borrowing issues when calculating neighbor interactions
    let boids_data: Vec<(Vec2, Vec2, Entity)> = boids
        .iter()
        .map(|(boid, transform, entity)| {
            (transform.translation.truncate(), boid.velocity, entity)
        })
        .collect();
    
    for (mut boid, mut transform, entity) in &mut boids {
        let pos = transform.translation.truncate();
        
        // Update damage flash timer
        boid.damage_flash_timer.tick(time.delta());
        
        // Reset acceleration for this frame
        boid.acceleration = Vec2::ZERO;
        
        // ===== EDGE AVOIDANCE FORCE =====
        // Apply forces to keep boids away from screen edges with smooth curves
        let edge_margin = 150.0;     // Distance from edge where force starts
        let edge_force = 300.0;      // Maximum force strength
        
        // Right edge avoidance
        if pos.x > half_width - edge_margin {
            let distance_to_edge = half_width - pos.x;
            let force = (1.0 - distance_to_edge / edge_margin).powf(2.0) * edge_force;
            boid.acceleration.x -= force;  // Push left
        } 
        // Left edge avoidance
        else if pos.x < -half_width + edge_margin {
            let distance_to_edge = pos.x + half_width;
            let force = (1.0 - distance_to_edge / edge_margin).powf(2.0) * edge_force;
            boid.acceleration.x += force;  // Push right
        }
        
        // Top edge avoidance
        if pos.y > half_height - edge_margin {
            let distance_to_edge = half_height - pos.y;
            let force = (1.0 - distance_to_edge / edge_margin).powf(2.0) * edge_force;
            boid.acceleration.y -= force;  // Push down
        } 
        // Bottom edge avoidance
        else if pos.y < -half_height + edge_margin {
            let distance_to_edge = pos.y + half_height;
            let force = (1.0 - distance_to_edge / edge_margin).powf(2.0) * edge_force;
            boid.acceleration.y += force;  // Push up
        }
        
        // ===== FLOCKING BEHAVIOR (Craig Reynolds' Boids Algorithm) =====
        let mut separation = Vec2::ZERO;  // Avoid crowding neighbors
        let mut alignment = Vec2::ZERO;   // Steer towards average heading of neighbors
        let mut cohesion = Vec2::ZERO;    // Steer towards average position of neighbors
        let mut neighbors = 0;
        
        let perception_radius = 100.0;  // How far boids can "see" each other
        let max_speed = 600.0;          // Maximum movement speed
        let max_force = 400.0;          // Maximum steering force
        
        // Check all other boids for flocking interactions
        for &(other_pos, other_vel, other_entity) in &boids_data {
            if entity == other_entity {
                continue;  // Skip self
            }
            
            let distance = pos.distance(other_pos);
            
            // Only consider boids within perception range
            if distance < perception_radius && distance > 0.0 {
                // SEPARATION: Avoid crowding (most important for natural movement)
                if distance < 40.0 {  // Personal space radius
                    let diff = (pos - other_pos).normalize_or_zero();
                    let force_strength = (40.0 - distance) / 40.0;  // Stronger when closer
                    separation += diff * force_strength;
                }
                
                // ALIGNMENT: Match velocity of neighbors
                alignment += other_vel;
                
                // COHESION: Move towards center of local group
                cohesion += other_pos;
                
                neighbors += 1;
            }
        }
        
        // Apply flocking forces if neighbors were found
        if neighbors > 0 {
            // Calculate average values
            alignment /= neighbors as f32;
            cohesion /= neighbors as f32;
            cohesion = cohesion - pos;  // Vector towards center
            
            // Convert to steering forces (desired velocity - current velocity)
            if separation.length() > 0.0 {
                separation = separation.normalize() * max_force;
            }
            if alignment.length() > 0.0 {
                let desired = alignment.normalize() * max_speed;
                alignment = (desired - boid.velocity) * 0.05;  // Gentle alignment
            }
            if cohesion.length() > 0.0 {
                let desired = cohesion.normalize() * max_speed;
                cohesion = (desired - boid.velocity) * 0.02;  // Gentle cohesion
            }
            
            // Apply forces with different weights for natural behavior
            boid.acceleration += separation * 2.0;  // Separation is most important
            boid.acceleration += alignment;         // Medium importance
            boid.acceleration += cohesion;          // Least important
        }
        
        // ===== WANDERING BEHAVIOR =====
        // Add some randomness to prevent perfectly uniform movement
        let wander_angle = time.elapsed_secs() * 2.0 + entity.index() as f32 * 0.5;
        let wander_force = Vec2::new(
            wander_angle.sin() * 20.0,
            (wander_angle * 1.3).cos() * 20.0,  // Different frequency for Y
        );
        boid.acceleration += wander_force;
        
        // ===== VELOCITY AND POSITION UPDATES =====
        // Apply acceleration to velocity with damping for smoother movement
        let acceleration_delta = boid.acceleration * time.delta_secs();
        boid.velocity += acceleration_delta;
        boid.velocity *= 0.99;  // Slight damping to prevent excessive speed buildup
        boid.velocity = boid.velocity.clamp_length_max(max_speed);

        // Ensure minimum speed to prevent boids from stopping completely
        if boid.velocity.length() < 100.0 {
            boid.velocity = boid.velocity.normalize_or_zero() * 100.0;
        }
        
        // Apply velocity again (this appears to be duplicate code - could be optimized)
        let delta_velocity = boid.acceleration * time.delta_secs();
        boid.velocity += delta_velocity;
        boid.velocity = boid.velocity.clamp_length_max(max_speed);
        
        // Update position based on velocity
        transform.translation.x += boid.velocity.x * time.delta_secs();
        transform.translation.y += boid.velocity.y * time.delta_secs();
    }
}

/// Handle screen wrapping - boids that exit one side appear on the opposite side
fn bounce_boids(
    mut boids: Query<&mut Transform, With<Boid>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else { return; };
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    
    for mut transform in &mut boids {
        // Horizontal wrapping
        if transform.translation.x > half_width {
            transform.translation.x = -half_width;
        } else if transform.translation.x < -half_width {
            transform.translation.x = half_width;
        }
        
        // Vertical wrapping
        if transform.translation.y > half_height {
            transform.translation.y = -half_height;
        } else if transform.translation.y < -half_height {
            transform.translation.y = half_height;
        }
    }
}

/// Create and update visual representations of boids (triangular meshes)
fn draw_boids(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    boids: Query<(Entity, &Transform, &Boid), Without<BoidVisual>>,  // Boids without visuals
    mut visuals: Query<(&mut Transform, &ChildOf, &mut MeshMaterial2d<ColorMaterial>), With<BoidVisual>>,
) {
    // Create triangle mesh pointing forward (used for all boids)
    let triangle_mesh = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 5.0),    // Top point (forward)
        Vec2::new(-3.0, -3.0),  // Bottom left
        Vec2::new(3.0, -3.0),   // Bottom right
    ));
    
    // Create visual representations for boids that don't have them yet
    for (entity, transform, _boid) in &boids {
        // Check if this boid already has a visual child
        if visuals.iter().find(|(_, child_of, _)| child_of.parent() == entity).is_none() {
            // Determine boid color based on position and Z-coordinate
            let base_color = if transform.translation.z > 0.5 {  // Special boids
                if transform.translation.x > 100.0 {
                    Color::srgb(1.0, 0.0, 0.5)  // Pink boid
                } else {
                    Color::srgb(1.0, 0.2, 0.2)  // Red boids
                }
            } else {
                Color::WHITE  // Normal flock members
            };
            
            let material = materials.add(ColorMaterial::from(base_color));
            
            // Create the visual mesh entity
            let visual = commands
                .spawn((
                    Mesh2d(triangle_mesh.clone()),
                    MeshMaterial2d(material),
                    Transform::from_scale(Vec3::splat(1.0)),
                    BoidVisual,
                ))
                .id();
            
            // Make visual a child of the boid entity
            commands.entity(entity).add_child(visual);
        }
    }
    
    // Update existing visual representations
    for (mut visual_transform, child_of, material_handle) in &mut visuals {
        if let Ok((_, transform, boid)) = boids.get(child_of.parent()) {
            // Update rotation to point in movement direction
            let angle = boid.velocity.y.atan2(boid.velocity.x) - std::f32::consts::FRAC_PI_2;
            visual_transform.rotation = Quat::from_rotation_z(angle);
            
            // Update color based on health and damage state
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Determine base color (same logic as creation)
                let base_color = if transform.translation.z > 0.5 {
                    if transform.translation.x > 100.0 {
                        Color::srgb(1.0, 0.0, 0.5)  // Pink
                    } else {
                        Color::srgb(1.0, 0.2, 0.2)  // Red
                    }
                } else {
                    Color::WHITE
                };
                
                // Apply damage flash effect if timer is active
                if !boid.damage_flash_timer.finished() {
                    // Create flashing effect with sine wave
                    let flash_progress = boid.damage_flash_timer.elapsed_secs() / boid.damage_flash_timer.duration().as_secs_f32();
                    let flash_intensity = (flash_progress * 10.0 * std::f32::consts::PI).sin().abs();
                    
                    // Flash to bright red regardless of base color
                    material.color = if flash_intensity > 0.5 {
                        Color::srgb(1.0, 0.0, 0.0)  // Bright red flash
                    } else {
                        base_color
                    };
                } else if boid.health < 1.0 {
                    // Show damage by darkening the color based on health
                    let health_factor = boid.health;
                    let srgba = base_color.to_srgba();
                    material.color = Color::srgb(
                        srgba.red * health_factor,
                        srgba.green * health_factor,
                        srgba.blue * health_factor,
                    );
                } else {
                    // Full health - use normal base color
                    material.color = base_color;
                }
            }
        }
    }
}

// ===== TURRET SYSTEMS =====

/// Create defensive turrets at strategic positions around the map
fn setup_turrets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else { return; };
    
    // Create meshes for turret components
    let turret_base = meshes.add(Rectangle::new(20.0, 20.0));      // Square base
    let turret_barrel = meshes.add(Rectangle::new(4.0, 15.0));     // Rectangular barrel
    let turret_material = materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.3)));  // Dark gray
    
    // Strategic turret positions for good map coverage
    let positions = vec![
        Vec2::new(-window.width() / 3.0, -window.height() / 3.0),  // Bottom left
        Vec2::new(window.width() / 3.0, -window.height() / 3.0),   // Bottom right
        Vec2::new(0.0, window.height() / 3.0),                     // Top center
        Vec2::new(-window.width() / 4.0, window.height() / 4.0),   // Top left
        Vec2::new(window.width() / 4.0, window.height() / 4.0),    // Top right
    ];
    
    for pos in positions {
        // Spawn turret base with targeting logic
        commands
            .spawn((
                Mesh2d(turret_base.clone()),
                MeshMaterial2d(turret_material.clone()),
                Transform::from_translation(pos.extend(-1.0)),  // Behind boids in Z-order
                Turret {
                    target: None,                                    // No initial target
                    range: 250.0,                                   // Targeting range
                    cooldown_timer: Timer::from_seconds(0.5, TimerMode::Once),  // Target acquisition delay
                },
            ))
            .with_children(|parent| {
                // Spawn turret barrel as child (rotates with targeting)
                parent.spawn((
                    Mesh2d(turret_barrel.clone()),
                    MeshMaterial2d(turret_material.clone()),
                    Transform::from_xyz(0.0, 10.0, 0.1),  // Offset forward from base
                ));
            });
    }
}

/// Update turret targeting logic and create laser beams
fn update_turrets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut turrets: Query<(Entity, &mut Turret, &Transform, &Children)>,
    mut barrel_transforms: Query<&mut Transform, (Without<Turret>, Without<Boid>)>,  // Turret barrels
    boids: Query<(&Transform, Entity), (With<Boid>, Without<Turret>)>,
    existing_beams: Query<&LaserBeam>,
    time: Res<Time>,
) {
    for (turret_entity, mut turret, turret_transform, children) in &mut turrets {
        // Update targeting cooldown timer
        turret.cooldown_timer.tick(time.delta());
        
        // ===== TARGET VALIDATION =====
        // Check if current target is still valid and within range
        let mut target_valid = false;
        if let Some(target_entity) = turret.target {
            if let Ok((boid_transform, _)) = boids.get(target_entity) {
                let distance = turret_transform
                    .translation
                    .truncate()
                    .distance(boid_transform.translation.truncate());
                target_valid = distance < turret.range;
            }
        }
        
        // If target is lost, clear it and start cooldown before finding new target
        if !target_valid && turret.target.is_some() {
            turret.target = None;
            turret.cooldown_timer.reset();
        }
        
        // ===== TARGET ACQUISITION =====
        // Find new target only after cooldown expires
        if turret.target.is_none() && turret.cooldown_timer.finished() {
            let mut closest_distance = f32::MAX;
            
            // Search for closest boid within range
            for (boid_transform, boid_entity) in &boids {
                let distance = turret_transform
                    .translation
                    .truncate()
                    .distance(boid_transform.translation.truncate());
                
                if distance < turret.range && distance < closest_distance {
                    closest_distance = distance;
                    turret.target = Some(boid_entity);
                }
            }
        }
        
        // ===== BARREL ROTATION AND LASER CREATION =====
        if let Some(target_entity) = turret.target {
            if let Ok((boid_transform, _)) = boids.get(target_entity) {
                // Calculate direction to target
                let direction = (boid_transform.translation.truncate() - turret_transform.translation.truncate()).normalize();
                let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                
                // Rotate turret barrel to face target
                for child in children.iter() {
                    if let Ok(mut barrel_transform) = barrel_transforms.get_mut(child) {
                        barrel_transform.rotation = Quat::from_rotation_z(angle);
                    }
                }
                
                // Create laser beam if one doesn't exist for this turret
                let has_beam = existing_beams.iter().any(|beam| beam.turret == turret_entity);
                if !has_beam {
                    let distance = turret_transform
                        .translation
                        .truncate()
                        .distance(boid_transform.translation.truncate());
                    
                    // Create laser mesh spanning the distance to target
                    let laser_mesh = meshes.add(Rectangle::new(2.0, distance));
                    let laser_material = materials.add(ColorMaterial::from(Color::srgba(1.0, 0.0, 0.0, 0.7)));  // Semi-transparent red
                    
                    // Spawn laser beam positioned between turret and target
                    commands.spawn((
                        Mesh2d(laser_mesh),
                        MeshMaterial2d(laser_material),
                        Transform::from_translation(turret_transform.translation + (direction * distance / 2.0).extend(0.0))
                            .with_rotation(Quat::from_rotation_z(angle)),
                        LaserBeam { turret: turret_entity },
                    ));
                }
            }
        }
    }
}

/// Update laser beam positions and lengths to track moving targets
fn update_lasers(
    mut commands: Commands,
    mut lasers: Query<(Entity, &LaserBeam, &mut Transform, &MeshMaterial2d<ColorMaterial>, &Mesh2d)>,
    turrets: Query<(&Turret, &Transform), Without<LaserBeam>>,
    boids: Query<&Transform, (With<Boid>, Without<LaserBeam>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (laser_entity, laser_beam, mut laser_transform, _, mesh_handle) in &mut lasers {
        // Get the turret that owns this laser
        if let Ok((turret, turret_transform)) = turrets.get(laser_beam.turret) {
            // Check if turret still has a target
            if let Some(target_entity) = turret.target {
                if let Ok(boid_transform) = boids.get(target_entity) {
                    // Update laser to connect turret and target
                    let direction = boid_transform.translation.truncate() - turret_transform.translation.truncate();
                    let distance = direction.length();
                    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                    
                    // Position laser at midpoint between turret and target
                    laser_transform.translation = turret_transform.translation + (direction.normalize() * distance / 2.0).extend(0.0);
                    laser_transform.rotation = Quat::from_rotation_z(angle);
                    
                    // Update laser mesh length to match current distance
                    if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
                        *mesh = Rectangle::new(2.0, distance).into();
                    }
                } else {
                    // Target entity no longer exists, remove laser
                    commands.entity(laser_entity).despawn();
                }
            } else {
                // Turret has no target, remove laser
                commands.entity(laser_entity).despawn();
            }
        }
    }
}

/// Apply damage to boids being targeted by turrets
fn apply_laser_damage(
    mut commands: Commands,
    turrets: Query<(&Turret, &Transform)>,
    mut boids: Query<(Entity, &mut Boid, &Transform)>,
    time: Res<Time>,
) {
    let damage_per_second = 0.5;  // Takes 2 seconds to kill a boid (1.0 health / 0.5 damage)
    
    for (turret, turret_transform) in &turrets {
        if let Some(target_entity) = turret.target {
            if let Ok((boid_entity, mut boid, boid_transform)) = boids.get_mut(target_entity) {
                // Verify target is still in range
                let distance = turret_transform
                    .translation
                    .truncate()
                    .distance(boid_transform.translation.truncate());
                
                if distance <= turret.range {
                    // Apply damage over time
                    boid.health -= damage_per_second * time.delta_secs();
                    
                    // Trigger damage flash effect
                    if boid.damage_flash_timer.finished() {
                        boid.damage_flash_timer = Timer::from_seconds(0.5, TimerMode::Once);
                    }
                    
                    // Destroy boid when health is depleted
                    if boid.health <= 0.0 {
                        commands.entity(boid_entity).despawn();
                    }
                }
            }
        }
    }
}

/// Maintain boid population by spawning new boids when others are destroyed
fn respawn_boids(
    mut commands: Commands,
    boids: Query<&Boid>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_query.single() else { return; };
    let boid_count = boids.iter().count();
    let target_count = 150;  // Maintain population of 150 boids
    
    // Only respawn if population has dropped
    if boid_count < target_count {
        let mut rng = rand::rng();
        
        // Spawn up to 5 new boids per frame (gradual replenishment)
        for _ in 0..(target_count - boid_count).min(5) {
            // Choose random edge to spawn from (0=left, 1=right, 2=bottom, 3=top)
            let edge = rng.random_range(0..4);
            let position = match edge {
                0 => Vec2::new(-window.width() / 2.0, rng.random_range(-window.height() / 2.0..window.height() / 2.0)),  // Left edge
                1 => Vec2::new(window.width() / 2.0, rng.random_range(-window.height() / 2.0..window.height() / 2.0)),   // Right edge
                2 => Vec2::new(rng.random_range(-window.width() / 2.0..window.width() / 2.0), -window.height() / 2.0),   // Bottom edge
                _ => Vec2::new(rng.random_range(-window.width() / 2.0..window.width() / 2.0), window.height() / 2.0),    // Top edge
            };
            
            // Random initial velocity
            let velocity = Vec2::new(
                rng.random_range(-150.0..150.0),
                rng.random_range(-150.0..150.0),
            );
            
            // Spawn new boid at edge
            commands.spawn((
                Boid {
                    velocity,
                    acceleration: Vec2::ZERO,
                    health: 1.0,  // Full health
                    damage_flash_timer: Timer::from_seconds(0.5, TimerMode::Once),
                },
                Transform::from_translation(position.extend(0.0)),  // Z=0 for normal boids
            ));
        }
    }
}