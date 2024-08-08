use ffi::Rectangle;
use raylib::color::Color;
use raylib::prelude::*;
use std::time::{Duration, Instant};

const FPS: f32 = 60.0;
const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
const PROJ_RADIUS: f32 = 16.0;
const PROJ_SPEED: f32 = 500.0;
const FRAME_DURATION: f32 = 1.0 / FPS;
const RACKET_WIDTH: f32 = 8.0 * PROJ_RADIUS;
const RACKET_HEIGHT: f32 = 16.0;
const RACKET_POS_Y: f32 = WINDOW_HEIGHT - RACKET_HEIGHT * 5.0;
const RACKET_SPEED: f32 = 700.0;
const BRICK_WIDTH: f32 = ((WINDOW_WIDTH - 5.0) / 10.0) - 5.0;
const BRICK_HEIGHT: f32 = 32.0;

const HI_COLOR: [Color; 6] = [
    Color::new(0xFF, 0, 0, 0xFF),
    Color::new(0xFF, 0xFF, 0, 0xFF),
    Color::new(0, 0xFF, 0, 0xFF),
    Color::new(0, 0xFF, 0xFF, 0xFF),
    Color::new(0, 0, 0xFF, 0xFF),
    Color::new(0xFF, 0, 0xFF, 0xFF),
];

const LO_COLOR: [Color; 6] = [
    Color::new(0x3F, 0, 0, 0xFF),
    Color::new(0x3F, 0x2F, 0, 0xFF),
    Color::new(0, 0x3F, 0, 0xFF),
    Color::new(0, 0x3F, 0x3F, 0xFF),
    Color::new(0, 0, 0x3F, 0xFF),
    Color::new(0x3F, 0, 0x3F, 0xFF),
];

fn check_collision_recs(rec1: Rectangle, rec2: Rectangle) -> bool {
    unsafe { ffi::CheckCollisionRecs(rec1, rec2) }
}

fn get_collision_recs(rec1: Rectangle, rec2: Rectangle) -> Rectangle {
    unsafe { ffi::GetCollisionRec(rec1, rec2) }
}

struct Brick {
    x: f32,
    y: f32,
    live: usize,
}

struct Projectile {
    x: f32,
    y: f32,
    speed: f32,
    direction: Vector2,
    already_in_collision: bool,
}

impl Projectile {
    fn new() -> Self {
        Self {
            x: WINDOW_WIDTH / 2.0,
            y: RACKET_POS_Y - PROJ_RADIUS - 1.0,
            speed: PROJ_SPEED,
            direction: Vector2 { x: 1.0, y: -1.0 },
            already_in_collision: false,
        }
    }
}
struct Racket {
    x: f32,
    direction: f32,
}

impl Racket {
    fn new() -> Self {
        Self {
            x: WINDOW_WIDTH / 2.0 - RACKET_WIDTH / 2.0,
            direction: 0.0,
        }
    }
}
struct Game {
    bricks: Vec<Brick>,
    ball: Projectile,
    racket: Racket,
    lives: usize,
    state: State,
    last_frame_instant: Instant,
}

impl Game {
    fn new() -> Self {
        let mut ret = Self {
            ball: Projectile::new(),
            bricks: Vec::new(),
            last_frame_instant: Instant::now(),
            lives: 3,
            racket: Racket::new(),
            state: State::InitialBreak(Instant::now()),
        };
        for j in 0..5 {
            for i in 0..10 {
                ret.bricks.push(Brick {
                    x: 5.0 + (i as f32) * (BRICK_WIDTH + 5.0),
                    y: 100.0 + (j as f32) * (BRICK_HEIGHT + 5.0),
                    live: 1,
                })
            }
        }
        ret
    }

    fn handle_input(&mut self, rl: &RaylibHandle) {
        self.racket.direction = 0.0;
        match (
            rl.is_key_down(KeyboardKey::KEY_LEFT),
            rl.is_key_down(KeyboardKey::KEY_RIGHT),
        ) {
            (true, false) => {
                if let ST::InitialBreak(grace) = self.state {
                    if Instant::now().duration_since(grace) > Duration::from_millis(500) {
                        self.ball.direction.x = -1.0;
                        self.state = ST::Running
                    }
                }
                self.racket.direction = -1.0;
            }
            (false, true) => {
                if let ST::InitialBreak(grace) = self.state {
                    if Instant::now().duration_since(grace) > Duration::from_millis(500) {
                        self.ball.direction.x = 1.0;
                        self.state = ST::Running
                    }
                }
                self.racket.direction = 1.0;
            }
            _ => self.racket.direction = 0.0,
        };

        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            match self.state {
                ST::Paused => self.state = ST::Running,
                ST::Running => self.state = ST::Paused,
                _ => (),
            }
        }

