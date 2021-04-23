use tangram_term::{event::Event, screen::Screen, style::Style};

fn main() {
	let mut screen = Screen::open().unwrap();
	screen.hide_cursor().unwrap();
	loop {
		let event = screen.listen().unwrap();
		match event {
			Event::Char('q') => break,
			_ => {
				screen.clear_row(0);
				screen.put_str(0, 0, Style::default(), &format!("{}", event));
				screen.flush().unwrap();
			}
		}
	}
}
