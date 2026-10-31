//! Laboratório SDL2: recebe teclado, mouse e controles SDL GameController.
//! `cargo run --release --features sdl2-backend --bin input_lab`
use std::collections::HashMap;
use std::time::Instant;

use itb_core::board_queries::{BoardSnapshot, CellState};
use itb_core::interaction::{BoardInteraction, InteractionEvent};
use itb_core::input::{DeviceEvent, GameAction, InputSystem, Key, MouseButton, PadAxis, PadButton, Tile, Viewport};
use sdl2::controller::{Axis, Button, GameController};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton as SdlMouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::GameControllerSubsystem;

const COLS: i32 = 8;
const ROWS: i32 = 8;

struct Demo {
    cursor: Tile,
    hovered: Option<Tile>,
    selected: Option<Tile>,
    turn: u32,
    zoom: i32,
    menu: bool,
    message: &'static str,
    board: BoardSnapshot,
    interaction: BoardInteraction,
}

impl Default for Demo {
    fn default() -> Self {
        let mut board = BoardSnapshot::new(8, 8, CellState::default()).expect("tabuleiro de teste valido");
        // Posições de unidades fictícias apenas para testar interação; NÃO são dados do jogo.
        for (x,y,team) in [(2,2,0),(5,5,1)] {
            board.set_cell(x,y,CellState{active_pawn_team:Some(team),..CellState::default()}).expect("posicao fixa valida");
        }
        let interaction=BoardInteraction::new(&board);
        Self {
            board, interaction,
            cursor: Tile { col: 4, row: 4 }, hovered: None, selected: None,
            turn: 1, zoom: 56, menu: false,
            message: "Pronto: use mouse, teclado ou controle",
        }
    }
}

impl Demo {
    fn viewport(&self, width: u32, height: u32) -> Viewport {
        let available_w = width.saturating_sub(24) / COLS as u32;
        let available_h = height.saturating_sub(90) / ROWS as u32;
        let size = available_w.min(available_h).min(self.zoom as u32).max(1);
        let board_width = size * COLS as u32;
        let board_height = size * ROWS as u32;
        Viewport {
            x: ((width.saturating_sub(board_width)) / 2) as i32,
            y: ((height.saturating_sub(board_height)) / 2) as i32,
            width: board_width, height: board_height,
            columns: COLS as u32, rows: ROWS as u32,
        }
    }

    fn apply(&mut self, action: GameAction) {
        let events=self.interaction.apply(&self.board,action);
        self.cursor=self.interaction.cursor();
        self.hovered=self.interaction.hover();
        self.selected=self.interaction.selection();
        for event in events {
            self.message=match event {
                InteractionEvent::Cursor(_) => "Cursor movido",
                InteractionEvent::Hover(_) => "Mouse sobre tabuleiro",
                InteractionEvent::Selected(_) => "Unidade de teste selecionada",
                InteractionEvent::SelectionCleared => "Selecao cancelada",
                InteractionEvent::MoveRequested{..} => "Movimento solicitado, NAO executado",
                InteractionEvent::UndoRequested => "Desfazer solicitado, NAO executado",
                InteractionEvent::ResetRequested => "Reiniciar solicitado, NAO executado",
                InteractionEvent::EndTurnRequested => "Terminar turno solicitado, NAO executado",
                InteractionEvent::MenuRequested => "Menu alternado no laboratorio",
                InteractionEvent::ZoomRequested(_) => "Zoom ajustado no laboratorio",
            };
            if let InteractionEvent::MoveRequested{from,to}=event {
                println!("Pedido de movimento: ({},{}) -> ({},{}) [nao aplicado]",from.col,from.row,to.col,to.row);
            }
        }
        match action {
            GameAction::Zoom(delta)=>{self.zoom=(self.zoom+i32::from(delta)*4).clamp(24,96);},
            GameAction::ToggleMenu=>{self.menu=!self.menu;},
            // Aumentar o contador de turnos aqui seria inventar regras nativas.
            _=>{}
        }
    }

}

