/*!
`term` is a crate for building terminal user interfaces in Rust.

```
use term::{Event, Screen, Style};

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
```

`term-ui` uses /dev/tty on Unix and $CONOUT$ on Windows instead of stdin/stdout so that your programs can still use standard I/O to do something useful. For example, you could write an interactive text editor that could be invoked like this.

```bash
echo "hello" | your-editor | # use the output...
```

Instead of interpreting "hello" as keystrokes in the interface, the text editor could use "hello" as the base text, and write the resulting text to stdout.

A `Screen` is a two dimensional grid of `Cell`s, each of which stores a single character and associated style information such as color, bold, etc. Your application can manipulate that grid, and then call "flush" on the `Screen` to write any changes to the terminal. Nothing is written to the terminal until `flush()` is called, and the library keeps track of which cells have changed to minimize the amount of data to write. This means your code can be less clever in keeping track of what parts of the screen need to be changed but your application will remain reasonably fast.
*/

pub mod event;
pub mod parser;
pub mod screen;
pub mod style;
pub mod terminal;
