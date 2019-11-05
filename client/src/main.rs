use fetris_protocol::{
    game::Direction, game::Input, game::TetriminoType, ClientRequest, ServerRequest,
};
use ncurses::*;
use std::env;
use std::io::Write;
use std::net::TcpStream;
use std::{thread, time};

const ORANGE: i16 = 55;

const PAIR_CYAN: i16 = 1;
const PAIR_BLUE: i16 = 2;
const PAIR_ORANGE: i16 = 3;
const PAIR_YELLOW: i16 = 4;
const PAIR_GREEN: i16 = 5;
const PAIR_MAGENTA: i16 = 6;
const PAIR_RED: i16 = 7;
const PAIR_WHITE: i16 = 8;

fn main() -> Result<(), std::io::Error> {
    if env::args().len() != 2 {
        println!("Usage: fetris server_address");
        return Ok(());
    }
    let mut stream = TcpStream::connect(env::args().nth(1).unwrap())?;
    initscr();
    start_color();

    cbreak();
    noecho();
    keypad(stdscr(), true);
    init_color(ORANGE, 1000, 680, 0);
    init_pair(PAIR_CYAN, constants::COLOR_WHITE, constants::COLOR_CYAN);
    init_pair(PAIR_BLUE, constants::COLOR_WHITE, constants::COLOR_BLUE);
    init_pair(PAIR_ORANGE, constants::COLOR_WHITE, ORANGE);
    init_pair(PAIR_YELLOW, constants::COLOR_WHITE, constants::COLOR_YELLOW);
    init_pair(PAIR_GREEN, constants::COLOR_WHITE, constants::COLOR_GREEN);
    init_pair(
        PAIR_MAGENTA,
        constants::COLOR_WHITE,
        constants::COLOR_MAGENTA,
    );
    init_pair(PAIR_RED, constants::COLOR_WHITE, constants::COLOR_RED);
    init_pair(PAIR_WHITE, constants::COLOR_WHITE, constants::COLOR_WHITE);

    let reader = stream.try_clone().unwrap();
    thread::spawn(move || {
        printw("Waiting for other players ...");
        loop {
            if let Ok(request) = ServerRequest::from_reader(&reader) {
                clear();
                if let ServerRequest::Action(_, game) = request {
                    let matrix = game.matrix();
                    let tetrimino = game.current_tetrimino();
                    let prediction =
                        if let Some(mut tetrimino_prediction) = game.current_tetrimino().clone() {
                            while tetrimino_prediction.can_move_to(&matrix, Direction::Down) {
                                tetrimino_prediction.apply_direction(Direction::Down);
                            }
                            Some(tetrimino_prediction)
                        } else {
                            None
                        };

                    for y in 0..22 {
                        let y = 21 - y;
                        for x in 0..10 {
                            if matrix[y][x] != None
                                || (tetrimino.is_some()
                                    && tetrimino.unwrap().check_position(x as i8, y as i8))
                            {
                                let ttype = if let Some(ttype) = matrix[y][x] {
                                    ttype
                                } else {
                                    tetrimino.unwrap().ttype()
                                };

                                let color = match ttype {
                                    TetriminoType::I => PAIR_CYAN,
                                    TetriminoType::J => PAIR_BLUE,
                                    TetriminoType::L => PAIR_ORANGE,
                                    TetriminoType::O => PAIR_YELLOW,
                                    TetriminoType::S => PAIR_GREEN,
                                    TetriminoType::T => PAIR_MAGENTA,
                                    TetriminoType::Z => PAIR_RED,
                                    TetriminoType::None => PAIR_WHITE,
                                };
                                attron(COLOR_PAIR(color));
                                printw("  ");
                                attroff(COLOR_PAIR(color));
                            } else if prediction.is_some()
                                && prediction.unwrap().check_position(x as i8, y as i8)
                            {
                                attron(COLOR_PAIR(PAIR_WHITE));
                                printw("  ");
                                attroff(COLOR_PAIR(PAIR_WHITE));
                            } else {
                                printw("  ");
                            }
                        }
                        printw("|\n");
                    }
                    printw(&format!(
                        "{:?}\n________________________________________________\n\n",
                        tetrimino
                    ));
                    refresh();
                }
            } else {
                break;
            }
        }
        println!("Invalid package");
        reader.shutdown(std::net::Shutdown::Both);
    });

    stream.write(&ClientRequest::AskForAGame.into_bytes());
    loop {
        match getch() {
            constants::KEY_LEFT => {
                stream.write(&ClientRequest::Input(Input::Left).into_bytes());
                printw("LEFT");
            }
            constants::KEY_RIGHT => {
                stream.write(&ClientRequest::Input(Input::Right).into_bytes());
                printw("RIGHT");
            }
            constants::KEY_DOWN => {
                stream.write(&ClientRequest::Input(Input::FastMove).into_bytes());
                printw("FAST");
            }
            10 => {
                stream.write(&ClientRequest::Input(Input::Rotate).into_bytes());
                printw("ROTATE");
            }
            _ => {}
        };

        if stream.peer_addr().is_err() {
            break;
        }
    }
    endwin();
    Ok(())
}
