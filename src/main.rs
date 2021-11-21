use x11rb::{connection::Connection, cursor::Handle as CursorHande, protocol::{Event, xproto::{AtomEnum, ColormapAlloc, ConnectionExt, CreateGCAux, CreateWindowAux, EventMask, Gravity, PropMode, Rectangle, WindowClass, free_pixmap}}, resource_manager::Database, wrapper::ConnectionExt as _};

fn main() -> anyhow::Result<()> {
    let (conn, screen_num) = x11rb::xcb_ffi::XCBConnection::connect(None)?;

    let screen = &conn.setup().roots[screen_num];
    let window = conn.generate_id()?;
    let gc_id = conn.generate_id()?;
    let cmap = conn.generate_id()?;
    let resource_db = Database::new_from_default(&conn)?;
    let cursor_handle = CursorHande::new(&conn, screen_num, &resource_db)?;

    let wm_proctocals = conn.intern_atom(false, b"WM_PROTOCOLS")?;
    let wm_proctocals = wm_proctocals.reply()?.atom;
    let wm_delete_window = conn.intern_atom(false, b"WM_DELETE_WINDOW")?;
    let wm_delete_window = wm_delete_window.reply()?.atom;
    let net_wm_name = conn.intern_atom(false, b"_NET_SM_NAME")?;
    let net_wm_name = net_wm_name.reply()?.atom;
    let utf8_str = conn.intern_atom(false, b"UTF8_STRING")?;
    let utf8_str = utf8_str.reply()?.atom;
    let cursor_handle = cursor_handle.reply()?;

    conn.create_colormap(ColormapAlloc::NONE, cmap, window, screen.root_visual)?;
    let rep = conn.alloc_color(cmap, 0xFF, 0, 0)?;
    let rep = rep.reply()?;
    let win_aux = CreateWindowAux::new()
        .event_mask(EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY | EventMask::NO_EVENT)
        .background_pixel(screen.black_pixel)
        .win_gravity(Gravity::NORTH_WEST)
        .cursor(cursor_handle.load_cursor(&conn, "wait")?);

    let gc_aux = CreateGCAux::new().foreground(rep.pixel);

    let (mut width, mut height) = (100, 100);

    conn.create_window(
        screen.root_depth,
        window,
        screen.root,
        0,
        0,
        width,
        height,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &win_aux,
    )?;
    let title = "Testing";

    conn.change_property8(
        PropMode::REPLACE,
        window,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        title.as_bytes(),
    )?;
    conn.change_property8(
        PropMode::REPLACE,
        window,
        net_wm_name,
        utf8_str,
        title.as_bytes(),
    )?;
    conn.change_property8(
        PropMode::REPLACE,
        window,
        AtomEnum::WM_CLASS,
        AtomEnum::STRING,
        b"testing\0simple_window\0",
    )?;
    conn.change_property32(
        PropMode::REPLACE,
        window,
        wm_proctocals,
        AtomEnum::ATOM,
        &[wm_delete_window],
    )?;
    let reply = conn
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 1024)?
        .reply()?;
    assert_eq!(reply.value, title.as_bytes());
    conn.create_gc(gc_id, window, &gc_aux)?;
    conn.map_window(window)?;
    conn.flush()?;
    loop {
        let evt = conn.wait_for_event()?;
        match evt {
            Event::Expose(evt) => {
                if evt.count == 0 {
                    let rect = Rectangle {
                        x: 100,
                        y: 100,
                        width: width.saturating_sub(200),
                        height: height.saturating_sub(200),
                    };
                    conn.poly_fill_rectangle(window, gc_id, &[rect])?;
                    conn.flush()?;
                }
            }
            Event::ConfigureNotify(evt) => {
                width = evt.width;
                height = evt.height;
            }
            Event::ClientMessage(evt) => {
                let data = evt.data.as_data32();
                if evt.format == 32 && evt.window == window && data[0] == wm_delete_window {
                    println!("close event");
                    return Ok(());
                }
            }
            _ => println!("Go unknown event"),
        }
    }
}
