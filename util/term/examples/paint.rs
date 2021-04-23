use tangram_error::Result;
use tangram_term::{
	event::{Event, MouseEvent},
	screen::Screen,
	style::{Color, Style, StyleMask},
};

fn main() -> Result<()> {
	let mut screen = Screen::open()?;
	screen.hide_cursor()?;
	loop {
		let event = screen.listen().unwrap();
		match event {
			Event::Char('q') => break,
			Event::Char('c') => {
				screen.clear()?;
				screen.flush()?;
			}
			Event::Mouse(e) => match e {
				MouseEvent::Down(row, col) | MouseEvent::Drag(row, col) => {
					let style = Style {
						background_color: Color::Red,
						foreground_color: Color::Red,
						style_mask: StyleMask::NORMAL,
					};
					screen.put(row, col, style, ' ');
					screen.flush()?;
				}
				_ => {}
			},
			_ => {}
		}
	}
	Ok(())
}
