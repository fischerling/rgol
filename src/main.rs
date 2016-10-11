/*
* Copyright (c) 2016, Florian Fischer
*
* This file is licensed under the conditions found in the LICENSE file in the
* projects root.
*/

extern crate termion;

use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor, terminal_size};
use std::io::{Read, Write, stdout, stdin};
use std::thread::sleep;
use std::time::Duration;

extern crate game_of_live;

use game_of_live::game;

const GREETING: [&'static str; 10] = [
"Welcome to the RGoL!",
"- Game of Live written in pure rust -",
"",
"[p]lay the game",
"[s]elect the grid size",
"[c]hange configuration",
"[l]oad a configuration from file",
"[i]nteractive - step by step mode",
"[a]xis - show x and y axes",
"[q] exits the current screen or the game"];

struct TUI<W: Write, R: Read>{
    stdout: W,
    stdin: R,
    term_size: (u16, u16),
    game: game::Game,
    step_by_step: bool,
    show_axis: bool,
}

impl<W: Write, R: Read> Drop for TUI<W, R> {
    fn drop(&mut self) {
        write!(self.stdout, "{}{}{}", cursor::Goto(1,1), clear::All, cursor::Show).unwrap();
    }
}

impl<W: Write, R: Read> TUI<W, R> {
    fn new(stdout: W, stdin: R) -> TUI<W, R> {
        let term_size = terminal_size().unwrap();
        TUI {
            stdout: stdout,
            stdin: stdin,
            term_size: term_size,
            game: game::Game::new(if term_size.0 > term_size.1 { (term_size.1/2 ) as usize}
                                  else { (term_size.0/2) as usize}),
            step_by_step: false,
            show_axis: false,
        }
    }

    fn draw_horisontal_line(&mut self, chr: &str, width: usize) {
        for _ in 0..width { write!(self.stdout, "{}", chr).unwrap(); }
    }

    fn plot_grid(&mut self, grid: &game::Grid, x:u16, y:u16) {

        let mut axis_space: u16 = 1;

        let corner_sign = "+";
        let size = grid.get_size();

        write!(self.stdout, "{}", clear::All).unwrap();

        // draw x axis
        if self.show_axis {
            axis_space = 4;
            for i in 1..(size/10)+1 {
                write!(self.stdout, "{}0", cursor::Goto((i*10+2) as u16,1)).unwrap();
            }
            write!(self.stdout, "{}{}{}-", cursor::Goto((size+1) as u16,1),
                                            size%10,
                                            cursor::Goto(1,2)).unwrap();
            for i in 0..(size/10)+1 {
                write!(self.stdout, "{}", i).unwrap();
                if size > (i+1)*10 {
                    self.draw_horisontal_line("-", 9);
                } else {
                    self.draw_horisontal_line("-", size-i*10-2)
                }
            }
            write!(self.stdout, "{}-> x", size/10).unwrap();
        }

        write!(self.stdout, "{}{}", cursor::Goto(1, axis_space), corner_sign).unwrap();
        self.draw_horisontal_line("-", size);
        write!(self.stdout, "{}", corner_sign).unwrap();
        if self.show_axis {
            write!(self.stdout, " |").unwrap();
        }

        for y in 0..size {
            write!(self.stdout, "{}|", cursor::Goto(1,((y+1) as u16)+axis_space)).unwrap();
            for x in 0..size {
                if grid.is_field_alive(x,y) {
                    write!(self.stdout, "#").unwrap();
                } else {
                    write!(self.stdout, " ").unwrap();
                }
            }
            if self.show_axis {
                if y%10 == 0 {
                    write!(self.stdout, "| {}", y).unwrap();
                } else {
                    write!(self.stdout, "| |").unwrap();
                }
            } else {
                write!(self.stdout, "|").unwrap();
            }
        }
        // write size at y axis
        if self.show_axis {
            write!(self.stdout, "{}{}{}|{}v y",
                cursor::Goto((size as u16)+4, (size as u16)+axis_space),
                size,
                cursor::Goto((size as u16)+4, (size as u16)+axis_space+1),
                cursor::Goto((size as u16)+4, (size as u16)+axis_space+2)).unwrap();
        }
        write!(self.stdout, "{}{}",
            cursor::Goto(1, ((size+1) as u16)+axis_space),
            corner_sign).unwrap();
            
        self.draw_horisontal_line("-", size);
        
        write!(self.stdout, "{}{}Generation: {}{}",
            corner_sign,
            cursor::Goto(1, ((size+2) as u16)+axis_space),
            grid.get_generation(),
            cursor::Goto(x,y)).unwrap();
            
        self.stdout.flush().unwrap();
    }

