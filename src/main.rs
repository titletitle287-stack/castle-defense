use macroquad::prelude::*;

const BASE_H: f32 = 540.0; // логическая высота экрана
const CASTLE_W: f32 = 110.0;
const CASTLE_H: f32 = 140.0;
const GROUND: f32 = 420.0;
const UI_TOP: f32 = 440.0;

struct Kind {
    hp: f32,
    speed: f32,
    dmg: f32,
    reward: u32,
    w: f32,
    h: f32,
    color: Color,
}

fn kind(k: usize) -> Kind {
    match k {
        0 => Kind { hp: 20.0, speed: 45.0, dmg: 4.0, reward: 3, w: 22.0, h: 30.0, color: Color::new(0.3, 0.7, 0.3, 1.0) },
        1 => Kind { hp: 12.0, speed: 90.0, dmg: 2.0, reward: 4, w: 18.0, h: 24.0, color: Color::new(0.9, 0.8, 0.2, 1.0) },
        _ => Kind { hp: 70.0, speed: 30.0, dmg: 9.0, reward: 10, w: 34.0, h: 46.0, color: Color::new(0.8, 0.2, 0.25, 1.0) },
    }
}

struct Arrow {
    p: Vec2,
    v: Vec2,
    dmg: f32,
    alive: bool,
}

struct Enemy {
    x: f32,
    hp: f32,
    max_hp: f32,
    kind: usize,
    cd: f32,
    hit: f32,
}

struct Particle {
    p: Vec2,
    v: Vec2,
    life: f32,
    c: Color,
}

struct Game {
    arrows: Vec<Arrow>,
    enemies: Vec<Enemy>,
    parts: Vec<Particle>,
    castle_hp: f32,
    castle_max: f32,
    gold: u32,
    wave: u32,
    to_spawn: u32,
    spawn_t: f32,
    break_t: f32,
    shoot_cd: f32,
    t: f32,
    dmg_lvl: u32,
    rate_lvl: u32,
    kills: u32,
    over: bool,
    lw: f32,
}

impl Game {
    fn new() -> Self {
        Game {
            arrows: vec![],
            enemies: vec![],
            parts: vec![],
            castle_hp: 100.0,
            castle_max: 100.0,
            gold: 0,
            wave: 1,
            to_spawn: 5,
            spawn_t: 1.0,
            break_t: 3.0,
            shoot_cd: 0.0,
            t: 0.0,
            dmg_lvl: 0,
            rate_lvl: 0,
            kills: 0,
            over: false,
            lw: 960.0,
        }
    }

    fn cost(&self, i: usize) -> u32 {
        match i {
            0 => 20 + 15 * self.dmg_lvl,
            1 => 25 + 20 * self.rate_lvl,
            _ => 30,
        }
    }

    fn buy(&mut self, i: usize) {
        let c = self.cost(i);
        if self.gold < c {
            return;
        }
        if i == 2 && self.castle_hp >= self.castle_max {
            return;
        }
        self.gold -= c;
        match i {
            0 => self.dmg_lvl += 1,
            1 => self.rate_lvl += 1,
            _ => self.castle_hp = (self.castle_hp + 50.0).min(self.castle_max),
        }
    }

    fn button_rects() -> [Rect; 3] {
        let mut r = [Rect::new(0.0, 0.0, 0.0, 0.0); 3];
        for i in 0..3 {
            r[i] = Rect::new(12.0 + i as f32 * 212.0, UI_TOP + 8.0, 200.0, 80.0);
        }
        r
    }

    fn damage(&self) -> f32 {
        10.0 + 5.0 * self.dmg_lvl as f32
    }

    fn cooldown(&self) -> f32 {
        (0.55 * 0.9f32.powi(self.rate_lvl as i32)).max(0.12)
    }

    fn spawn_enemy(&mut self) {
        let r = rand::gen_range(0.0, 1.0);
        let k = if self.wave >= 4 && r < 0.15 {
            2
        } else if self.wave >= 2 && r < 0.45 {
            1
        } else {
            0
        };
        let kd = kind(k);
        let hp = kd.hp * (1.0 + 0.12 * (self.wave as f32 - 1.0));
        self.enemies.push(Enemy { x: self.lw + 30.0, hp, max_hp: hp, kind: k, cd: 0.5, hit: 0.0 });
    }

