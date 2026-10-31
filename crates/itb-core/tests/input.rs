use itb_core::input::{
    DeviceEvent as Ev, GameAction as Act, InputSystem, Key, MouseButton, PadAxis, PadButton,
    Tile, Viewport,
};

fn viewport() -> Viewport {
    Viewport { x: 10, y: 20, width: 400, height: 400, columns: 8, rows: 8 }
}
fn input() -> InputSystem { InputSystem::new(viewport()) }
fn down(key: Key) -> Ev { Ev::KeyDown { key, repeat: false } }
fn up(key: Key) -> Ev { Ev::KeyUp { key } }
fn step(dx: i8, dy: i8) -> Act { Act::CursorStep { dx, dy } }

#[test]
fn keyboard_maps_commands_and_ignores_os_autorepeat() {
    let mut x = input();
    assert_eq!(x.handle(down(Key::Enter)), vec![Act::Confirm]);
    assert!(x.handle(Ev::KeyDown { key: Key::Enter, repeat: true }).is_empty());
    assert!(x.handle(down(Key::Enter)).is_empty());
    x.handle(up(Key::Enter));
    assert_eq!(x.handle(down(Key::Space)), vec![Act::Confirm]);
    x.handle(up(Key::Space));
    assert_eq!(x.handle(down(Key::Escape)), vec![Act::Cancel]);
    assert_eq!(x.handle(down(Key::E)), vec![Act::EndTurn]);
    assert_eq!(x.handle(down(Key::Z)), vec![Act::Undo]);
    assert_eq!(x.handle(down(Key::R)), vec![Act::ResetTurn]);
    assert_eq!(x.handle(down(Key::Tab)), vec![Act::ToggleMenu]);
}

#[test]
fn keyboard_arrows_and_wasd_are_equivalent_and_composable() {
    let mut x = input();
    assert_eq!(x.handle(down(Key::Right)), vec![step(1, 0)]);
    assert!(x.handle(down(Key::D)).is_empty());
    assert!(x.handle(up(Key::Right)).is_empty()); // D continua pressionado
    assert_eq!(x.handle(down(Key::Up)), vec![step(1, -1)]);
    assert_eq!(x.handle(up(Key::D)), vec![step(0, -1)]);
    assert!(x.handle(up(Key::Up)).is_empty());
    assert!(x.tick(1000).is_empty());
}

#[test]
fn opposite_directions_cancel_without_releasing_other_keys() {
    let mut x = input();
    assert_eq!(x.handle(down(Key::Left)), vec![step(-1, 0)]);
    assert!(x.handle(down(Key::Right)).is_empty());
    assert_eq!(x.handle(up(Key::Left)), vec![step(1, 0)]);
    assert!(x.handle(up(Key::Right)).is_empty());
}

#[test]
fn held_navigation_repeats_by_time_independently_of_os() {
    let mut x = input();
    assert_eq!(x.handle(down(Key::Down)), vec![step(0, 1)]);
    assert!(x.tick(249).is_empty());
    assert_eq!(x.tick(1), vec![step(0, 1)]);
    assert!(x.tick(109).is_empty());
    assert_eq!(x.tick(1), vec![step(0, 1)]);
    x.handle(up(Key::Down));
    assert!(x.tick(900).is_empty());
}

#[test]
fn long_frame_is_capped_to_eight_repeat_events() {
    let mut x = input();
    x.handle(down(Key::Right));
    assert_eq!(x.tick(15_000).len(), 8);
    assert!(x.tick(0).is_empty());
}

#[test]
fn controller_hotplug_buttons_and_dpad() {
    let mut x = input();
    x.handle(Ev::ControllerConnected { id: 41 });
    assert_eq!(x.connected_controllers(), 1);
    assert_eq!(x.handle(Ev::ControllerDown { id: 41, button: PadButton::Right }), vec![step(1, 0)]);
    assert!(x.handle(Ev::ControllerDown { id: 41, button: PadButton::Right }).is_empty());
    assert_eq!(x.handle(Ev::ControllerDown { id: 41, button: PadButton::South }), vec![Act::Confirm]);
    assert_eq!(x.handle(Ev::ControllerDown { id: 41, button: PadButton::East }), vec![Act::Cancel]);
    assert_eq!(x.handle(Ev::ControllerDown { id: 41, button: PadButton::RightShoulder }), vec![Act::EndTurn]);
    assert_eq!(x.handle(Ev::ControllerDown { id: 41, button: PadButton::LeftShoulder }), vec![Act::Undo]);
    assert_eq!(x.handle(Ev::ControllerDown { id: 41, button: PadButton::North }), vec![Act::ResetTurn]);
    assert_eq!(x.handle(Ev::ControllerDown { id: 41, button: PadButton::Start }), vec![Act::ToggleMenu]);
    x.handle(Ev::ControllerDisconnected { id: 41 });
    assert_eq!(x.connected_controllers(), 0);
    assert!(x.tick(500).is_empty());
}