fn key_from_sdl(k: Keycode) -> Option<Key> {
    Some(match k {
        Keycode::Up => Key::Up, Keycode::Down => Key::Down,
        Keycode::Left => Key::Left, Keycode::Right => Key::Right,
        Keycode::W => Key::W, Keycode::A => Key::A,
        Keycode::S => Key::S, Keycode::D => Key::D,
        Keycode::Return | Keycode::KpEnter => Key::Enter,
        Keycode::Space => Key::Space, Keycode::Escape => Key::Escape,
        Keycode::Backspace => Key::Backspace, Keycode::Tab => Key::Tab,
        Keycode::E => Key::E, Keycode::Z => Key::Z, Keycode::R => Key::R,
        _ => return None,
    })
}

fn button_from_sdl(b: Button) -> Option<PadButton> {
    Some(match b {
        Button::DPadUp => PadButton::Up, Button::DPadDown => PadButton::Down,
        Button::DPadLeft => PadButton::Left, Button::DPadRight => PadButton::Right,
        Button::A => PadButton::South, Button::B => PadButton::East,
        Button::X => PadButton::West, Button::Y => PadButton::North,
        Button::Start => PadButton::Start, Button::Back => PadButton::Select,
        Button::LeftShoulder => PadButton::LeftShoulder,
        Button::RightShoulder => PadButton::RightShoulder,
        _ => return None,
    })
}

fn open_controller(
    subsystem: &GameControllerSubsystem, index: u32,
    devices: &mut HashMap<u32, GameController>, input: &mut InputSystem,
) {
    if !subsystem.is_game_controller(index) { return; }
    match subsystem.open(index) {
        Ok(controller) => {
            let id = controller.instance_id();
            if devices.contains_key(&id) { return; }
            println!("Controle conectado: {} (instancia {})", controller.name(), id);
            devices.insert(id, controller);
            input.handle(DeviceEvent::ControllerConnected { id });
        }
        Err(err) => eprintln!("Controle nao abriu (indice {index}): {err}"),
    }
}