    fn update(&mut self, dt: f32, u: f32) {
        let (mx, my) = mouse_position();
        let (mx, my) = (mx / u, my / u);

        if self.over {
            if is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space) {
                *self = Game::new();
            }
            return;
        }

        self.t += dt;

        // --- ввод ---
        if is_mouse_button_pressed(MouseButton::Left) {
            for (i, r) in Game::button_rects().iter().enumerate() {
                if r.contains(vec2(mx, my)) {
                    self.buy(i);
                }
            }
        }
        if is_key_pressed(KeyCode::Key1) {
            self.buy(0);
        }
        if is_key_pressed(KeyCode::Key2) {
            self.buy(1);
        }
        if is_key_pressed(KeyCode::Key3) {
            self.buy(2);
        }

        // --- стрельба ---
        self.shoot_cd -= dt;
        let want_shoot = is_mouse_button_down(MouseButton::Left) && my < UI_TOP - 5.0;
        if want_shoot && self.shoot_cd <= 0.0 {
            let o = vec2(70.0, GROUND - CASTLE_H - 20.0);
            let mut d = vec2(mx, my) - o;
            if d.x < 20.0 {
                d.x = 20.0;
            }
            let d = d.normalize();
            self.arrows.push(Arrow { p: o, v: d * 650.0, dmg: self.damage(), alive: true });
            self.shoot_cd = self.cooldown();
        }

        // --- стрелы ---
        for a in self.arrows.iter_mut() {
            a.v.y += 500.0 * dt;
            a.p += a.v * dt;
            if a.p.y > GROUND || a.p.x > self.lw + 50.0 || a.p.x < -50.0 || a.p.y < -300.0 {
                a.alive = false;
                continue;
            }
            for e in self.enemies.iter_mut() {
                if e.hp <= 0.0 {
                    continue;
                }
                let kd = kind(e.kind);
                let rect = Rect::new(e.x - kd.w / 2.0, GROUND - kd.h, kd.w, kd.h);
                if rect.contains(a.p) {
                    e.hp -= a.dmg;
                    e.hit = 0.08;
                    a.alive = false;
                    for _ in 0..6 {
                        self.parts.push(Particle {
                            p: a.p,
                            v: vec2(rand::gen_range(-80.0, 80.0), rand::gen_range(-120.0, 0.0)),
                            life: 0.4,
                            c: Color::new(0.9, 0.9, 0.5, 1.0),
                        });
                    }
                    break;
                }
            }
        }
        self.arrows.retain(|a| a.alive);

        // --- враги ---
        for e in self.enemies.iter_mut() {
            let kd = kind(e.kind);
            e.hit = (e.hit - dt).max(0.0);
            let stop_x = CASTLE_W + kd.w / 2.0;
            if e.x > stop_x {
                e.x -= kd.speed * dt;
            } else {
                e.cd -= dt;
                if e.cd <= 0.0 {
                    self.castle_hp -= kd.dmg;
                    e.cd = 1.0;
                    for _ in 0..4 {
                        self.parts.push(Particle {
                            p: vec2(CASTLE_W, GROUND - rand::gen_range(10.0, 100.0)),
                            v: vec2(rand::gen_range(-60.0, 60.0), rand::gen_range(-100.0, -10.0)),
                            life: 0.5,
                            c: Color::new(0.6, 0.6, 0.65, 1.0),
                        });
                    }
                }
            }
        }

        // --- смерть врагов ---
        let mut dead: Vec<(f32, usize)> = vec![];
        for e in &self.enemies {
            if e.hp <= 0.0 {
                dead.push((e.x, e.kind));
            }
        }
        self.enemies.retain(|e| e.hp > 0.0);
        for (x, k) in dead {
            let kd = kind(k);
            self.gold += kd.reward;
            self.kills += 1;
            for _ in 0..12 {
                self.parts.push(Particle {
                    p: vec2(x, GROUND - kd.h / 2.0),
                    v: vec2(rand::gen_range(-120.0, 120.0), rand::gen_range(-200.0, -20.0)),
                    life: 0.6,
                    c: kd.color,
                });
            }
        }