    fn play(&mut self) {

        let mut gen: game::Grid;
        
        let mut buf = [0];

        //let mut stdin = async_stdin();

        loop {
            gen = self.game.next().unwrap();
            self.plot_grid(&gen, 1, 1);
            self.stdin.read(&mut buf).unwrap();
            match buf[0] {
                b'q' => break,
                _    => continue,
            }
        }
    }
    
    fn new_size(&mut self) {
        let mut buf = [0];
        let new_size;
        let mut v: Vec<u8> = Vec::new();
        loop {
            write!(self.stdout, "{}{}Enter new size: ", clear::All, cursor::Goto(1,1)).unwrap();
            self.stdout.flush().unwrap();
            
            loop {
                self.stdin.read(&mut buf).unwrap();
                match buf[0] {
                    b'0' ... b'9'   => { v.push(buf[0]);
                                        write!(self.stdout, "{}", buf[0] as char).unwrap(); },
                    _               => break,
                }
                self.stdout.flush().unwrap();
            }

            match String::from_utf8(v.clone()).unwrap().parse::<usize>() {
                Ok(num) => { new_size = num; },
                Err(_) => continue,
            }
            self.game.resize(new_size);
            break;
        }
    }

    fn new_configuration(&mut self) {
        write!(self.stdout, "{}{}{}", clear::All, cursor::Goto(1,1), cursor::Show).unwrap();
        
        let mut buf = [0];
        let size = self.game.get_board().get_size();
        let mut axis_space = 0;
            if self.show_axis { axis_space = 3; }
        let mut x = 2;
        let mut y = 2 + axis_space;
        loop {
            
            let grid = self.game.get_board();
            self.plot_grid(&grid, x as u16, y as u16);
            
            self.stdin.read(&mut buf).unwrap();
            
            match buf[0] {
                b'h' | b'a' => x = if x > 2 { x-1 } else { 2 },
                b'j' | b's' => y = if y < size+1+axis_space { y+1 } else { size+1+axis_space },
                b'k' | b'w' => y = if y > 2 { y-1 } else { 2 },
                b'l' | b'd' => x = if x < size+1 { x+1 } else { size+1 },
                b' ' => { self.game.toggle_field(x-2,y-2-axis_space); },
                b'q' => break,
                _ => (),
            }
            println!("");
            write!(self.stdout, "{}", cursor::Goto(x as u16, y as u16)).unwrap();
            self.stdout.flush().unwrap();

        }
        write!(self.stdout, "{}", cursor::Hide).unwrap();
    }

    fn load_configuration(&mut self) {}

    fn toggle_interactive(&mut self) {
        self.step_by_step = !self.step_by_step;
    }
    fn toggle_show_axis(&mut self) {
        self.show_axis = !self.show_axis;
    }

    // insert glider for testing
    fn glider(&mut self) {
        self.game.toggle_field(1,0);
        self.game.toggle_field(2,1);
        self.game.toggle_field(0,2);
        self.game.toggle_field(1,2);
        self.game.toggle_field(2,2);
    }

    fn menu(&mut self) {
        let mut choice = [0];
        loop {
            write!(self.stdout, "{}{}{}", clear::All, cursor::Goto(1,1), cursor::Hide).unwrap();
            for (i, l) in GREETING.into_iter().enumerate() {
                write!(self.stdout, "{}{}",
                    cursor::Goto((self.term_size.0 as u16)/2-(l.len() as u16)/2, (i+1) as u16),
                    l.to_string()).unwrap();
            }
            self.stdout.flush().unwrap();

            self.stdin.read(&mut choice).unwrap();

            match choice[0] {
                b'p' => self.play(),
                b's' => self.new_size(),
                b'c' => self.new_configuration(),
                b'l' => self.load_configuration(),
                b'i' => self.toggle_interactive(),
                b'a' => self.toggle_show_axis(),
                b't' => self.glider(),
                b'q' => return,
                _ => {},
            }
        }
    }

}

fn main() {

    let stdout = stdout();
    let stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();

    let mut tui = TUI::new(stdout, stdin);

    tui.menu();
    
    // blinker
    //game.toggle_field(0,1);
    //game.toggle_field(1,1);
    //game.toggle_field(2,1);

}
