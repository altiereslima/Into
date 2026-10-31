//! Entrada portátil: teclado, mouse e gamepad convertidos em ações de jogo.
//!
//! Este módulo não requer SDL2 e pode ser testado sem qualquer janela.
//! O mapa de teclas/botões é PROVISÓRIO, não recuperado do jogo original.

use std::collections::{HashMap, HashSet};

pub const DEFAULT_REPEAT_DELAY_MS: u32 = 250;
pub const DEFAULT_REPEAT_INTERVAL_MS: u32 = 110;
pub const STICK_PRESS_THRESHOLD: i16 = 14_000;
pub const STICK_RELEASE_THRESHOLD: i16 = 9_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Key {
    Up, Down, Left, Right, W, A, S, D,
    Enter, Space, Escape, Backspace, Tab, E, Z, R,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PadButton {
    Up, Down, Left, Right, South, East, West, North,
    Start, Select, LeftShoulder, RightShoulder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PadAxis { LeftX, LeftY }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MouseButton { Left, Right, Middle }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceEvent {
    KeyDown { key: Key, repeat: bool },
    KeyUp { key: Key },
    MouseMoved { x: i32, y: i32 },
    MouseDown { button: MouseButton, x: i32, y: i32 },
    MouseUp { button: MouseButton, x: i32, y: i32 },
    MouseWheel { y: i32 },
    ControllerConnected { id: u32 },
    ControllerDisconnected { id: u32 },
    ControllerDown { id: u32, button: PadButton },
    ControllerUp { id: u32, button: PadButton },
    ControllerAxis { id: u32, axis: PadAxis, value: i16 },
    FocusLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tile { pub col: i32, pub row: i32 }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub columns: u32,
    pub rows: u32,
}

impl Viewport {
    /// Coordenadas do mouse -> tile; limites inferiores inclusivos, superiores exclusivos.
    /// Usa i64 para não transbordar ao operar com coordenadas de janela grandes.
    pub fn locate(&self, x: i32, y: i32) -> Option<Tile> {
        if self.width == 0 || self.height == 0 || self.columns == 0 || self.rows == 0 {
            return None;
        }
        let rx = i64::from(x) - i64::from(self.x);
        let ry = i64::from(y) - i64::from(self.y);
        if rx < 0 || ry < 0 || rx >= i64::from(self.width) || ry >= i64::from(self.height) {
            return None;
        }
        let col = rx * i64::from(self.columns) / i64::from(self.width);
        let row = ry * i64::from(self.rows) / i64::from(self.height);
        Some(Tile { col: col as i32, row: row as i32 })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameAction {
    CursorStep { dx: i8, dy: i8 },
    HoverTile(Option<Tile>),
    SelectTile(Tile),
    Confirm,
    Cancel,
    EndTurn,
    Undo,
    ResetTurn,
    ToggleMenu,
    Zoom(i8),
}

#[derive(Default)]
struct PadState {
    held: HashSet<PadButton>,
    axis_x: i8,
    axis_y: i8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Direction { x: i8, y: i8 }

/// Adaptador de eventos independente da plataforma. Um InputSystem por janela/jogador.
pub struct InputSystem {
    keys: HashSet<Key>,
    pads: HashMap<u32, PadState>,
    viewport: Viewport,
    hover: Option<Tile>,
    direction: Direction,
    held_ms: u32,
    repeat_delay_ms: u32,
    repeat_interval_ms: u32,
}

impl InputSystem {
    pub fn new(viewport: Viewport) -> Self {
        Self {
            keys: HashSet::new(), pads: HashMap::new(), viewport,
            hover: None, direction: Direction::default(), held_ms: 0,
            repeat_delay_ms: DEFAULT_REPEAT_DELAY_MS,
            repeat_interval_ms: DEFAULT_REPEAT_INTERVAL_MS,
        }
    }

    pub fn set_viewport(&mut self, viewport: Viewport) { self.viewport = viewport; }
    pub fn viewport(&self) -> Viewport { self.viewport }
    pub fn connected_controllers(&self) -> usize { self.pads.len() }

    pub fn set_repeat(&mut self, delay_ms: u32, interval_ms: u32) {
        self.repeat_delay_ms = delay_ms;
        self.repeat_interval_ms = interval_ms.max(1);
        self.held_ms = 0;
    }

    /// Processar evento, emitindo comandos imediatos e eventuais passos de cursor.
    pub fn handle(&mut self, event: DeviceEvent) -> Vec<GameAction> {
        let mut out = Vec::new();
        match event {
            DeviceEvent::KeyDown { key, repeat } => {
                if !repeat && self.keys.insert(key) {
                    if let Some(action) = key_action(key) { out.push(action); }
                }
            }
            DeviceEvent::KeyUp { key } => { self.keys.remove(&key); }
            DeviceEvent::MouseMoved { x, y } => { self.set_hover(x, y, &mut out); }
            DeviceEvent::MouseDown { button, x, y } => {
                self.set_hover(x, y, &mut out);
                match button {
                    MouseButton::Left => {
                        if let Some(tile) = self.viewport.locate(x, y) { out.push(GameAction::SelectTile(tile)); }
                    }
                    MouseButton::Right => out.push(GameAction::Cancel),
                    MouseButton::Middle => {},
                }
            }
            DeviceEvent::MouseUp { .. } => {}
            DeviceEvent::MouseWheel { y } => {
                if y != 0 { out.push(GameAction::Zoom(y.signum() as i8)); }
            }
            DeviceEvent::ControllerConnected { id } => { self.pads.entry(id).or_default(); }
            DeviceEvent::ControllerDisconnected { id } => { self.pads.remove(&id); }
            DeviceEvent::ControllerDown { id, button } => {
                let state = self.pads.entry(id).or_default();
                if state.held.insert(button) {
                    if let Some(action) = pad_action(button) { out.push(action); }
                }
            }
            DeviceEvent::ControllerUp { id, button } => {
                if let Some(state) = self.pads.get_mut(&id) { state.held.remove(&button); }
            }
            DeviceEvent::ControllerAxis { id, axis, value } => {
                let state = self.pads.entry(id).or_default();
                let previous = match axis { PadAxis::LeftX => state.axis_x, PadAxis::LeftY => state.axis_y };
                let direction = axis_direction(previous, value);
                match axis { PadAxis::LeftX => state.axis_x = direction, PadAxis::LeftY => state.axis_y = direction }
            }
            DeviceEvent::FocusLost => {
                self.keys.clear();
                for pad in self.pads.values_mut() {
                    pad.held.clear(); pad.axis_x = 0; pad.axis_y = 0;
                }
                self.hover = None;
                out.push(GameAction::HoverTile(None));
            }
        }
        self.refresh_direction(&mut out);
        out
    }

    /// Avançar relógio de repetição em milissegundos. Chamar a cada frame.
    /// Limite de 8 eventos por chamada protege contra picos após travamento/suspensão.
    pub fn tick(&mut self, delta_ms: u32) -> Vec<GameAction> {
        if self.direction == Direction::default() { return Vec::new(); }
        let previous = self.held_ms;
        let next = self.held_ms.saturating_add(delta_ms);
        self.held_ms = next;
        let repeat_count = |ms: u32| -> u32 {
            if ms < self.repeat_delay_ms { 0 }
            else { 1 + (ms - self.repeat_delay_ms) / self.repeat_interval_ms }
        };
        let n = repeat_count(next).saturating_sub(repeat_count(previous)).min(8);
        (0..n).map(|_| GameAction::CursorStep { dx: self.direction.x, dy: self.direction.y }).collect()
    }

    fn set_hover(&mut self, x: i32, y: i32, out: &mut Vec<GameAction>) {
        let tile = self.viewport.locate(x, y);
        if tile != self.hover { self.hover = tile; out.push(GameAction::HoverTile(tile)); }
    }

    fn refresh_direction(&mut self, out: &mut Vec<GameAction>) {
        let dir = self.calculate_direction();
        if dir != self.direction {
            self.direction = dir;
            self.held_ms = 0;
            if dir != Direction::default() {
                out.push(GameAction::CursorStep { dx: dir.x, dy: dir.y });
            }
        }
    }

    fn calculate_direction(&self) -> Direction {
        let left_key = self.keys.contains(&Key::Left) || self.keys.contains(&Key::A);
        let right_key = self.keys.contains(&Key::Right) || self.keys.contains(&Key::D);
        let up_key = self.keys.contains(&Key::Up) || self.keys.contains(&Key::W);
        let down_key = self.keys.contains(&Key::Down) || self.keys.contains(&Key::S);
        let mut left = left_key;
        let mut right = right_key;
        let mut up = up_key;
        let mut down = down_key;
        for pad in self.pads.values() {
            left |= pad.held.contains(&PadButton::Left) || pad.axis_x == -1;
            right |= pad.held.contains(&PadButton::Right) || pad.axis_x == 1;
            up |= pad.held.contains(&PadButton::Up) || pad.axis_y == -1;
            down |= pad.held.contains(&PadButton::Down) || pad.axis_y == 1;
        }
        Direction { x: (right as i8) - (left as i8), y: (down as i8) - (up as i8) }
    }
}

fn axis_direction(previous: i8, value: i16) -> i8 {
    // Histerese: evita oscilações próximas do limiar e preserva o sinal.
    if previous == -1 && value <= -STICK_RELEASE_THRESHOLD { return -1; }
    if previous == 1 && value >= STICK_RELEASE_THRESHOLD { return 1; }
    if value <= -STICK_PRESS_THRESHOLD { -1 }
    else if value >= STICK_PRESS_THRESHOLD { 1 }
    else { 0 }
}

fn key_action(key: Key) -> Option<GameAction> {
    match key {
        Key::Enter | Key::Space => Some(GameAction::Confirm),
        Key::Escape => Some(GameAction::Cancel),
        Key::Backspace | Key::Z => Some(GameAction::Undo),
        Key::E => Some(GameAction::EndTurn),
        Key::R => Some(GameAction::ResetTurn),
        Key::Tab => Some(GameAction::ToggleMenu),
        _ => None,
    }
}

fn pad_action(button: PadButton) -> Option<GameAction> {
    match button {
        PadButton::South => Some(GameAction::Confirm),
        PadButton::East => Some(GameAction::Cancel),
        PadButton::West | PadButton::LeftShoulder => Some(GameAction::Undo),
        PadButton::RightShoulder => Some(GameAction::EndTurn),
        PadButton::North => Some(GameAction::ResetTurn),
        PadButton::Start | PadButton::Select => Some(GameAction::ToggleMenu),
        _ => None,
    }
}