        // --- частицы ---
        for p in self.parts.iter_mut() {
            p.life -= dt;
            p.v.y += 400.0 * dt;
            p.p += p.v * dt;
        }
        self.parts.retain(|p| p.life > 0.0);

        // --- волны ---
        if self.to_spawn > 0 {
            self.spawn_t -= dt;
            if self.spawn_t <= 0.0 {
                self.spawn_enemy();
                self.to_spawn -= 1;
                self.spawn_t = rand::gen_range(0.5, 1.4) / (1.0 + 0.05 * self.wave as f32);
            }
        } else if self.enemies.is_empty() {
            self.break_t -= dt;
            if self.break_t <= 0.0 {
                self.wave += 1;
                self.to_spawn = 4 + self.wave * 3;
                self.break_t = 3.0;
                self.gold += 10;
            }
        }

        if self.castle_hp <= 0.0 {
            self.castle_hp = 0.0;
            self.over = true;
        }
    }

    fn draw(&self, u: f32) {
        let r = |x: f32, y: f32, w: f32, h: f32, c: Color| draw_rectangle(x * u, y * u, w * u, h * u, c);
        let text = |s: &str, x: f32, y: f32, size: f32, c: Color| {
            draw_text(s, x * u, y * u, size * u, c);
        };

        clear_background(Color::new(0.07, 0.09, 0.2, 1.0));

        // звёзды и луна
        for i in 0..40 {
            let x = (i as f32 * 97.3) % self.lw;
            let y = (i as f32 * 53.7) % 300.0;
            r(x, y, 2.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.7));
        }
        draw_circle(self.lw * 0.8 * u, 80.0 * u, 28.0 * u, Color::new(0.95, 0.95, 0.8, 1.0));

        // земля и панель
        r(0.0, GROUND, self.lw, 14.0, Color::new(0.16, 0.3, 0.14, 1.0));
        r(0.0, GROUND + 14.0, self.lw, BASE_H, Color::new(0.1, 0.11, 0.2, 1.0));

        // замок
        let cy = GROUND - CASTLE_H;
        r(0.0, cy, CASTLE_W, CASTLE_H, Color::new(0.45, 0.45, 0.5, 1.0));
        for i in 0..4 {
            r(i as f32 * 30.0 + 2.0, cy - 14.0, 20.0, 14.0, Color::new(0.5, 0.5, 0.55, 1.0));
        }
        for row in 0..7 {
            r(0.0, cy + row as f32 * 20.0, CASTLE_W, 2.0, Color::new(0.38, 0.38, 0.43, 1.0));
        }
        r(CASTLE_W / 2.0 - 16.0, GROUND - 46.0, 32.0, 46.0, Color::new(0.25, 0.15, 0.08, 1.0));

        // полоска HP замка
        let frac = (self.castle_hp / self.castle_max).clamp(0.0, 1.0);
        r(10.0, cy - 40.0, CASTLE_W - 20.0, 8.0, Color::new(0.5, 0.1, 0.1, 1.0));
        r(10.0, cy - 40.0, (CASTLE_W - 20.0) * frac, 8.0, Color::new(0.2, 0.8, 0.3, 1.0));

        // лучник
        let hx = 62.0;
        let hy = cy;
        r(hx - 7.0, hy - 22.0, 14.0, 22.0, Color::new(0.2, 0.4, 0.9, 1.0));
        r(hx - 6.0, hy - 34.0, 12.0, 12.0, Color::new(0.95, 0.8, 0.65, 1.0));
        r(hx - 7.0, hy - 38.0, 14.0, 5.0, Color::new(0.8, 0.8, 0.85, 1.0));
        draw_line((hx + 10.0) * u, (hy - 34.0) * u, (hx + 10.0) * u, (hy - 6.0) * u, 3.0 * u, Color::new(0.55, 0.35, 0.15, 1.0));

        // враги
        for e in &self.enemies {
            let kd = kind(e.kind);
            let col = if e.hit > 0.0 { WHITE } else { kd.color };
            let x = e.x - kd.w / 2.0;
            r(x, GROUND - kd.h, kd.w, kd.h - 6.0, col);
            let wob = (self.t * 10.0 + e.x).sin() * 2.0;
            r(x + 3.0, GROUND - 6.0, 6.0, 6.0, Color::new(0.1, 0.1, 0.1, 1.0));
            r(x + kd.w - 9.0, GROUND - 6.0 - wob.abs() * 0.5, 6.0, 6.0, Color::new(0.1, 0.1, 0.1, 1.0));
            r(x + 3.0, GROUND - kd.h + 6.0, 6.0, 6.0, WHITE);
            r(x + 4.0, GROUND - kd.h + 8.0, 3.0, 3.0, BLACK);
            if e.hp < e.max_hp {
                r(x, GROUND - kd.h - 8.0, kd.w, 4.0, Color::new(0.5, 0.1, 0.1, 1.0));
                r(x, GROUND - kd.h - 8.0, kd.w * (e.hp / e.max_hp).max(0.0), 4.0, Color::new(0.2, 0.9, 0.3, 1.0));
            }
        }

        // стрелы
        for a in &self.arrows {
            let tail = a.p - a.v.normalize() * 14.0;
            draw_line(tail.x * u, tail.y * u, a.p.x * u, a.p.y * u, 2.0 * u, Color::new(0.85, 0.75, 0.5, 1.0));
        }

        // частицы
        for p in &self.parts {
            let mut c = p.c;
            c.a = (p.life * 2.0).min(1.0);
            r(p.p.x, p.p.y, 3.0, 3.0, c);
        }

        // HUD
        text(&format!("Gold: {}", self.gold), 12.0, 28.0, 28.0, Color::new(1.0, 0.85, 0.2, 1.0));
        text(&format!("Wave: {}", self.wave), 12.0, 56.0, 28.0, WHITE);
        text(&format!("Kills: {}", self.kills), 12.0, 84.0, 24.0, LIGHTGRAY);
        text(&format!("Castle: {:.0}/{:.0}", self.castle_hp, self.castle_max), 12.0, 110.0, 24.0, Color::new(0.5, 0.9, 0.5, 1.0));

        if self.to_spawn == 0 && self.enemies.is_empty() && !self.over {
            let s = "Wave cleared!";
            let w = measure_text(s, None, (30.0 * u) as u16, 1.0).width / u;
            text(s, self.lw / 2.0 - w / 2.0, 150.0, 30.0, Color::new(0.6, 1.0, 0.6, 1.0));
        }

        // кнопки улучшений
        let labels = ["[1] Damage", "[2] Fire rate", "[3] Repair +50"];
        let levels = [self.dmg_lvl, self.rate_lvl, 0];
        for (i, rc) in Game::button_rects().iter().enumerate() {
            let can = self.gold >= self.cost(i);
            let bg = if can { Color::new(0.18, 0.3, 0.2, 1.0) } else { Color::new(0.16, 0.17, 0.28, 1.0) };
            r(rc.x, rc.y, rc.w, rc.h, bg);
            draw_rectangle_lines(rc.x * u, rc.y * u, rc.w * u, rc.h * u, 2.0 * u, if can { GREEN } else { GRAY });
            let title = if i < 2 { format!("{} Lv{}", labels[i], levels[i]) } else { labels[i].to_string() };
            text(&title, rc.x + 10.0, rc.y + 32.0, 24.0, WHITE);
            text(&format!("Cost: {}", self.cost(i)), rc.x + 10.0, rc.y + 62.0, 24.0, if can { Color::new(1.0, 0.85, 0.2, 1.0) } else { GRAY });
        }

        // экран поражения
        if self.over {
            r(0.0, 0.0, self.lw, BASE_H, Color::new(0.0, 0.0, 0.0, 0.7));
            let lines = [
                ("GAME OVER".to_string(), 60.0),
                (format!("Wave {}  |  Kills {}", self.wave, self.kills), 30.0),
                ("Tap to restart".to_string(), 28.0),
            ];
            let mut y = 200.0;
            for (s, size) in lines.iter() {
                let w = measure_text(s, None, (*size * u) as u16, 1.0).width / u;
                text(s, self.lw / 2.0 - w / 2.0, y, *size, WHITE);
                y += size + 20.0;
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Castle Defense".to_owned(),
        window_width: 960,
        window_height: 540,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);
    let mut g = Game::new();
    loop {
        let dt = get_frame_time().min(0.05);
        let u = screen_height() / BASE_H;
        g.lw = screen_width() / u;
        g.update(dt, u);
        g.draw(u);
        next_frame().await;
    }
  }
