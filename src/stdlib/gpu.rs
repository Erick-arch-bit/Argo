use std::collections::HashMap;
use std::process::Command;

use crate::evaluator::{LlaveHash, Objeto};

fn extraer_entero(args: &[Objeto], idx: usize, nombre: &str) -> Result<i64, Objeto> {
    match args.get(idx) {
        Some(Objeto::Entero(v)) => Ok(*v),
        Some(other) => Err(Objeto::Error(format!(
            "gpu: '{}' debe ser un entero, se recibió {}",
            nombre,
            match other {
                Objeto::Entero(_) => "entero",
                Objeto::Flotante(_) => "flotante",
                Objeto::Booleano(_) => "booleano",
                Objeto::Cadena(_) => "cadena",
                Objeto::Nulo => "nulo",
                Objeto::Buffer(_) => "buffer",
                Objeto::Arreglo(_) => "arreglo",
                Objeto::Diccionario(_) => "diccionario",
                Objeto::Funcion { .. } | Objeto::Nativa(_) => "función",
                Objeto::Retorno(_) => "retorno",
                Objeto::StructDef(_) => "struct_def",
                Objeto::Instancia { .. } => "instancia",
                Objeto::Break => "break",
                Objeto::Continue => "continue",
                Objeto::Error(_, _) => "error",
                Objeto::Canal(_) => "canal",
                Objeto::Excepcion(_) => "excepcion",
            }
        ), Vec::new())),
        None => Err(Objeto::Error(format!(
            "gpu: falta el argumento '{}'", nombre
        ), Vec::new())),
    }
}

fn extraer_framebuffer(args: &[Objeto]) -> Result<(Vec<u8>, usize, usize), Objeto> {
    let fb = match args.first() {
        Some(Objeto::Diccionario(d)) => d,
        Some(other) => return Err(Objeto::Error(format!(
            "gpu: se esperaba un framebuffer (diccionario), se recibió {}",
            match other {
                Objeto::Entero(_) => "entero", Objeto::Flotante(_) => "flotante",
                Objeto::Booleano(_) => "booleano", Objeto::Cadena(_) => "cadena",
                Objeto::Nulo => "nulo", Objeto::Buffer(_) => "buffer",
                Objeto::Arreglo(_) => "arreglo", Objeto::Diccionario(_) => "diccionario",
                Objeto::Funcion { .. } | Objeto::Nativa(_) => "función",
                Objeto::Retorno(_) => "retorno", Objeto::StructDef(_) => "struct_def",
                Objeto::Instancia { .. } => "instancia", Objeto::Break => "break",
                Objeto::Continue => "continue", Objeto::Error(_, _) => "error",
                Objeto::Canal(_) => "canal",
                Objeto::Excepcion(_) => "excepcion",
            }
        ), Vec::new())),
        None => return Err(Objeto::Error("gpu: falta el argumento framebuffer".to_string(), Vec::new())),
    };

    let buffer = match fb.get(&LlaveHash::Cadena("buffer".to_string())) {
        Some(Objeto::Buffer(b)) => b.clone(),
        Some(other) => return Err(Objeto::Error(format!(
            "gpu: framebuffer inválido: 'buffer' debe ser un buffer, se recibió {}",
            match other {
                Objeto::Entero(_) => "entero", Objeto::Flotante(_) => "flotante",
                Objeto::Booleano(_) => "booleano", Objeto::Cadena(_) => "cadena",
                Objeto::Nulo => "nulo", Objeto::Buffer(_) => "buffer",
                Objeto::Arreglo(_) => "arreglo", Objeto::Diccionario(_) => "diccionario",
                Objeto::Funcion { .. } | Objeto::Nativa(_) => "función",
                Objeto::Retorno(_) => "retorno", Objeto::StructDef(_) => "struct_def",
                Objeto::Instancia { .. } => "instancia", Objeto::Break => "break",
                Objeto::Continue => "continue", Objeto::Error(_, _) => "error",
                Objeto::Canal(_) => "canal",
                Objeto::Excepcion(_) => "excepcion",
            }
        ), Vec::new())),
        None => return Err(Objeto::Error("gpu: framebuffer no contiene 'buffer'".to_string(), Vec::new())),
    };
    let ancho = match fb.get(&LlaveHash::Cadena("ancho".to_string())) {
        Some(Objeto::Entero(w)) => *w as usize,
        _ => return Err(Objeto::Error("gpu: framebuffer inválido: 'ancho' debe ser un entero".to_string(), Vec::new())),
    };
    let alto = match fb.get(&LlaveHash::Cadena("alto".to_string())) {
        Some(Objeto::Entero(h)) => *h as usize,
        _ => return Err(Objeto::Error("gpu: framebuffer inválido: 'alto' debe ser un entero".to_string(), Vec::new())),
    };
    if buffer.len() < ancho * alto * 4 {
        return Err(Objeto::Error("gpu: framebuffer: tamaño de datos insuficiente".to_string(), Vec::new()));
    }
    Ok((buffer, ancho, alto))
}

