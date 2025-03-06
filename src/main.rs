use std::{env, time::Duration, io, io::Write, collections::VecDeque, thread::sleep};
use crossterm::{execute, cursor, terminal, event, style, event::Event};
use rand::distributions::Distribution;

#[derive(PartialEq)]
enum Direction
{
    Up, Down, Left, Right
}

// +----------+    +------------+    +------------+
// | Score: 0 |    | Score:  10 |    | Score: 100 |
// |  Replay  |    |   Replay   |    |   Replay   |
// |   Quit   |    |    Quit    |    |    Quit    |
// +----------+    +------------+    +------------+

struct SnakeInfo {
    pub head_pos: (u16,u16),
    pub body_pos: VecDeque<(u16,u16)>,
    pub growing: u8,
    pub dir: Direction,
}

impl SnakeInfo {
    pub fn new(term_size: (u16,u16)) -> Self {
        SnakeInfo {
            head_pos: (term_size.0 / 5, term_size.1 / 2),
            body_pos: VecDeque::new(),
            growing: 2,
            dir: Direction::Right,
        }
    }

    pub fn update(&mut self) {
        let _ = execute!(io::stdout(), cursor::MoveTo(self.head_pos.0, self.head_pos.1));
        print!("▒");
        self.body_pos.push_back(self.head_pos);
        if self.growing > 0 {
            self.growing -= 1;
        }
        else {
            let _pos = self.body_pos.pop_front();
            move_cursor_to(_pos.expect("Reason"));
            print!(" ");
        }
        match self.dir {
            Direction::Up => self.head_pos.1 -= 1,
            Direction::Down => self.head_pos.1 += 1,
            Direction::Left => self.head_pos.0 -= 1,
            Direction::Right => self.head_pos.0 += 1
        }
        move_cursor_to(self.head_pos);
        print!("█");
    }

    fn should_be_dead(&mut self, term_size: (u16, u16)) -> bool {
        for &i in self.body_pos.iter() {
            if self.head_pos == i {
                return true;
            }
        }
    
        return self.head_pos.0 < 1 || self.head_pos.1 < 1 || self.head_pos.0 >= term_size.0 - 1 || self.head_pos.1 >= term_size.1 - 1;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let delay = if args.len() > 1 {
        args[1].parse::<u64>().unwrap()
    }
    else {
        100
    };

    let mut term_size: (u16, u16) = start_game();
    let mut start_term_size = term_size;
    let mut middle: (u16,u16) = (start_term_size.0 / 2, start_term_size.1 / 2);
    let mut fruit_pos: (u16,u16) = (start_term_size.0 / 5 * 4, start_term_size.1 / 2);
    let mut score: u8 = 0;
    let mut menu_selection: u8 = 0;
    let mut playing: bool = true;
    let mut focused: bool = true;
    let mut snake: SnakeInfo = SnakeInfo::new(start_term_size);
    move_cursor_to(fruit_pos);
    print!("■");
    'game_loop: loop {
        sleep(Duration::from_millis(delay));
        loop {
            if event::poll(Duration::from_millis(0)).unwrap() {
                match event::read().unwrap() {
                    Event::FocusGained => focused = true, // TODO unpause properly
                    Event::FocusLost => focused = false, // TODO pause properly
                    Event::Key(event) => {
                        if event.code == event::KeyCode::Char('q') {
                            break;
                        }
                        else if playing {
                            if event.code == event::KeyCode::Up {
                                if snake.dir == Direction::Down || snake.dir == Direction::Up {
                                    continue;
                                }
                                snake.dir = Direction::Up;
                            }
                            else if event.code == event::KeyCode::Down {
                                if snake.dir == Direction::Down || snake.dir == Direction::Up {
                                    continue;
                                }
                                snake.dir = Direction::Down;
                            }
                            else if event.code == event::KeyCode::Left {
                                if snake.dir == Direction::Left || snake.dir == Direction::Right {
                                    continue;
                                }
                                snake.dir = Direction::Left;
                            }
                            else if event.code == event::KeyCode::Right {
                                if snake.dir == Direction::Left || snake.dir == Direction::Right {
                                    continue;
                                }
                                snake.dir = Direction::Right;
                            }
                        }
                        else {
                            if event.code == event::KeyCode::Up || event.code == event::KeyCode::Down {
                                menu_selection = menu_selection ^ 1;
                                rewrite_menu(middle, menu_selection, score);
                            }
                            else if event.code == event::KeyCode::Enter {
                                if menu_selection == 0 {
                                    start_term_size = term_size;
                                    middle = (start_term_size.0 / 2, start_term_size.1 / 2);
                                    snake = SnakeInfo::new(start_term_size);
                                    draw_box((0,0), start_term_size);
                                    fruit_pos = (start_term_size.0 / 5 * 4, start_term_size.1 / 2);
                                    move_cursor_to(fruit_pos);
                                    print!("■");
                                    score = 0;
                                    playing = true;
                                }
                                else {
                                    break 'game_loop;
                                }
                            }
                        }
                    },
                    Event::Resize(width, height) => {
                        move_cursor_to((0,0));
                        print!("Terminal resized, apply in next game");
                        term_size = (width, height)
                    },
                    _default => continue
                }
            }
            break;
        }
        
        if playing && focused {
            snake.update();
            if snake.head_pos == fruit_pos {
                snake.growing += 2;
                score += 1;
                fruit_pos = place_new_fruit(&snake.body_pos, start_term_size);
                move_cursor_to(fruit_pos);
                print!("■");
            }
            let _ = io::stdout().flush();
            if snake.should_be_dead(start_term_size) {
                playing = false;
                draw_box((middle.0 - 7, middle.1 - 2), (middle.0 + 7, middle.1 + 2));
                rewrite_menu(middle, menu_selection, score);
            }
        }
    }

