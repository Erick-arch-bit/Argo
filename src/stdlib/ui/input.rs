use std::io::Read;

pub fn leer_tecla() -> &'static str {
    let mut stdin = std::io::stdin();
    let mut buffer = [0u8; 1];

    match stdin.read(&mut buffer) {
        Ok(0) => return "q",
        Ok(_) => {}
        Err(_) => return "q",
    }

    match buffer[0] {
        13 => "Enter",
        27 => {
            let mut seq = [0u8; 2];
            let _ = stdin.read(&mut seq);
            match seq {
                [91, 65] => "ArrowUp",
                [91, 66] => "ArrowDown",
                [91, 67] => "ArrowRight",
                [91, 68] => "ArrowLeft",
                _ => "Escape",
            }
        }
        127 => "Backspace",
        9 => "Tab",
        b'q' | b'Q' => "q",
        _ => "q",
    }
}