fn reconstruir_fb(buffer: Vec<u8>, ancho: usize, alto: usize) -> Objeto {
    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();
    mapa.insert(LlaveHash::Cadena("buffer".to_string()), Objeto::Buffer(buffer));
    mapa.insert(LlaveHash::Cadena("ancho".to_string()), Objeto::Entero(ancho as i64));
    mapa.insert(LlaveHash::Cadena("alto".to_string()), Objeto::Entero(alto as i64));
    Objeto::Diccionario(mapa)
}

fn coord_valida(x: i64, y: i64, ancho: usize, alto: usize) -> bool {
    x >= 0 && (x as usize) < ancho && y >= 0 && (y as usize) < alto
}

fn indice_pixel(x: i64, y: i64, ancho: usize) -> usize {
    ((y as usize) * ancho + (x as usize)) * 4
}

fn saturar(v: i64) -> u8 {
    if v < 0 { 0 } else if v > 255 { 255 } else { v as u8 }
}

pub fn crear_modulo() -> Objeto {
    fn gpu_crear_buffer(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(format!(
                "gpu.crear_buffer: se esperaban 2 argumentos (ancho, alto), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let ancho = match extraer_entero(&args, 0, "ancho") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let alto = match extraer_entero(&args, 1, "alto") {
            Ok(v) => v,
            Err(e) => return e,
        };
        if ancho <= 0 || alto <= 0 {
            return Objeto::Error(format!(
                "gpu.crear_buffer: ancho y alto deben ser positivos, se recibió {}x{}",
                ancho, alto
            ), Vec::new());
        }
        let (w, h) = (ancho as usize, alto as usize);
        let tamano = w * h * 4;
        if tamano > 256 * 1024 * 1024 {
            return Objeto::Error(format!(
                "gpu.crear_buffer: el framebuffer {}x{} es demasiado grande ({} MB)",
                w, h, tamano / (1024 * 1024)
            ), Vec::new());
        }
        reconstruir_fb(vec![0; tamano], w, h)
    }

    fn gpu_pixel(args: Vec<Objeto>) -> Objeto {
        if args.len() != 6 {
            return Objeto::Error(format!(
                "gpu.pixel: se esperaban 6 argumentos (fb, x, y, r, g, b), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let (mut buf, w, h) = match extraer_framebuffer(&args) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let x = match extraer_entero(&args, 1, "x") { Ok(v) => v, Err(e) => return e };
        let y = match extraer_entero(&args, 2, "y") { Ok(v) => v, Err(e) => return e };
        let r = match extraer_entero(&args, 3, "r") { Ok(v) => v, Err(e) => return e };
        let g = match extraer_entero(&args, 4, "g") { Ok(v) => v, Err(e) => return e };
        let b = match extraer_entero(&args, 5, "b") { Ok(v) => v, Err(e) => return e };

        if !coord_valida(x, y, w, h) {
            return reconstruir_fb(buf, w, h);
        }
        let idx = indice_pixel(x, y, w);
        buf[idx] = saturar(r);
        buf[idx + 1] = saturar(g);
        buf[idx + 2] = saturar(b);
        buf[idx + 3] = 255;
        reconstruir_fb(buf, w, h)
    }

    fn gpu_linea(args: Vec<Objeto>) -> Objeto {
        if args.len() != 8 {
            return Objeto::Error(format!(
                "gpu.linea: se esperaban 8 argumentos (fb, x1, y1, x2, y2, r, g, b), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let (mut buf, w, h) = match extraer_framebuffer(&args) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let x1 = match extraer_entero(&args, 1, "x1") { Ok(v) => v, Err(e) => return e };
        let y1 = match extraer_entero(&args, 2, "y1") { Ok(v) => v, Err(e) => return e };
        let x2 = match extraer_entero(&args, 3, "x2") { Ok(v) => v, Err(e) => return e };
        let y2 = match extraer_entero(&args, 4, "y2") { Ok(v) => v, Err(e) => return e };
        let r = saturar(match extraer_entero(&args, 5, "r") { Ok(v) => v, Err(e) => return e });
        let g = saturar(match extraer_entero(&args, 6, "g") { Ok(v) => v, Err(e) => return e });
        let b = saturar(match extraer_entero(&args, 7, "b") { Ok(v) => v, Err(e) => return e });

        let mut x = x1;
        let mut y = y1;
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if coord_valida(x, y, w, h) {
                let idx = indice_pixel(x, y, w);
                buf[idx] = r;
                buf[idx + 1] = g;
                buf[idx + 2] = b;
                buf[idx + 3] = 255;
            }
            if x == x2 && y == y2 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx; }
            if e2 <= dx { err += dx; y += sy; }
        }
        reconstruir_fb(buf, w, h)
    }

    fn gpu_rect(args: Vec<Objeto>) -> Objeto {
        if args.len() != 8 {
            return Objeto::Error(format!(
                "gpu.rect: se esperaban 8 argumentos (fb, x, y, ancho, alto, r, g, b), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let (mut buf, w, h) = match extraer_framebuffer(&args) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let rx = match extraer_entero(&args, 1, "x") { Ok(v) => v, Err(e) => return e };
        let ry = match extraer_entero(&args, 2, "y") { Ok(v) => v, Err(e) => return e };
        let rw = match extraer_entero(&args, 3, "ancho") { Ok(v) => v, Err(e) => return e };
        let rh = match extraer_entero(&args, 4, "alto") { Ok(v) => v, Err(e) => return e };
        let r_color = saturar(match extraer_entero(&args, 5, "r") { Ok(v) => v, Err(e) => return e });
        let g_color = saturar(match extraer_entero(&args, 6, "g") { Ok(v) => v, Err(e) => return e });
        let b_color = saturar(match extraer_entero(&args, 7, "b") { Ok(v) => v, Err(e) => return e });

        let (x_min, x_max) = (rx.max(0), (rx + rw).min(w as i64));
        let (y_min, y_max) = (ry.max(0), (ry + rh).min(h as i64));

        for py in y_min..y_max {
            let fila_start = indice_pixel(x_min, py, w);
            let fila_end = indice_pixel(x_max, py, w);
            for idx in (fila_start..fila_end).step_by(4) {
                buf[idx] = r_color;
                buf[idx + 1] = g_color;
                buf[idx + 2] = b_color;
                buf[idx + 3] = 255;
            }
        }
        reconstruir_fb(buf, w, h)
    }

    fn gpu_circulo(args: Vec<Objeto>) -> Objeto {
        if args.len() != 7 {
            return Objeto::Error(format!(
                "gpu.circulo: se esperaban 7 argumentos (fb, cx, cy, radio, r, g, b), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let (mut buf, w, h) = match extraer_framebuffer(&args) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let cx = match extraer_entero(&args, 1, "cx") { Ok(v) => v, Err(e) => return e };
        let cy = match extraer_entero(&args, 2, "cy") { Ok(v) => v, Err(e) => return e };
        let radio = match extraer_entero(&args, 3, "radio") { Ok(v) => v, Err(e) => return e };
        let r_color = saturar(match extraer_entero(&args, 4, "r") { Ok(v) => v, Err(e) => return e });
        let g_color = saturar(match extraer_entero(&args, 5, "g") { Ok(v) => v, Err(e) => return e });
        let b_color = saturar(match extraer_entero(&args, 6, "b") { Ok(v) => v, Err(e) => return e });

        if radio < 0 { return reconstruir_fb(buf, w, h); }

        let y_min = (cy - radio).max(0) as usize;
        let y_max = (cy + radio + 1).min(h as i64) as usize;
        let w_usize = w;

        for py in y_min..y_max {
            let dy = py as i64 - cy;
            let half_chord = ((radio * radio - dy * dy) as f64).sqrt() as i64;
            let x_start = (cx - half_chord).max(0) as usize;
            let x_end = (cx + half_chord + 1).min(w_usize as i64) as usize;
            let fila_start = (py * w_usize + x_start) * 4;
            let fila_end = (py * w_usize + x_end) * 4;
            for idx in (fila_start..fila_end).step_by(4) {
                buf[idx] = r_color;
                buf[idx + 1] = g_color;
                buf[idx + 2] = b_color;
                buf[idx + 3] = 255;
            }
        }
        reconstruir_fb(buf, w, h)
    }

    fn gpu_guardar(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(format!(
                "gpu.guardar: se esperaban 2 argumentos (fb, ruta), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let (buf, w, h) = match extraer_framebuffer(&args) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let ruta = match &args[1] {
            Objeto::Cadena(s) => s.clone(),
            other => return Objeto::Error(format!(
                "gpu.guardar: la ruta debe ser una cadena, se recibió {}",
                match other {
                    Objeto::Entero(_) => "entero", Objeto::Flotante(_) => "flotante",
                    Objeto::Booleano(_) => "booleano", Objeto::Cadena(_) => "cadena",
                    Objeto::Nulo => "nulo", Objeto::Buffer(_) => "buffer",
                    Objeto::Arreglo(_) => "arreglo", Objeto::Diccionario(_) => "diccionario",
                    Objeto::Funcion { .. } | Objeto::Nativa(_) => "función",
                    Objeto::Retorno(_) => "retorno", Objeto::StructDef(_) => "struct_def",
                    Objeto::Instancia { .. } => "instancia", Objeto::Break => "break",
                    Objeto::Continue => "continue", Objeto::Error(_, _) => "error",
                    Objeto::Canal(_) => "canal",
                    Objeto::Excepcion(_) => "excepcion",
                }
            ), Vec::new()),
        };

        let mut ppm = Vec::with_capacity(w * h * 3 + 100);
        ppm.extend_from_slice(format!("P6\n{} {}\n255\n", w, h).as_bytes());
        for py in 0..h {
            for px in 0..w {
                let idx = (py * w + px) * 4;
                ppm.push(buf[idx]);
                ppm.push(buf[idx + 1]);
                ppm.push(buf[idx + 2]);
            }
        }
        match std::fs::write(&ruta, &ppm) {
            Ok(_) => Objeto::Booleano(true),
            Err(e) => Objeto::Error(format!("gpu.guardar: error al escribir '{}': {}", ruta, e), Vec::new()),
        }
    }

    fn gpu_mostrar(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(format!(
                "gpu.mostrar: se esperaba 1 argumento (fb), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let (buf, w, h) = match extraer_framebuffer(&args) {
            Ok(v) => v,
            Err(e) => return e,
        };

        let ruta_tmp = format!("/tmp/argo_gpu_{}.ppm", std::process::id());
        let mut ppm = Vec::with_capacity(w * h * 3 + 100);
        ppm.extend_from_slice(format!("P6\n{} {}\n255\n", w, h).as_bytes());
        for py in 0..h {
            for px in 0..w {
                let idx = (py * w + px) * 4;
                ppm.push(buf[idx]);
                ppm.push(buf[idx + 1]);
                ppm.push(buf[idx + 2]);
            }
        }
        if std::fs::write(&ruta_tmp, &ppm).is_err() {
            return Objeto::Error("gpu.mostrar: error al escribir archivo temporal".to_string(), Vec::new());
        }

        let resultado = if cfg!(target_os = "macos") {
            Command::new("open").arg(&ruta_tmp).output()
        } else if cfg!(target_os = "linux") {
            let viewers = ["xdg-open", "feh", "display", "eog", "gimp"];
            let mut res = Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no viewer"));
            for v in &viewers {
                res = Command::new(v).arg(&ruta_tmp).output();
                if res.is_ok() { break; }
            }
            res
        } else {
            Command::new("cmd").args(["/c", "start", ""]).arg(&ruta_tmp).output()
        };

        match resultado {
            Ok(_) => Objeto::Cadena(ruta_tmp),
            Err(_) => Objeto::Cadena(format!("archivo guardado en: {}", ruta_tmp)),
        }
    }

    fn gpu_limpiar(args: Vec<Objeto>) -> Objeto {
        if args.len() != 4 {
            return Objeto::Error(format!(
                "gpu.limpiar: se esperaban 4 argumentos (fb, r, g, b), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let (mut buf, w, h) = match extraer_framebuffer(&args) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let r = saturar(match extraer_entero(&args, 1, "r") { Ok(v) => v, Err(e) => return e });
        let g = saturar(match extraer_entero(&args, 2, "g") { Ok(v) => v, Err(e) => return e });
        let b = saturar(match extraer_entero(&args, 3, "b") { Ok(v) => v, Err(e) => return e });

        for chunk in buf.chunks_exact_mut(4) {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = 255;
        }
        reconstruir_fb(buf, w, h)
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();
    mapa.insert(LlaveHash::Cadena("crear_buffer".to_string()), Objeto::Nativa(gpu_crear_buffer as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("pixel".to_string()), Objeto::Nativa(gpu_pixel as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("linea".to_string()), Objeto::Nativa(gpu_linea as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("rect".to_string()), Objeto::Nativa(gpu_rect as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("circulo".to_string()), Objeto::Nativa(gpu_circulo as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("guardar".to_string()), Objeto::Nativa(gpu_guardar as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("mostrar".to_string()), Objeto::Nativa(gpu_mostrar as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("limpiar".to_string()), Objeto::Nativa(gpu_limpiar as fn(Vec<Objeto>) -> Objeto));
    Objeto::Diccionario(mapa)
}