#[test]
fn analog_stick_uses_hysteresis_and_returns_to_neutral() {
    let mut x = input();
    x.handle(Ev::ControllerConnected { id: 3 });
    assert!(x.handle(Ev::ControllerAxis { id: 3, axis: PadAxis::LeftX, value: 13_999 }).is_empty());
    assert_eq!(x.handle(Ev::ControllerAxis { id: 3, axis: PadAxis::LeftX, value: 20_000 }), vec![step(1, 0)]);
    assert!(x.handle(Ev::ControllerAxis { id: 3, axis: PadAxis::LeftX, value: 10_000 }).is_empty());
    assert!(x.handle(Ev::ControllerAxis { id: 3, axis: PadAxis::LeftX, value: 8_000 }).is_empty());
    assert!(x.tick(300).is_empty());
    assert_eq!(x.handle(Ev::ControllerAxis { id: 3, axis: PadAxis::LeftY, value: -20_000 }), vec![step(0, -1)]);
    assert_eq!(x.handle(Ev::ControllerAxis { id: 3, axis: PadAxis::LeftX, value: -20_000 }), vec![step(-1, -1)]);
}

#[test]
fn multiple_devices_do_not_block_each_other() {
    let mut x = input();
    x.handle(Ev::ControllerConnected { id: 1 });
    x.handle(Ev::ControllerConnected { id: 2 });
    assert_eq!(x.handle(Ev::ControllerDown { id: 1, button: PadButton::Up }), vec![step(0, -1)]);
    assert!(x.handle(down(Key::W)).is_empty());
    assert!(x.handle(Ev::ControllerDisconnected { id: 1 }).is_empty());
    assert_eq!(x.connected_controllers(), 1);
    assert!(x.tick(250).contains(&step(0, -1)));
    x.handle(up(Key::W));
    assert!(x.tick(500).is_empty());
}

#[test]
fn viewport_maps_pixels_and_excludes_right_and_bottom_edges() {
    let v = viewport();
    assert_eq!(v.locate(10, 20), Some(Tile { col: 0, row: 0 }));
    assert_eq!(v.locate(60, 70), Some(Tile { col: 1, row: 1 }));
    assert_eq!(v.locate(409, 419), Some(Tile { col: 7, row: 7 }));
    assert_eq!(v.locate(410, 419), None);
    assert_eq!(v.locate(409, 420), None);
    assert_eq!(v.locate(9, 20), None);
    assert_eq!(v.locate(10, 19), None);
    assert_eq!(Viewport { width: 0, ..v }.locate(10, 20), None);
    assert_eq!(Viewport { columns: 0, ..v }.locate(10, 20), None);
}

#[test]
fn mouse_hover_click_right_click_and_wheel() {
    let mut x = input();
    let tile = Tile { col: 2, row: 3 };
    assert_eq!(x.handle(Ev::MouseMoved { x: 112, y: 175 }), vec![Act::HoverTile(Some(tile))]);
    assert!(x.handle(Ev::MouseMoved { x: 120, y: 178 }).is_empty());
    assert_eq!(x.handle(Ev::MouseDown { button: MouseButton::Left, x: 120, y: 178 }), vec![Act::SelectTile(tile)]);
    assert_eq!(x.handle(Ev::MouseDown { button: MouseButton::Right, x: 120, y: 178 }), vec![Act::Cancel]);
    assert_eq!(x.handle(Ev::MouseWheel { y: 4 }), vec![Act::Zoom(1)]);
    assert_eq!(x.handle(Ev::MouseWheel { y: -1 }), vec![Act::Zoom(-1)]);
    assert!(x.handle(Ev::MouseWheel { y: 0 }).is_empty());
    assert_eq!(x.handle(Ev::MouseMoved { x: -1, y: 50 }), vec![Act::HoverTile(None)]);
    assert!(x.handle(Ev::MouseDown { button: MouseButton::Left, x: -1, y: 50 }).is_empty());
}

#[test]
fn focus_loss_releases_all_inputs_and_hover() {
    let mut x = input();
    x.handle(down(Key::Up));
    x.handle(Ev::ControllerDown { id: 4, button: PadButton::Down });
    x.handle(Ev::MouseMoved { x: 15, y: 25 });
    assert_eq!(x.handle(Ev::FocusLost), vec![Act::HoverTile(None)]);
    assert!(x.tick(10000).is_empty());
    assert_eq!(x.handle(down(Key::Up)), vec![step(0, -1)]);
}

#[test]
fn resizing_viewport_retargets_clicks() {
    let mut x = input();
    x.set_viewport(Viewport { x: 0, y: 0, width: 80, height: 80, columns: 8, rows: 8 });
    assert_eq!(x.handle(Ev::MouseDown { button: MouseButton::Left, x: 79, y: 79 }),
        vec![Act::HoverTile(Some(Tile { col: 7, row: 7 })), Act::SelectTile(Tile { col: 7, row: 7 })]);
}
