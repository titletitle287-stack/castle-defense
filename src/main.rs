use macroquad::prelude::*;

const BASE_H: f32 = 540.0;
const CASTLE_W: f32 = 120.0;
const CASTLE_H: f32 = 150.0;
const GROUND: f32 = 420.0;
const UI_TOP: f32 = 448.0;
const MAX_PARTS: usize = 500;

// ---------- рисование с масштабом и тряской ----------

#[derive(Clone, Copy)]
struct G {
    u: f32,
    ox: f32,
    oy: f32,
}

impl G {
    fn rect(&self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        draw_rectangle((x + self.ox) * self.u, (y + self.oy) * self.u, w * self.u, h * self.u, c);
    }
    fn circ(&self, x: f32, y: f32, r: f32, c: Color) {
        draw_circle((x + self.ox) * self.u, (y + self.oy) * self.u, r * self.u, c);
    }
    fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, th: f32, c: Color) {
        draw_line(
            (x1 + self.ox) * self.u,
            (y1 + self.oy) * self.u,
            (x2 + self.ox) * self.u,
            (y2 + self.oy) * self.u,
            th * self.u,
            c,
        );
    }
    fn tri(&self, a: (f32, f32), b: (f32, f32), c: (f32, f32), color: Color) {
        let u = self.u;
        draw_triangle(
            vec2((a.0 + self.ox) * u, (a.1 + self.oy) * u),
            vec2((b.0 + self.ox) * u, (b.1 + self.oy) * u),
            vec2((c.0 + self.ox) * u, (c.1 + self.oy) * u),
            color,
        );
    }
    fn text(&self, s: &str, x: f32, y: f32, size: f32, c: Color) {
        draw_text(s, (x + self.ox) * self.u, (y + self.oy) * self.u, size * self.u, c);
    }
    fn text_w(&self, s: &str, size: f32) -> f32 {
        measure_text(s, None, (size * self.u) as u16, 1.0).width / self.u
    }
    fn text_center(&self, s: &str, cx: f32, y: f32, size: f32, c: Color) {
        let w = self.text_w(s, size);
        self.text(s, cx - w / 2.0, y, size, c);
    }
}

fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::new(r, g, b, a)
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn with_a(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

// ---------- враги ----------

struct Kind {
    hp: f32,
    speed: f32,
    reward: u32,
    w: f32,
    h: f32,
    color: Color,
    flying: bool,
}

fn kind(k: usize) -> Kind {
    match k {
        0 => Kind { hp: 20.0, speed: 45.0, reward: 3, w: 22.0, h: 30.0, color: rgba(0.35, 0.75, 0.35, 1.0), flying: false },
        1 => Kind { hp: 12.0, speed: 95.0, reward: 4, w: 18.0, h: 24.0, color: rgba(0.95, 0.6, 0.15, 1.0), flying: false },
        2 => Kind { hp: 70.0, speed: 30.0, reward: 10, w: 36.0, h: 48.0, color: rgba(0.8, 0.25, 0.3, 1.0), flying: false },
        3 => Kind { hp: 10.0, speed: 70.0, reward: 5, w: 26.0, h: 16.0, color: rgba(0.6, 0.4, 0.85, 1.0), flying: true },
        _ => Kind { hp: 4000.0, speed: 18.0, reward: 150, w: 80.0, h: 100.0, color: rgba(0.45, 0.15, 0.55, 1.0), flying: false },
    }
}

struct Enemy {
    x: f32,
    hp: f32,
    max_hp: f32,
    kind: usize,
    hit: f32,
    lane: f32,
    yo: f32,
    fy: f32,
    ph: f32,
}

fn enemy_rect(e: &Enemy, t: f32) -> Rect {
    let kd = kind(e.kind);
    if kd.flying {
        let cy = e.fy + (t * 3.0 + e.ph).sin() * 10.0;
        Rect::new(e.x - kd.w / 2.0, cy - kd.h / 2.0, kd.w, kd.h)
    } else {
        Rect::new(e.x - kd.w / 2.0, GROUND + e.yo - kd.h, kd.w, kd.h)
    }
}

struct Arrow {
    p: Vec2,
    v: Vec2,
    dmg: f32,
    alive: bool,
    pierce: u32,
}

struct Particle {
    p: Vec2,
    v: Vec2,
    life: f32,
    max: f32,
    c: Color,
    size: f32,
    grav: f32,
}

struct FText {
    p: Vec2,
    val: u32,
    life: f32,
}

struct Meteor {
    p: Vec2,
    v: Vec2,
    alive: bool,
}

fn burst(parts: &mut Vec<Particle>, p: Vec2, n: usize, c: Color, speed: f32, life: f32) {
    for _ in 0..n {
        if parts.len() >= MAX_PARTS {
            return;
        }
        let a: f32 = rand::gen_range(0.0, 6.2832);
        let s: f32 = rand::gen_range(0.2, 1.0) * speed;
        let size: f32 = rand::gen_range(2.0, 4.0);
        parts.push(Particle {
            p,
            v: vec2(a.cos() * s, a.sin() * s - speed * 0.3),
            life,
            max: life,
            c,
            size,
            grav: 400.0,
        });
    }
}

fn bow_origin() -> Vec2 {
    vec2(72.0, GROUND - CASTLE_H - 22.0)
}

// ---------- игра ----------

struct Game {
    arrows: Vec<Arrow>,
    enemies: Vec<Enemy>,
    parts: Vec<Particle>,
    ftexts: Vec<FText>,
    meteors: Vec<Meteor>,
    gold: u32,
    wave: u32,
    to_spawn: u32,
    spawn_t: f32,
    break_t: f32,
    shoot_cd: f32,
    recoil: f32,
    t: f32,
    dmg_lvl: u32,
    rate_lvl: u32,
    multi: u32,
    kills: u32,
    lw: f32,
    aim: Vec2,
    shake: f32,
    banner_t: f32,
    ult_cd: f32,
}

impl Game {
    fn new() -> Self {
        Game {
            arrows: vec![],
            enemies: vec![],
            parts: vec![],
            ftexts: vec![],
            meteors: vec![],
            gold: 0,
            wave: 1,
            to_spawn: 80,
            spawn_t: 0.5,
            break_t: 3.0,
            shoot_cd: 0.0,
            recoil: 0.0,
            t: 0.0,
            dmg_lvl: 100,
            rate_lvl: 100,
            multi: 3,
            kills: 0,
            lw: 960.0,
            aim: vec2(0.97, -0.25).normalize(),
            shake: 0.0,
            banner_t: 2.5,
            ult_cd: 0.0,
        }
    }

    fn cost(&self, i: usize) -> u32 {
        match i {
            0 => 20 + 15 * self.dmg_lvl,
            1 => 25 + 20 * self.rate_lvl,
            2 => 150 * self.multi,
            _ => 0,
        }
    }

    fn damage(&self) -> f32 {
        10.0 + 5.0 * self.dmg_lvl as f32
    }

    fn cooldown(&self) -> f32 {
        (0.55 * 0.9f32.powi(self.rate_lvl as i32)).max(0.03)
    }

    fn button_rects(lw: f32) -> [Rect; 4] {
        let gap = 8.0;
        let w = ((lw - gap * 5.0) / 4.0).min(210.0);
        let total = w * 4.0 + gap * 3.0;
        let x0 = (lw - total) / 2.0;
        let mut r = [Rect::new(0.0, 0.0, 0.0, 0.0); 4];
        for i in 0..4 {
            r[i] = Rect::new(x0 + i as f32 * (w + gap), UI_TOP + 8.0, w, 76.0);
        }
        r
    }

    fn buy(&mut self, i: usize) {
        match i {
            0 => {
                let c = self.cost(0);
                if self.gold >= c {
                    self.gold -= c;
                    self.dmg_lvl += 1;
                }
            }
            1 => {
                let c = self.cost(1);
                if self.gold >= c {
                    self.gold -= c;
                    self.rate_lvl += 1;
                }
            }
            2 => {
                if self.multi < 9 {
                    let c = self.cost(2);
                    if self.gold >= c {
                        self.gold -= c;
                        self.multi += 1;
                    }
                }
            }
            _ => {
                if self.ult_cd <= 0.0 {
                    self.ult_cd = 20.0;
                    for _ in 0..16 {
                        let x: f32 = rand::gen_range(self.lw * 0.3, self.lw);
                        let y: f32 = rand::gen_range(-500.0, -40.0);
                        self.meteors.push(Meteor { p: vec2(x, y), v: vec2(-160.0, 560.0), alive: true });
                    }
                }
            }
        }
    }

    fn spawn_enemy(&mut self) {
        let r: f32 = rand::gen_range(0.0, 1.0);
        let k = if self.wave >= 2 && r < 0.02 {
            4
        } else if self.wave >= 2 && r < 0.2 {
            3
        } else if self.wave >= 3 && r < 0.32 {
            2
        } else if r < 0.55 {
            1
        } else {
            0
        };
        let kd = kind(k);
        let hp = kd.hp * (1.0 + 0.12 * (self.wave as f32 - 1.0));
        let x: f32 = self.lw + 30.0 + rand::gen_range(0.0, 160.0);
        let lane: f32 = if kd.flying { rand::gen_range(0.0, 90.0) } else { rand::gen_range(0.0, 150.0) };
        let yo: f32 = rand::gen_range(0.0, 14.0);
        let fy: f32 = rand::gen_range(120.0, 300.0);
        let ph: f32 = rand::gen_range(0.0, 6.28);
        self.enemies.push(Enemy { x, hp, max_hp: hp, kind: k, hit: 0.0, lane, yo, fy, ph });
        if k == 4 {
            self.shake = (self.shake + 0.3).min(0.8);
        }
    }

    fn update(&mut self, dt: f32, u: f32) {
        let (mx, my) = mouse_position();
        let (mx, my) = (mx / u, my / u);

        self.t += dt;
        self.ult_cd = (self.ult_cd - dt).max(0.0);
        self.recoil = (self.recoil - dt * 6.0).max(0.0);
        self.banner_t = (self.banner_t - dt).max(0.0);
        self.shake = (self.shake - dt * 1.5).max(0.0);

        // --- ввод ---
        if is_mouse_button_pressed(MouseButton::Left) {
            let rects = Game::button_rects(self.lw);
            for i in 0..4 {
                if rects[i].contains(vec2(mx, my)) {
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
        if is_key_pressed(KeyCode::Key4) {
            self.buy(3);
        }

        // --- прицел и стрельба ---
        let o = bow_origin();
        if my < UI_TOP - 5.0 && (mx > 1.0 || my > 1.0) {
            let mut d = vec2(mx, my) - o;
            if d.x < 20.0 {
                d.x = 20.0;
            }
            self.aim = d.normalize();
        }

        self.shoot_cd -= dt;
        let want = is_mouse_button_down(MouseButton::Left) && my < UI_TOP - 5.0;
        if want && self.shoot_cd <= 0.0 {
            let n = self.multi;
            let d = self.aim;
            let dmg = self.damage();
            for i in 0..n {
                let off = (i as f32 - (n as f32 - 1.0) / 2.0) * 0.09;
                let (sn, cs) = off.sin_cos();
                let dd = vec2(d.x * cs - d.y * sn, d.x * sn + d.y * cs);
                self.arrows.push(Arrow { p: o + dd * 22.0, v: dd * 720.0, dmg, alive: true, pierce: 3 });
            }
            self.shoot_cd = self.cooldown();
            self.recoil = 1.0;
            burst(&mut self.parts, o + d * 22.0, 3, rgba(1.0, 0.9, 0.5, 1.0), 90.0, 0.25);
        }

        // --- стрелы ---
        let t = self.t;
        for a in self.arrows.iter_mut() {
            a.v.y += 500.0 * dt;
            a.p += a.v * dt;
            if a.p.y > GROUND + 18.0 || a.p.x > self.lw + 200.0 || a.p.x < -50.0 || a.p.y < -300.0 {
                a.alive = false;
                continue;
            }
            for e in self.enemies.iter_mut() {
                if e.hp <= 0.0 {
                    continue;
                }
                let rc = enemy_rect(e, t);
                if rc.contains(a.p) {
                    e.hp -= a.dmg;
                    e.hit = 0.08;
                    let kd = kind(e.kind);
                    burst(&mut self.parts, a.p, 3, kd.color, 110.0, 0.35);
                    if e.hp > 0.0 {
                        a.alive = false;
                        break;
                    }
                    a.pierce -= 1;
                    if a.pierce == 0 {
                        a.alive = false;
                        break;
                    }
                }
            }
        }
        self.arrows.retain(|a| a.alive);

        // --- метеориты ---
        for m in self.meteors.iter_mut() {
            m.p += m.v * dt;
            if m.p.y >= GROUND + 6.0 {
                m.alive = false;
                for e in self.enemies.iter_mut() {
                    if (e.x - m.p.x).abs() < 80.0 {
                        e.hp -= 900.0;
                        e.hit = 0.1;
                    }
                }
                burst(&mut self.parts, m.p, 14, rgba(1.0, 0.55, 0.1, 1.0), 220.0, 0.6);
                burst(&mut self.parts, m.p, 8, rgba(1.0, 0.9, 0.4, 1.0), 140.0, 0.45);
                self.shake = (self.shake + 0.12).min(0.6);
            }
        }
        self.meteors.retain(|m| m.alive);

        // --- движение врагов (замок неуязвим) ---
        for e in self.enemies.iter_mut() {
            let kd = kind(e.kind);
            e.hit = (e.hit - dt).max(0.0);
            let stop_x = CASTLE_W + kd.w / 2.0 + e.lane;
            if e.x > stop_x {
                e.x -= kd.speed * dt;
            }
        }

        // --- смерть врагов ---
        let mut dead: Vec<(Vec2, usize)> = vec![];
        for e in &self.enemies {
            if e.hp <= 0.0 {
                let rc = enemy_rect(e, self.t);
                dead.push((vec2(rc.x + rc.w / 2.0, rc.y + rc.h / 2.0), e.kind));
            }
        }
        self.enemies.retain(|e| e.hp > 0.0);
        for (p, k) in dead {
            let kd = kind(k);
            self.gold += kd.reward;
            self.kills += 1;
            let n = if k == 4 { 40 } else { 6 };
            let sp = if k == 4 { 260.0 } else { 130.0 };
            burst(&mut self.parts, p, n, kd.color, sp, 0.6);
            burst(&mut self.parts, p, 2, rgba(1.0, 0.85, 0.2, 1.0), 120.0, 0.7);
            if self.ftexts.len() < 30 {
                self.ftexts.push(FText { p, val: kd.reward, life: 0.9 });
            }
            if k == 4 {
                self.shake = (self.shake + 0.5).min(0.8);
            }
        }

        // --- частицы и надписи ---
        for p in self.parts.iter_mut() {
            p.life -= dt;
            p.v.y += p.grav * dt;
            p.p += p.v * dt;
        }
        self.parts.retain(|p| p.life > 0.0);
        for f in self.ftexts.iter_mut() {
            f.life -= dt;
            f.p.y -= 30.0 * dt;
        }
        self.ftexts.retain(|f| f.life > 0.0);

        // --- волны ---
        if self.to_spawn > 0 {
            self.spawn_t -= dt;
            if self.spawn_t <= 0.0 {
                self.spawn_enemy();
                self.to_spawn -= 1;
                self.spawn_t = rand::gen_range(0.04, 0.2) / (1.0 + 0.05 * self.wave as f32);
            }
        } else if self.enemies.is_empty() {
            self.break_t -= dt;
            if self.break_t <= 0.0 {
                self.wave += 1;
                self.to_spawn = 80 + self.wave * 40;
                self.break_t = 3.0;
                self.gold += 10;
                self.banner_t = 2.5;
            }
        }
    }

    // ---------- отрисовка ----------

    fn draw_sky(&self, g: &G) {
        let top = rgba(0.03, 0.04, 0.16, 1.0);
        let bot = rgba(0.45, 0.22, 0.42, 1.0);
        let n = 28;
        let hh = GROUND / n as f32;
        for i in 0..n {
            let t = i as f32 / (n as f32 - 1.0);
            g.rect(0.0, i as f32 * hh, self.lw + 2.0, hh + 1.0, mix(top, bot, t * t));
        }

        // звёзды
        for i in 0..60 {
            let x = (i as f32 * 97.3) % self.lw;
            let y = (i as f32 * 53.7) % (GROUND * 0.6);
            let a = 0.4 + 0.5 * (self.t * 2.0 + i as f32).sin().abs();
            g.rect(x, y, 2.0, 2.0, rgba(1.0, 1.0, 1.0, a));
        }

        // луна
        let mx = self.lw * 0.82;
        g.circ(mx, 90.0, 52.0, rgba(1.0, 0.95, 0.8, 0.06));
        g.circ(mx, 90.0, 40.0, rgba(1.0, 0.95, 0.8, 0.10));
        g.circ(mx, 90.0, 28.0, rgba(0.97, 0.96, 0.85, 1.0));
        g.circ(mx - 8.0, 84.0, 6.0, rgba(0.85, 0.84, 0.74, 1.0));
        g.circ(mx + 9.0, 98.0, 4.0, rgba(0.85, 0.84, 0.74, 1.0));
        g.circ(mx + 6.0, 80.0, 3.0, rgba(0.85, 0.84, 0.74, 1.0));

        // облака
        for i in 0..5 {
            let x = ((i as f32 * 260.0 + self.t * (6.0 + i as f32 * 2.0)) % (self.lw + 300.0)) - 150.0;
            let y = 40.0 + i as f32 * 28.0;
            let c = rgba(0.8, 0.7, 0.9, 0.12);
            g.circ(x, y, 18.0, c);
            g.circ(x + 20.0, y - 6.0, 22.0, c);
            g.circ(x + 42.0, y, 18.0, c);
            g.rect(x, y, 42.0, 18.0, c);
        }

        // дальние горы
        let count = (self.lw / 120.0) as i32 + 3;
        for i in 0..count {
            let x = i as f32 * 120.0 - 60.0;
            let h = 70.0 + ((i as f32 * 1.7).sin() * 0.5 + 0.5) * 70.0;
            g.tri((x, GROUND), (x + 60.0, GROUND - h), (x + 120.0, GROUND), rgba(0.16, 0.12, 0.3, 1.0));
        }
        // ближние холмы
        let count2 = (self.lw / 90.0) as i32 + 3;
        for i in 0..count2 {
            let x = i as f32 * 90.0 - 30.0;
            let h = 35.0 + ((i as f32 * 2.3 + 1.0).sin() * 0.5 + 0.5) * 45.0;
            g.tri((x, GROUND), (x + 45.0, GROUND - h), (x + 90.0, GROUND), rgba(0.10, 0.14, 0.24, 1.0));
        }
    }

    fn draw_ground(&self, g: &G) {
        let w = self.lw + 40.0;
        g.rect(-20.0, GROUND - 2.0, w, 20.0, rgba(0.14, 0.32, 0.15, 1.0));
        g.rect(-20.0, GROUND + 16.0, w, UI_TOP - GROUND - 16.0 + 6.0, rgba(0.22, 0.14, 0.1, 1.0));
        g.rect(-20.0, GROUND - 2.0, w, 3.0, rgba(0.25, 0.5, 0.22, 1.0));
        let n = (self.lw / 12.0) as i32 + 3;
        for i in 0..n {
            let x = i as f32 * 12.0;
            let h = 4.0 + ((i * 7) % 5) as f32;
            g.tri((x, GROUND - 2.0), (x + 4.0, GROUND - 2.0 - h), (x + 8.0, GROUND - 2.0), rgba(0.2, 0.45, 0.2, 1.0));
        }
    }

    fn draw_castle(&self, g: &G) {
        let cy = GROUND - CASTLE_H;
        let stone = rgba(0.42, 0.43, 0.5, 1.0);
        let brick = rgba(0.3, 0.31, 0.38, 1.0);
        let wood = rgba(0.3, 0.17, 0.08, 1.0);

        g.rect(0.0, cy, CASTLE_W, CASTLE_H, stone);
        g.rect(CASTLE_W - 12.0, cy, 12.0, CASTLE_H, rgba(0.32, 0.33, 0.4, 1.0));
        let rows = (CASTLE_H / 15.0) as i32;
        for row in 0..rows {
            let y = cy + row as f32 * 15.0;
            g.rect(0.0, y, CASTLE_W, 1.5, brick);
            let mut x = if row % 2 == 0 { 0.0 } else { 15.0 };
            while x < CASTLE_W {
                g.rect(x, y, 1.5, 15.0, brick);
                x += 30.0;
            }
        }
        // зубцы
        for i in 0..5 {
            let x = i as f32 * 26.0;
            g.rect(x, cy - 16.0, 16.0, 16.0, rgba(0.48, 0.49, 0.56, 1.0));
            g.rect(x + 12.0, cy - 16.0, 4.0, 16.0, rgba(0.36, 0.37, 0.44, 1.0));
        }
        // ворота
        g.circ(CASTLE_W / 2.0, GROUND - 50.0, 18.0, wood);
        g.rect(CASTLE_W / 2.0 - 18.0, GROUND - 50.0, 36.0, 50.0, wood);
        g.rect(CASTLE_W / 2.0 - 1.0, GROUND - 68.0, 2.0, 68.0, rgba(0.18, 0.1, 0.05, 1.0));
        for k in 0..4 {
            g.rect(CASTLE_W / 2.0 - 18.0, GROUND - 14.0 - k as f32 * 14.0, 36.0, 2.0, rgba(0.18, 0.1, 0.05, 1.0));
        }
        // окна
        let glow = 0.75 + 0.25 * (self.t * 7.0).sin();
        g.rect(20.0, cy + 30.0, 8.0, 18.0, rgba(1.0, 0.8, 0.3, glow));
        g.rect(92.0, cy + 30.0, 8.0, 18.0, rgba(1.0, 0.8, 0.3, glow));
        // факелы
        for tx in [30.0f32, 90.0f32] {
            let ty = GROUND - 62.0;
            g.rect(tx - 1.5, ty, 3.0, 14.0, rgba(0.35, 0.2, 0.1, 1.0));
            let fl = (self.t * 12.0 + tx).sin();
            g.circ(tx, ty - 2.0, 18.0, rgba(1.0, 0.6, 0.1, 0.12));
            g.circ(tx, ty - 2.0, 5.0 + fl * 1.2, rgba(1.0, 0.55, 0.1, 1.0));
            g.circ(tx, ty - 4.0, 3.0, rgba(1.0, 0.9, 0.3, 1.0));
        }
        // флаг
        let fx = CASTLE_W - 22.0;
        g.rect(fx, cy - 58.0, 3.0, 42.0, rgba(0.35, 0.2, 0.1, 1.0));
        let wv = (self.t * 4.0).sin();
        g.tri(
            (fx + 3.0, cy - 58.0),
            (fx + 3.0, cy - 40.0),
            (fx + 30.0 + wv * 3.0, cy - 49.0 + wv * 2.0),
            rgba(0.85, 0.12, 0.18, 1.0),
        );
    }

    fn draw_hero(&self, g: &G) {
        let hx = 70.0;
        let feet = GROUND - CASTLE_H;
        let wave = (self.t * 5.0).sin() * 3.0;
        // плащ
        g.tri((hx - 5.0, feet - 28.0), (hx - 5.0, feet - 6.0), (hx - 18.0 + wave, feet - 8.0), rgba(0.75, 0.1, 0.15, 1.0));
        // ноги
        g.rect(hx - 6.0, feet - 8.0, 5.0, 8.0, rgba(0.2, 0.2, 0.3, 1.0));
        g.rect(hx + 1.0, feet - 8.0, 5.0, 8.0, rgba(0.2, 0.2, 0.3, 1.0));
        // торс и пояс
        g.rect(hx - 7.0, feet - 26.0, 14.0, 18.0, rgba(0.2, 0.42, 0.9, 1.0));
        g.rect(hx - 7.0, feet - 14.0, 14.0, 3.0, rgba(0.55, 0.35, 0.15, 1.0));
        // голова и шлем
        g.rect(hx - 6.0, feet - 38.0, 12.0, 12.0, rgba(0.96, 0.8, 0.65, 1.0));
        g.rect(hx - 7.0, feet - 42.0, 14.0, 7.0, rgba(0.7, 0.72, 0.8, 1.0));
        g.rect(hx - 2.0, feet - 49.0, 4.0, 8.0, rgba(0.85, 0.15, 0.2, 1.0));
        g.rect(hx + 1.0, feet - 34.0, 3.0, 3.0, BLACK);

        // лук
        let aim = self.aim;
        let pivot = vec2(hx + 2.0, feet - 20.0);
        let bc = pivot + aim * (12.0 - self.recoil * 3.0);
        let perp = vec2(-aim.y, aim.x);
        let top = bc + perp * 17.0 - aim * 5.0;
        let bot = bc - perp * 17.0 - aim * 5.0;
        let mid = bc + aim * 5.0;
        let wood = rgba(0.6, 0.38, 0.16, 1.0);
        g.line(top.x, top.y, mid.x, mid.y, 3.0, wood);
        g.line(mid.x, mid.y, bot.x, bot.y, 3.0, wood);
        let ready = 1.0 - (self.shoot_cd / self.cooldown()).clamp(0.0, 1.0);
        let pull = 2.0 + 7.0 * ready;
        let sp = bc - aim * pull;
        let sc = rgba(1.0, 1.0, 1.0, 0.8);
        g.line(top.x, top.y, sp.x, sp.y, 1.2, sc);
        g.line(sp.x, sp.y, bot.x, bot.y, 1.2, sc);
        let tip = sp + aim * 22.0;
        g.line(sp.x, sp.y, tip.x, tip.y, 2.0, rgba(0.9, 0.8, 0.55, 1.0));
        g.line(pivot.x, pivot.y, bc.x, bc.y, 4.0, rgba(0.96, 0.8, 0.65, 1.0));
    }

    fn draw_enemy(&self, g: &G, e: &Enemy) {
        let kd = kind(e.kind);
        let flash = e.hit > 0.0;
        let body = if flash { WHITE } else { kd.color };
        let dark = if flash { WHITE } else { mix(kd.color, BLACK, 0.45) };
        let light = if flash { WHITE } else { mix(kd.color, WHITE, 0.3) };
        let red = rgba(1.0, 0.15, 0.15, 1.0);

        if kd.flying {
            let cy = e.fy + (self.t * 3.0 + e.ph).sin() * 10.0;
            let flap = (self.t * 14.0 + e.ph).sin() * 10.0;
            let x = e.x;
            g.rect(x - 8.0, GROUND + 6.0, 16.0, 3.0, rgba(0.0, 0.0, 0.0, 0.18));
            g.tri((x - 3.0, cy - 2.0), (x - 26.0, cy - 10.0 + flap), (x - 18.0, cy + 8.0), dark);
            g.tri((x + 3.0, cy - 2.0), (x + 26.0, cy - 10.0 + flap), (x + 18.0, cy + 8.0), dark);
            g.circ(x, cy, 8.0, body);
            g.tri((x - 6.0, cy - 5.0), (x - 4.0, cy - 13.0), (x - 1.0, cy - 6.0), dark);
            g.tri((x + 6.0, cy - 5.0), (x + 4.0, cy - 13.0), (x + 1.0, cy - 6.0), dark);
            g.circ(x - 3.0, cy - 1.0, 1.8, red);
            g.circ(x + 3.0, cy - 1.0, 1.8, red);
            if e.hp < e.max_hp {
                g.rect(x - 12.0, cy - 20.0, 24.0, 3.0, rgba(0.4, 0.05, 0.05, 0.9));
                g.rect(x - 12.0, cy - 20.0, 24.0 * (e.hp / e.max_hp).max(0.0), 3.0, rgba(0.2, 0.9, 0.3, 1.0));
            }
            return;
        }

        let base = GROUND + e.yo;
        let ph = self.t * 9.0 + e.ph;
        let bounce = (ph * 0.5).sin().abs() * 2.0;
        let by = base - kd.h - bounce;
        let bx = e.x - kd.w / 2.0;

        // аура босса
        if e.kind == 4 {
            let pulse = 0.10 + 0.05 * (self.t * 3.0).sin();
            g.circ(e.x, by + kd.h * 0.5, kd.w * 0.9, rgba(0.7, 0.2, 0.9, pulse));
        }

        // тень
        g.rect(bx - 2.0, base - 2.0, kd.w + 4.0, 4.0, rgba(0.0, 0.0, 0.0, 0.3));

        // ноги
        let l1 = ph.sin().max(0.0) * 3.0;
        let l2 = (-ph.sin()).max(0.0) * 3.0;
        g.rect(bx + kd.w * 0.15, base - 8.0 - l1, kd.w * 0.3, 8.0, dark);
        g.rect(bx + kd.w * 0.55, base - 8.0 - l2, kd.w * 0.3, 8.0, dark);

        // особенности сзади
        if e.kind == 1 {
            let tw = (self.t * 8.0 + e.ph).sin() * 4.0;
            g.line(bx + kd.w, by + kd.h - 14.0, bx + kd.w + 12.0, by + kd.h - 24.0 + tw, 2.0, dark);
        }
        if e.kind == 2 {
            g.line(bx - 4.0, by + 16.0, bx - 16.0, by - 4.0, 5.0, rgba(0.4, 0.25, 0.1, 1.0));
            g.circ(bx - 17.0, by - 6.0, 6.0, rgba(0.35, 0.22, 0.1, 1.0));
        }

        // тело
        g.rect(bx, by, kd.w, kd.h - 8.0, body);
        g.rect(bx, by, kd.w, 4.0, light);
        g.rect(bx, by + kd.h - 16.0, kd.w, 3.0, dark);

        // детали по типам
        match e.kind {
            0 => {
                g.tri((bx, by + 3.0), (bx - 9.0, by - 1.0), (bx, by + 11.0), body);
                g.tri((bx + kd.w, by + 3.0), (bx + kd.w + 9.0, by - 1.0), (bx + kd.w, by + 11.0), body);
                g.rect(bx + 4.0, by + kd.h * 0.5, 3.0, 3.0, WHITE);
                g.rect(bx + 10.0, by + kd.h * 0.5, 3.0, 3.0, WHITE);
            }
            1 => {
                let hc = rgba(1.0, 0.85, 0.3, 1.0);
                g.tri((bx + 2.0, by), (bx + 4.0, by - 9.0), (bx + 8.0, by), hc);
                g.tri((bx + kd.w - 8.0, by), (bx + kd.w - 4.0, by - 9.0), (bx + kd.w - 2.0, by), hc);
            }
            2 => {
                let sc = rgba(0.9, 0.9, 0.95, 1.0);
                g.tri((bx - 2.0, by + 2.0), (bx + 4.0, by - 10.0), (bx + 10.0, by + 2.0), sc);
                g.tri((bx + kd.w - 10.0, by + 2.0), (bx + kd.w - 4.0, by - 10.0), (bx + kd.w + 2.0, by + 2.0), sc);
            }
            _ => {
                let hc = rgba(0.92, 0.9, 0.8, 1.0);
                g.tri((bx - 4.0, by + 6.0), (bx - 16.0, by - 22.0), (bx + 14.0, by + 2.0), hc);
                g.tri((bx + kd.w + 4.0, by + 6.0), (bx + kd.w + 16.0, by - 22.0), (bx + kd.w - 14.0, by + 2.0), hc);
                let gold = rgba(1.0, 0.82, 0.2, 1.0);
                g.rect(bx + 14.0, by - 6.0, kd.w - 28.0, 8.0, gold);
                g.tri((bx + 14.0, by - 6.0), (bx + 20.0, by - 18.0), (bx + 26.0, by - 6.0), gold);
                g.tri((bx + kd.w * 0.5 - 6.0, by - 6.0), (bx + kd.w * 0.5, by - 20.0), (bx + kd.w * 0.5 + 6.0, by - 6.0), gold);
                g.tri((bx + kd.w - 26.0, by - 6.0), (bx + kd.w - 20.0, by - 18.0), (bx + kd.w - 14.0, by - 6.0), gold);
            }
        }

        // глаза
        let es = (kd.w * 0.22).max(5.0).min(10.0);
        let ey = by + kd.h * 0.2;
        let ec = if e.kind == 4 { rgba(1.0, 0.95, 0.2, 1.0) } else { WHITE };
        g.rect(bx + kd.w * 0.12, ey, es, es, ec);
        g.rect(bx + kd.w * 0.5, ey, es, es, ec);
        g.rect(bx + kd.w * 0.12, ey + 1.0, es * 0.5, es - 2.0, red);
        g.rect(bx + kd.w * 0.5, ey + 1.0, es * 0.5, es - 2.0, red);

        // полоска здоровья
        if e.hp < e.max_hp {
            let hh = if e.kind == 4 { 6.0 } else { 4.0 };
            let top = if e.kind == 4 { by - 28.0 } else { by - 10.0 };
            g.rect(bx, top, kd.w, hh, rgba(0.4, 0.05, 0.05, 0.9));
            g.rect(bx, top, kd.w * (e.hp / e.max_hp).max(0.0), hh, rgba(0.2, 0.9, 0.3, 1.0));
        }
    }

    fn draw(&self, u: f32) {
        let g0 = G { u, ox: 0.0, oy: 0.0 };
        let s = self.shake * 10.0;
        let sx: f32 = rand::gen_range(-1.0, 1.0);
        let sy: f32 = rand::gen_range(-1.0, 1.0);
        let gs = G { u, ox: sx * s, oy: sy * s };

        clear_background(rgba(0.12, 0.08, 0.1, 1.0));

        // фон
        self.draw_sky(&g0);
        self.draw_ground(&gs);
        self.draw_castle(&gs);

        // враги
        for e in &self.enemies {
            self.draw_enemy(&gs, e);
        }

        self.draw_hero(&gs);

        // стрелы
        for a in &self.arrows {
            let d = a.v.normalize();
            let tail = a.p - d * 20.0;
            gs.line(tail.x, tail.y, a.p.x, a.p.y, 5.0, rgba(1.0, 0.7, 0.2, 0.25));
            gs.line(tail.x, tail.y, a.p.x, a.p.y, 2.0, rgba(1.0, 0.95, 0.7, 1.0));
            gs.circ(a.p.x, a.p.y, 2.2, WHITE);
        }

        // метеориты
        for m in &self.meteors {
            let d = m.v.normalize();
            let tail = m.p - d * 50.0;
            gs.line(tail.x, tail.y, m.p.x, m.p.y, 10.0, rgba(1.0, 0.4, 0.1, 0.3));
            gs.circ(m.p.x, m.p.y, 14.0, rgba(1.0, 0.5, 0.1, 0.25));
            gs.circ(m.p.x, m.p.y, 7.0, rgba(1.0, 0.5, 0.1, 1.0));
            gs.circ(m.p.x, m.p.y, 4.0, rgba(1.0, 0.95, 0.5, 1.0));
        }

        // частицы
        for p in &self.parts {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            let sz = p.size * (0.5 + a * 0.5);
            gs.rect(p.p.x, p.p.y, sz, sz, with_a(p.c, a));
        }

        // всплывающие числа
        for f in &self.ftexts {
            let a = (f.life / 0.9).min(1.0);
            gs.text(&format!("+{}", f.val), f.p.x, f.p.y, 20.0, rgba(1.0, 0.9, 0.3, a));
        }

        // ---------- интерфейс ----------
        g0.rect(8.0, 8.0, 200.0, 98.0, rgba(0.0, 0.0, 0.0, 0.45));
        g0.circ(24.0, 26.0, 8.0, rgba(1.0, 0.82, 0.2, 1.0));
        g0.circ(24.0, 26.0, 5.0, rgba(0.9, 0.65, 0.1, 1.0));
        g0.text(&format!("{}", self.gold), 40.0, 34.0, 28.0, rgba(1.0, 0.9, 0.4, 1.0));
        g0.text(&format!("Wave {}", self.wave), 16.0, 58.0, 24.0, WHITE);
        g0.text(&format!("Kills {}", self.kills), 16.0, 78.0, 22.0, LIGHTGRAY);
        g0.text("Castle: invincible", 16.0, 98.0, 20.0, rgba(0.5, 0.9, 0.5, 1.0));

        // баннер волны
        if self.banner_t > 0.0 {
            let a = (self.banner_t / 1.0).min(1.0);
            let txt = format!("WAVE {}", self.wave);
            g0.text_center(&txt, self.lw / 2.0 + 3.0, 183.0, 70.0, rgba(0.0, 0.0, 0.0, a * 0.6));
            g0.text_center(&txt, self.lw / 2.0, 180.0, 70.0, rgba(1.0, 0.9, 0.5, a));
        }
        if self.enemies.iter().any(|e| e.kind == 4) {
            let pulse = 0.7 + 0.3 * (self.t * 6.0).sin();
            g0.text_center("BOSS!", self.lw / 2.0, 40.0, 40.0, rgba(1.0, 0.25, 0.25, pulse));
        }
        if self.to_spawn == 0 && self.enemies.is_empty() {
            g0.text_center("Wave cleared!", self.lw / 2.0, 150.0, 30.0, rgba(0.6, 1.0, 0.6, 1.0));
        }

        // панель кнопок
        g0.rect(0.0, UI_TOP, self.lw, BASE_H - UI_TOP, rgba(0.08, 0.09, 0.17, 1.0));
        g0.rect(0.0, UI_TOP, self.lw, 2.0, rgba(0.3, 0.32, 0.55, 1.0));

        let rects = Game::button_rects(self.lw);
        let labels = ["[1] Damage", "[2] Fire rate", "[3] Multishot", "[4] Meteor"];
        for (i, rc) in rects.iter().enumerate() {
            let (can, line1, line2) = match i {
                0 => (self.gold >= self.cost(0), format!("{} Lv{}", labels[0], self.dmg_lvl), format!("Cost {}", self.cost(0))),
                1 => (self.gold >= self.cost(1), format!("{} Lv{}", labels[1], self.rate_lvl), format!("Cost {}", self.cost(1))),
                2 => {
                    if self.multi >= 9 {
                        (false, format!("{} x{}", labels[2], self.multi), "MAX".to_string())
                    } else {
                        (self.gold >= self.cost(2), format!("{} x{}", labels[2], self.multi), format!("Cost {}", self.cost(2)))
                    }
                }
                _ => {
                    if self.ult_cd <= 0.0 {
                        (true, labels[3].to_string(), "READY".to_string())
                    } else {
                        (false, labels[3].to_string(), format!("{:.0}s", self.ult_cd.ceil()))
                    }
                }
            };
            let bg = if can { rgba(0.14, 0.3, 0.2, 1.0) } else { rgba(0.13, 0.14, 0.26, 1.0) };
            let border = if can {
                if i == 3 { rgba(1.0, 0.6, 0.15, 1.0) } else { rgba(0.3, 0.9, 0.4, 1.0) }
            } else {
                rgba(0.4, 0.42, 0.55, 1.0)
            };
            g0.rect(rc.x, rc.y, rc.w, rc.h, bg);
            g0.rect(rc.x, rc.y, rc.w, 4.0, with_a(border, 0.5));
            draw_rectangle_lines(rc.x * u, rc.y * u, rc.w * u, rc.h * u, 2.0 * u, border);
            g0.text(&line1, rc.x + 10.0, rc.y + 32.0, 22.0, WHITE);
            let c2 = if can { rgba(1.0, 0.88, 0.35, 1.0) } else { GRAY };
            g0.text(&line2, rc.x + 10.0, rc.y + 62.0, 24.0, c2);
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
