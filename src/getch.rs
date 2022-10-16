use std::io;
use std::io::Read;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

pub fn getch() -> io::Result<[u8; 3]> {
    let tio = Termios::from_fd(0)?;
    let mut tmp_tio = tio.clone();
    tmp_tio.c_lflag &= !(ICANON | ECHO);
    tcsetattr(0, TCSANOW, &mut tmp_tio)?;
    let mut reader = io::stdin();
    let mut buf = [0; 3];
    reader.read(&mut buf)?;
    tcsetattr(0, TCSANOW, &tio)?;
    Ok(buf)
}

#[cfg(test)]
mod test_ch {
    use super::*;
    #[test]
    fn get_chars() {
        println!("{:?}", getch().unwrap());
    }
}