    end_game();
}

fn rewrite_menu(middle: (u16, u16), selection: u8, score: u8) {
    move_cursor_to((middle.0 - 6, middle.1 - 1));
    let _ = print!(" Score: {:03} ", score);
    move_cursor_to((middle.0 - 6, middle.1));
    if selection == 0 {
        let _ = execute!(io::stdout(), style::SetBackgroundColor(style::Color::White), style::SetForegroundColor(style::Color::Black));
    }
    let _ = print!("   Replay   ");
    if selection == 1 {
        let _ = execute!(io::stdout(), style::SetBackgroundColor(style::Color::White), style::SetForegroundColor(style::Color::Black));
    }
    else {
        let _ = execute!(io::stdout(), style::ResetColor);
    }
    move_cursor_to((middle.0 - 6, middle.1 + 1));
    let _ = print!("    Quit    ");
    if selection == 1 {
        let _ = execute!(io::stdout(), style::ResetColor);
    }
    let _ = io::stdout().flush();
}

fn place_new_fruit(b_pos: &VecDeque<(u16,u16)>, term_size: (u16,u16)) -> (u16, u16) {
    let mut rng = rand::thread_rng();
    loop {
        let x = rand::distributions::Uniform::from(1..term_size.0 - 1).sample(&mut rng);
        let y = rand::distributions::Uniform::from(1..term_size.1 - 1).sample(&mut rng);
        let n_pos: (u16,u16) = (x, y);

        if is_position_free(n_pos, b_pos) {
            return n_pos;
        }
    }
}

fn draw_box(top_left: (u16, u16), bottom_right: (u16, u16)) {
    move_cursor_to(top_left);
    let _ = print!("┌");
    for _i in 1..(bottom_right.0 - top_left.0 - 1)
    {
        let _ = print!("─");
    }
    let _ = print!("┐");
    for y in (top_left.1 + 1)..(bottom_right.1)
    {
        move_cursor_to((top_left.0, y));
        let _ = print!("│");
        for _i in 1..(bottom_right.0 - top_left.0 - 1)
        {
            let _ = print!(" ");
        }
        let _ = print!("│");
    }
    move_cursor_to((top_left.0, bottom_right.1));
    let _ = print!("└");
    for _i in 1..(bottom_right.0 - top_left.0 - 1)
    {
        let _ = print!("─");
    }
    let _ = print!("┘");
    let _ = io::stdout().flush();
}

fn is_position_free(pos: (u16,u16), body: &VecDeque<(u16,u16)>) -> bool {
    for i in body {
        if pos == *i {
            return false;
        }
    }
    return true;
}

fn move_cursor_to(pos: (u16,u16)) {
    let _ = execute!(io::stdout(), cursor::MoveTo(pos.0, pos.1));
}

fn start_game() -> (u16, u16) {
    let _ = execute!(io::stdout(), terminal::EnterAlternateScreen, event::EnableMouseCapture, event::EnableFocusChange, cursor::Hide);
    let _ = terminal::enable_raw_mode();
    // if these go wrong, the game still works but looks less good, should probably just claim that it failed to launch

    let t = terminal::size().unwrap();
    draw_box((0,0), t);
    return t;
}

fn end_game() {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), terminal::LeaveAlternateScreen, event::DisableMouseCapture, event::DisableFocusChange, cursor::Show);
    // ignore if stuff goes wrong, it's the end of the game anyways
}
