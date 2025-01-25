use std::{time::Duration, io, io::Write, collections::VecDeque};
use crossterm::{execute, cursor, terminal, event};
use rand::distributions::{Distribution, Uniform};

enum Direction
{
    Up, Down, Left, Right
}

fn main()
{
    let termSize = startGame();
    let mut headPos: (u16,u16) = (termSize.0 / 5, termSize.1 / 2);
    let mut dir: Direction = Direction::Right;
    let mut bodyPos: VecDeque<(u16, u16)> = VecDeque::new();
    let mut growing: u8 = 2;
    let mut fruitPos: (u16,u16) = (termSize.0 / 5 * 4, termSize.1 / 2);
    moveCursorTo(fruitPos);
    print!("■");
    loop
    {
        if event::poll(Duration::from_millis(50)).unwrap()
        {
            if let event::Event::Key(event) = event::read().unwrap()
            {
                if event.code == event::KeyCode::Char('q')
                {
                    break;
                }
                else if event.code == event::KeyCode::Up
                {
                    dir = Direction::Up;
                }
                else if event.code == event::KeyCode::Down
                {
                    dir = Direction::Down;
                }
                else if event.code == event::KeyCode::Left
                {
                    dir = Direction::Left;
                }
                else if event.code == event::KeyCode::Right
                {
                    dir = Direction::Right;
                }
            }
        }
        execute!(io::stdout(), cursor::MoveTo(headPos.0, headPos.1));
        print!("▒");
        bodyPos.push_back(headPos);
        if growing > 0
        {
            growing -= 1;
        }
        else
        {
            let pos = bodyPos.pop_front();
            moveCursorTo(pos.expect("Reason"));
            print!(" ");
        }
        match dir
        {
            Direction::Up => headPos.1 -= 1,
            Direction::Down => headPos.1 += 1,
            Direction::Left => headPos.0 -= 1,
            Direction::Right => headPos.0 += 1
        }
        moveCursorTo(headPos);
        print!("█");
        if headPos == fruitPos
        {
            growing += 2;
            fruitPos = placeNewFruit(&bodyPos, termSize);
            moveCursorTo(fruitPos);
            print!("■");
        }
        io::stdout().flush();
        if shouldBeDead(headPos, &bodyPos, termSize)
        {
            break;
        }
    }

    endGame();
}

fn placeNewFruit(bPos: &VecDeque<(u16,u16)>, termSize: (u16,u16)) -> (u16, u16)
{
    let mut rng = rand::thread_rng();
    loop
    {
        let x = rand::distributions::Uniform::from(1..termSize.0 - 1).sample(&mut rng);
        let y = rand::distributions::Uniform::from(1..termSize.1 - 1).sample(&mut rng);
        let nPos: (u16,u16) = (x, y);

        if isPositionFree(nPos, bPos)
        {
            return nPos;
        }
    }
}

fn isPositionFree(pos: (u16,u16), body: &VecDeque<(u16,u16)>) -> bool
{
    for i in body
    {
        if pos == *i
        {
            return false;
        }
    }
    return true;
}

fn shouldBeDead(hpos: (u16,u16), bPos: &VecDeque<(u16,u16)>, gSize: (u16, u16)) -> bool
{
    for i in bPos
    {
        if hpos == *i
        {
            return true;
        }
    }

    return hpos.0 < 1 || hpos.1 < 1 || hpos.0 >= gSize.0 - 1 || hpos.1 >= gSize.1 - 1;
}

fn moveCursorTo(pos: (u16,u16))
{
    let _ = execute!(io::stdout(), cursor::MoveTo(pos.0, pos.1));
}

fn drawBoard(termSize: (u16, u16))
{
    let mut out = io::stdout();
    let _ = execute!(out, cursor::Hide, cursor::MoveTo(0,0));
    print!("┌");
    for _i in 1..(termSize.0 - 1)
    {
        print!("─");
    }
    print!("┐");

    for _i in 1..(termSize.1 - 1)
    {
        execute!(out, cursor::MoveToNextLine(1));
        print!("│");
        execute!(out, cursor::MoveToColumn(termSize.0 - 1));
        print!("│");
    }

    execute!(out, cursor::MoveToNextLine(1));
    write!(out, "└");
    for _i in 1..(termSize.0 - 1)
    {
        print!("─");
    }
    print!("┘");
    out.flush();
}

fn startGame() -> (u16, u16)
{
    let _ = execute!(io::stdout(), terminal::EnterAlternateScreen, event::EnableMouseCapture);
    let _ = terminal::enable_raw_mode();

    let t = terminal::size().unwrap();
    drawBoard(t);
    return t;
}

fn endGame()
{
    let _ = terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), terminal::LeaveAlternateScreen, event::DisableMouseCapture);
}
