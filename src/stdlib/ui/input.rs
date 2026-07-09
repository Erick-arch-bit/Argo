use std::io::Read;

pub fn leer_tecla() -> String {
    let mut stdin = std::io::stdin();
    let mut buffer = [0u8; 1];

    // Leer un byte
    match stdin.read(&mut buffer) {
        Ok(0) => return "q".to_string(),
        Ok(_) => {}
        Err(_) => return "q".to_string(),
    }

    let byte = buffer[0];

    match byte {
        // Enter
        13 => "Enter".to_string(),
        // Escape sequence (arrow keys, etc.)
        27 => {
            let mut seq = [0u8; 2];
            let _ = stdin.read(&mut seq);
            match seq {
                [91, 65] => "ArrowUp".to_string(),
                [91, 66] => "ArrowDown".to_string(),
                [91, 67] => "ArrowRight".to_string(),
                [91, 68] => "ArrowLeft".to_string(),
                _ => "Escape".to_string(),
            }
        }
        // Backspace
        127 => "Backspace".to_string(),
        // Tab
        9 => "Tab".to_string(),
        // Caracteres normales (q, space, etc.)
        _ => {
            if (32..=126).contains(&byte) {
                (byte as char).to_string()
            } else {
                "?".to_string()
            }
        }
    }
}