fn render(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, demo: &Demo) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(20, 24, 31));
    canvas.clear();
    let (width, height) = canvas.window().size();
    let v = demo.viewport(width, height);
    let cell = v.width / v.columns;
    for row in 0..ROWS {
        for col in 0..COLS {
            let tile = Tile { col, row };
            let is_cursor = tile == demo.cursor;
            let is_selected = demo.selected == Some(tile);
            let is_hovered = demo.hovered == Some(tile);
            let color = if is_selected { Color::RGB(56, 185, 118) }
                else if is_cursor { Color::RGB(253, 204, 75) }
                else if is_hovered { Color::RGB(102, 137, 213) }
                else if (row + col) % 2 == 0 { Color::RGB(60, 76, 93) }
                else { Color::RGB(53, 66, 79) };
            let x = v.x + col * cell as i32;
            let y = v.y + row * cell as i32;
            canvas.set_draw_color(color);
            canvas.fill_rect(Rect::new(x + 1, y + 1, cell.saturating_sub(2), cell.saturating_sub(2)))?;
            // Marcador abstrato para unidades FICTÍCIAS do laboratório.
            if demo.board.get_cell(col,row).active_pawn_team.is_some() {
                canvas.set_draw_color(Color::RGB(231, 235, 239));
                let s=cell / 3;
                canvas.fill_rect(Rect::new(x+(cell-s) as i32 / 2,y+(cell-s) as i32 / 2,s,s))?;
            }
        }
    }
    canvas.present();
    Ok(())
}

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let controller_subsystem = sdl.game_controller()?;
    let window = video.window("Into the Breach / Rust - Input Lab", 840, 690)
        .position_centered().resizable().build().map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().accelerated().present_vsync()
        .build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl.event_pump()?;
    let mut demo = Demo::default();
    let mut input = InputSystem::new(demo.viewport(840, 690));
    let mut devices: HashMap<u32, GameController> = HashMap::new();
    for i in 0..controller_subsystem.num_joysticks()? {
        open_controller(&controller_subsystem, i, &mut devices, &mut input);
    }
    let mut last = Instant::now();

    println!("Teclado: WASD/setas = cursor, Enter/Espaco = selecionar, Esc = voltar, E = turno, Z = desfazer, R = reset, Tab = menu");
    println!("Mouse: movimento = hover, esquerdo = selecionar, direito = voltar, roda = zoom");
    println!("Controle: analogico/D-pad = cursor, A = selecionar, B = voltar, RB = turno, LB/X = desfazer, Y = reset, Start = menu");

    'running: loop {
        input.set_viewport(demo.viewport(canvas.window().size().0, canvas.window().size().1));
        for event in event_pump.poll_iter() {
            let translated = match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Q), repeat: false, .. } => break 'running,
                Event::KeyDown { keycode: Some(k), repeat, .. } => key_from_sdl(k)
                    .map(|key| DeviceEvent::KeyDown { key, repeat }),
                Event::KeyUp { keycode: Some(k), .. } => key_from_sdl(k)
                    .map(|key| DeviceEvent::KeyUp { key }),
                Event::MouseMotion { x, y, .. } => Some(DeviceEvent::MouseMoved { x, y }),
                Event::MouseButtonDown { mouse_btn, x, y, .. } => match mouse_btn {
                    SdlMouseButton::Left => Some(DeviceEvent::MouseDown { button: MouseButton::Left, x, y }),
                    SdlMouseButton::Right => Some(DeviceEvent::MouseDown { button: MouseButton::Right, x, y }),
                    SdlMouseButton::Middle => Some(DeviceEvent::MouseDown { button: MouseButton::Middle, x, y }),
                    _ => None,
                },
                Event::MouseButtonUp { mouse_btn, x, y, .. } => match mouse_btn {
                    SdlMouseButton::Left => Some(DeviceEvent::MouseUp { button: MouseButton::Left, x, y }),
                    SdlMouseButton::Right => Some(DeviceEvent::MouseUp { button: MouseButton::Right, x, y }),
                    SdlMouseButton::Middle => Some(DeviceEvent::MouseUp { button: MouseButton::Middle, x, y }),
                    _ => None,
                },
                Event::MouseWheel { y, .. } => Some(DeviceEvent::MouseWheel { y }),
                Event::ControllerDeviceAdded { which, .. } => {
                    open_controller(&controller_subsystem, which, &mut devices, &mut input);
                    None
                }
                Event::ControllerDeviceRemoved { which, .. } => {
                    devices.remove(&which);
                    Some(DeviceEvent::ControllerDisconnected { id: which })
                }
                Event::ControllerButtonDown { which, button, .. } => button_from_sdl(button)
                    .map(|button| DeviceEvent::ControllerDown { id: which, button }),
                Event::ControllerButtonUp { which, button, .. } => button_from_sdl(button)
                    .map(|button| DeviceEvent::ControllerUp { id: which, button }),
                Event::ControllerAxisMotion { which, axis: Axis::LeftX, value, .. } =>
                    Some(DeviceEvent::ControllerAxis { id: which, axis: PadAxis::LeftX, value }),
                Event::ControllerAxisMotion { which, axis: Axis::LeftY, value, .. } =>
                    Some(DeviceEvent::ControllerAxis { id: which, axis: PadAxis::LeftY, value }),
                Event::Window { win_event: WindowEvent::FocusLost, .. } => Some(DeviceEvent::FocusLost),
                _ => None,
            };
            if let Some(event) = translated {
                for action in input.handle(event) { demo.apply(action); }
            }
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_millis().min(200) as u32;
        last = now;
        for action in input.tick(dt) { demo.apply(action); }
        render(&mut canvas, &demo)?;
        let title = format!(
            "ITB Rust: entrada | Turno {} | Cursor {},{} | Controles {} | {}",
            demo.turn, demo.cursor.col, demo.cursor.row, input.connected_controllers(), demo.message,
        );
        canvas.window_mut().set_title(&title).map_err(|e| e.to_string())?;
    }
    Ok(())
}
