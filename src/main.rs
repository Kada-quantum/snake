use std::io;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use rand::prelude::*;

mod snake;
use snake::{Directions, Food, Snake};

fn get_inputs(
    exit: Arc<AtomicBool>,
) -> (mpsc::Receiver<[u8; 3]>, thread::JoinHandle<io::Result<()>>) {
    let (sen, rec) = mpsc::sync_channel(1);
    let handle = thread::spawn(move || {
        let mut buf: [u8; 3];
        while !exit.load(Ordering::Relaxed) {
            buf = getch::getch()?;
            sen.send(buf).unwrap();
        }
        Ok(())
    });
    (rec, handle)
}

fn render(
    t_size: (usize, usize),
    food: Food,
    snake: Snake,
    exit: Arc<AtomicBool>,
) -> io::Result<()> {
    if !exit.load(Ordering::Relaxed) {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        let mut texture = vec![vec![b' '; t_size.0]; t_size.1];
        texture[0] = vec![b'#'; t_size.0];
        for row in texture.iter_mut() {
            row[0] = b'#';
            row[t_size.0 - 1] = b'#';
        }
        texture[t_size.1 - 1] = vec![b'#'; t_size.0];
        for (x, y) in snake.into_iter() {
            texture[y.wrapping_sub(1)][x.wrapping_sub(1)] = b'O';
        }
        if let Food::Exists(x, y) = food {
            texture[y.wrapping_sub(1)][x.wrapping_sub(1)] = b'%';
        }
        let out = io::stdout();
        let mut lock = out.lock();
        for line in &texture {
            lock.write_all(line.as_slice())?;
        }
        lock.flush()?;
    } else {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("length of the snake: {}", snake.len());
        println!("press any key to exit...");
        io::stdout().flush()?;
    }
    Ok(())
}

fn generate_food(t_size: (usize, usize)) -> Food {
    thread::spawn(move || {
        let mut rng = thread_rng();
        Food::Exists(rng.gen_range(2..t_size.0), rng.gen_range(2..t_size.1))
    })
    .join()
    .unwrap()
}

fn clock(snake: Arc<Mutex<Snake>>, exit: Arc<AtomicBool>) -> thread::JoinHandle<io::Result<()>> {
    thread::spawn(move || {
        let mut food = Food::Eaten;
        while !exit.load(Ordering::Relaxed) {
            let (w, h) = terminal_size::terminal_size().unwrap();
            let t_size = (w.0 as usize, h.0 as usize);
            if food == Food::Eaten {
                food = generate_food(t_size);
            }
            (*snake.lock().unwrap()).move_heads(&mut food, t_size, exit.clone());
            render(t_size, food, (*snake.lock().unwrap()).clone(), exit.clone())?;
            thread::sleep(std::time::Duration::new(0, 50000000));
        }
        Ok(())
    })
}

fn main() -> io::Result<()> {
    let exit = Arc::new(AtomicBool::new(false));
    let snake = Arc::new(Mutex::new(Snake::new()));
    let clock_handle = clock(snake.clone(), exit.clone());
    let (inputs, in_handle) = get_inputs(exit.clone());
    for input in inputs {
        match input {
            [119, 0, 0] => (*snake.lock().unwrap()).change_dir(Directions::Up), // 'w'
            [87, 0, 0] => (*snake.lock().unwrap()).change_dir(Directions::Up),  // 'W'
            [27, 91, 65] => (*snake.lock().unwrap()).change_dir(Directions::Up), // 'up arrow'
            [115, 0, 0] => (*snake.lock().unwrap()).change_dir(Directions::Down), // 's'
            [83, 0, 0] => (*snake.lock().unwrap()).change_dir(Directions::Down), // 'S'
            [27, 91, 66] => (*snake.lock().unwrap()).change_dir(Directions::Down), // 'down arrow'
            [100, 0, 0] => (*snake.lock().unwrap()).change_dir(Directions::Right), // 'd'
            [68, 0, 0] => (*snake.lock().unwrap()).change_dir(Directions::Right), // 'D'
            [27, 91, 67] => (*snake.lock().unwrap()).change_dir(Directions::Right), // 'right arrow'
            [97, 0, 0] => (*snake.lock().unwrap()).change_dir(Directions::Left), // 'a'
            [65, 0, 0] => (*snake.lock().unwrap()).change_dir(Directions::Left), // 'A'
            [27, 91, 68] => (*snake.lock().unwrap()).change_dir(Directions::Left), // 'left arrow'
            [113, 0, 0] => {
                exit.swap(true, Ordering::Relaxed);
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                println!("length of the snake: {}", (*snake.lock().unwrap()).len());
                println!("press any key to exit...");
                io::stdout().flush()?;
            }
            [27, 0, 0] => {
                exit.swap(true, Ordering::Relaxed);
                print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
                println!("length of the snake: {}", (*snake.lock().unwrap()).len());
                println!("press any key to exit...");
                io::stdout().flush()?;
            }
            _ => (),
        }
    }
    thread::sleep(std::time::Duration::new(2, 0));
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    io::stdout().flush()?;
    in_handle.join().unwrap()?;
    clock_handle.join().unwrap()?;
    Ok(())
}