        if let ST::Winning | ST::GameOver = self.state {
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                *self = Game::new();
            }
        }
    }

    fn calculate_physics(&mut self, duration: &Duration) {
        if self.ball.y >= WINDOW_HEIGHT + PROJ_RADIUS {
            if self.lives == 0 {
                self.state = ST::GameOver;
            } else {
                self.state = ST::InitialBreak(Instant::now());
                self.lives -= 1;
                self.ball = Projectile::new();
                self.racket = Racket::new();
            }
        }

        if let ST::Running = self.state {
            if self.ball.y <= 0.0 {
                self.ball.speed += 2.0;
                self.ball.direction.y = 1.0;
            }

            if self.ball.x <= PROJ_RADIUS {
                self.ball.speed += 2.0;
                self.ball.direction.x = 1.0;
            }

            if self.ball.x >= WINDOW_WIDTH - PROJ_RADIUS {
                self.ball.speed += 2.0;
                self.ball.direction.x = -1.0;
            }

            self.racket.x += self.racket.direction * RACKET_SPEED * duration.as_secs_f32();

            if self.racket.x <= 0.0 {
                self.racket.x = 0.0;
            }

            if self.racket.x >= WINDOW_WIDTH - RACKET_WIDTH - 0.0 {
                self.racket.x = WINDOW_WIDTH - RACKET_WIDTH - 0.0;
            }

            let collision_result = check_collision_recs(
                Rectangle {
                    x: self.ball.x,
                    y: self.ball.y,
                    width: PROJ_RADIUS,
                    height: PROJ_RADIUS,
                },
                Rectangle {
                    x: self.racket.x,
                    y: RACKET_POS_Y,
                    width: RACKET_WIDTH,
                    height: RACKET_HEIGHT,
                },
            );

            self.ball.already_in_collision = if collision_result {
                if !self.ball.already_in_collision {
                    self.ball.speed += 2.0;
                    self.ball.direction.y *= -1.0;
                }
                true
            } else {
                false
            };

            for brick in self.bricks.iter_mut() {
                let coll = get_collision_recs(
                    Rectangle {
                        x: self.ball.x,
                        y: self.ball.y,
                        width: PROJ_RADIUS,
                        height: PROJ_RADIUS,
                    },
                    Rectangle {
                        x: brick.x,
                        y: brick.y,
                        width: BRICK_WIDTH,
                        height: BRICK_HEIGHT,
                    },
                );

                if coll.width * coll.height > 0.0 {
                    brick.live -= 1;
                    self.ball.speed += 4.0;
                    if coll.width > coll.height {
                        self.ball.direction.y *= -1.0;
                    } else if coll.width < coll.height {
                        self.ball.direction.x *= -1.0;
                    } else {
                        self.ball.direction.y *= -1.0;
                        self.ball.direction.x *= -1.0;
                    }
                    break;
                }
            }

            self.bricks.retain(|b| b.live > 0);

            if self.bricks.is_empty() {
                self.state = ST::Winning;
            }

            self.ball.x +=
                self.ball.direction.x * self.ball.speed / 2.0f32.sqrt() * duration.as_secs_f32();
            self.ball.y +=
                self.ball.direction.y * self.ball.speed / 2.0f32.sqrt() * duration.as_secs_f32();
        }
    }

    fn render(&self, mut d: RaylibDrawHandle) {
        d.clear_background(Color::BLACK);
        d.draw_circle(
            self.ball.x as i32,
            self.ball.y as i32,
            PROJ_RADIUS,
            Color::WHITE,
        );

        d.draw_rectangle_gradient_v(
            self.racket.x as i32,
            RACKET_POS_Y as i32,
            RACKET_WIDTH as i32,
            RACKET_HEIGHT as i32,
            Color::RED,
            Color::new(80, 0, 0, 255),
        );

        for brick in self.bricks.iter() {
            d.draw_rectangle_gradient_v(
                brick.x as i32,
                brick.y as i32,
                BRICK_WIDTH as i32,
                BRICK_HEIGHT as i32,
                HI_COLOR[brick.live],
                LO_COLOR[brick.live],
            );
        }

        for i in 0..self.lives {
            d.draw_circle(
                (5.0 + PROJ_RADIUS + (i as f32) * (PROJ_RADIUS * 2.0 + 5.0)) as i32,
                (5.0 + PROJ_RADIUS) as i32,
                PROJ_RADIUS,
                Color::WHITE,
            );
        }

        match self.state {
            ST::Paused => draw_center_string(&mut d, "PAUSED"),
            ST::Winning => draw_center_string(&mut d, "YOU WON"),
            ST::GameOver => draw_center_string(&mut d, "GAME OVER"),
            _ => (),
        }
    }
}

enum State {
    Running,
    InitialBreak(Instant),
    Paused,
    Winning,
    GameOver,
}

use State as ST;

fn draw_center_string(d: &mut RaylibDrawHandle, s: &str) {
    let width = d.measure_text(s, 50);
    d.draw_text(
        s,
        (WINDOW_WIDTH / 2.0) as i32 - width / 2,
        (WINDOW_HEIGHT / 2.0) as i32 - 50 / 2,
        50,
        Color::YELLOW,
    );
}
fn main() {
    let mut game = Game::new();

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .title("Pong")
        .build();

    while !rl.window_should_close() {
        let duration = Instant::now().duration_since(game.last_frame_instant);
        if duration > Duration::from_secs_f32(FRAME_DURATION) {
            game.handle_input(&rl);
            game.calculate_physics(&duration);
            let d = rl.begin_drawing(&thread);
            game.render(d);
            game.last_frame_instant = Instant::now();
        }
    }
}
