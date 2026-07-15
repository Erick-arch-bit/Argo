#![allow(
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::trim_split_whitespace
)]

pub mod animaciones;
pub mod engine;
pub mod widgets;

use std::collections::HashMap;
use std::io::BufWriter;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use crate::evaluator::object::LlaveHash;
use crate::evaluator::Objeto;

use engine::MotorRenderizado;
use widgets::{parsear_color, ArbolUI, DatosWidget, TipoLayout};

// ---------------------------------------------------------------------------
// Estado global de la UI — Singleton thread-safe
// ---------------------------------------------------------------------------
struct UiGlobal {
    arbol: ArbolUI,
    engine: MotorRenderizado,
    animaciones: animaciones::MotorAnimaciones,
    primer_frame: bool,
}

static UI_STATE: OnceLock<Mutex<UiGlobal>> = OnceLock::new();

fn get_state() -> &'static Mutex<UiGlobal> {
    UI_STATE.get_or_init(|| {
        Mutex::new(UiGlobal {
            arbol: ArbolUI::new(),
            engine: MotorRenderizado::new(),
            animaciones: animaciones::MotorAnimaciones::new(),
            primer_frame: true,
        })
    })
}

// ---------------------------------------------------------------------------
// Input — Lectura de teclado sin bloqueo (non-blocking con ioctl FIONREAD)
// ---------------------------------------------------------------------------
fn leer_tecla_no_bloqueante() -> &'static str {
    use std::io::Read;

    #[cfg(unix)]
    {
        // Verificar si hay bytes disponibles en stdin usando ioctl FIONREAD
        let hay_datos = unsafe {
            let fd = 0i32; // stdin
            let mut nbytes: i32 = 0;
            // FIONREAD = 0x541B en Linux
            let ioctl_result = libc_ioctl(fd, 0x541B, &mut nbytes as *mut i32);
            ioctl_result == 0 && nbytes > 0
        };

        if !hay_datos {
            return "none";
        }

        let mut buffer = [0u8; 1];
        match std::io::stdin().read(&mut buffer) {
            Ok(0) => "none",
            Ok(_) => match buffer[0] {
                13 => "Enter",
                27 => {
                    let mut seq = [0u8; 2];
                    let _ = std::io::stdin().read(&mut seq);
                    match seq {
                        [91, 65] => "ArrowUp",
                        [91, 66] => "ArrowDown",
                        [91, 67] => "ArrowRight",
                        [91, 68] => "ArrowLeft",
                        _ => "Escape",
                    }
                }
                9 => "Tab",
                127 => "Backspace",
                b'q' | b'Q' => "q",
                _ => "none",
            },
            Err(_) => "none",
        }
    }

    #[cfg(not(unix))]
    {
        "none"
    }
}

/// Wrapper raw para ioctl sin depender del crate libc.
#[cfg(unix)]
unsafe fn libc_ioctl(fd: i32, request: i32, argp: *mut i32) -> i32 {
    unsafe extern "C" {
        fn ioctl(fd: i32, request: i32, ...) -> i32;
    }
    unsafe { ioctl(fd, request, argp) }
}

// ---------------------------------------------------------------------------
// Dibujar recursivo — Renderiza el árbol en el MotorRenderizado
// ---------------------------------------------------------------------------
fn dibujar_nodo(
    arbol: &ArbolUI,
    id: usize,
    engine: &mut MotorRenderizado,
    theme: &widgets::Theme,
    foco_id: Option<usize>,
) {
    let bounds = match arbol.bounds.get(&id) {
        Some(b) => *b,
        None => return,
    };

    let nodo = match arbol.nodos.get(&id) {
        Some(n) => n,
        None => return,
    };

    // Dibujar widget propio si es nodo hoja
    if let Some(ref widget) = nodo.widget {
        match widget {
            DatosWidget::Texto { valor, color, grande } => {
                let fg = color.unwrap_or(theme.fg);
                let fg_final = if *grande { theme.accent } else { fg };
                engine.poner_texto(bounds.x, bounds.y, valor, fg_final, theme.bg);
            }
            DatosWidget::Boton { texto, id_callback: _ } => {
                let tiene_foco = foco_id == Some(nodo.id);
                let ancho_total = texto.chars().count() + 4;
                if tiene_foco {
                    engine.poner_borde(bounds.x, bounds.y, ancho_total, 3, theme.fg);
                    engine.poner_texto_centrado(
                        bounds.x,
                        bounds.y + 1,
                        ancho_total,
                        texto,
                        theme.bg,
                        theme.fg,
                    );
                } else {
                    engine.poner_rectangulo(bounds.x, bounds.y, ancho_total, 3, theme.primary);
                    engine.poner_texto_centrado(
                        bounds.x,
                        bounds.y + 1,
                        ancho_total,
                        texto,
                        theme.fg,
                        theme.primary,
                    );
                }
            }
            DatosWidget::BarraProgreso { valor, color } => {
                let w = bounds.w.max(4);
                // Fondo de la barra
                engine.poner_rectangulo(bounds.x, bounds.y, w, 3, theme.text_bg);
                engine.poner_borde(bounds.x, bounds.y, w, 3, theme.border);
                // Relleno proporcional
                let interior_w = w.saturating_sub(2);
                let llenos = ((*valor / 100.0) * interior_w as f32) as usize;
                if llenos > 0 && interior_w > 0 {
                    engine.rellenar(
                        bounds.x + 1,
                        bounds.y + 1,
                        llenos.min(interior_w),
                        1,
                        '█',
                        *color,
                        *color,
                    );
                }
                // Texto de porcentaje
                let porcentaje = format!("{:.0}%", valor);
                engine.poner_texto_centrado(
                    bounds.x,
                    bounds.y + 1,
                    w,
                    &porcentaje,
                    theme.fg,
                    theme.text_bg,
                );
            }
            DatosWidget::Rectangulo { color, relleno } => {
                if *relleno {
                    engine.poner_rectangulo(bounds.x, bounds.y, bounds.w, bounds.h, *color);
                } else {
                    engine.poner_borde(bounds.x, bounds.y, bounds.w, bounds.h, *color);
                }
            }
            DatosWidget::Input { placeholder } => {
                engine.poner_borde(bounds.x, bounds.y, bounds.w, 3, theme.border);
                let interior_w = bounds.w.saturating_sub(2);
                if interior_w > 0 {
                    let truncado: String = placeholder.chars().take(interior_w).collect();
                    let pad = interior_w.saturating_sub(truncado.len());
                    let mut interior = String::with_capacity(interior_w);
                    interior.push_str(&truncado);
                    for _ in 0..pad {
                        interior.push(' ');
                    }
                    engine.poner_texto(
                        bounds.x + 1,
                        bounds.y + 1,
                        &interior,
                        theme.secondary,
                        theme.text_bg,
                    );
                }
            }
            DatosWidget::Separador => {
                engine.rellenar(
                    bounds.x,
                    bounds.y,
                    bounds.w.max(1),
                    1,
                    '─',
                    theme.border,
                    theme.bg,
                );
            }
        }
    }

    // Dibujar hijos recursivamente
    for hijo in &nodo.hijos {
        dibujar_nodo(arbol, hijo.id, engine, theme, foco_id);
    }
}

// ---------------------------------------------------------------------------
// crear_modulo — API pública para Argo-Lang
// ---------------------------------------------------------------------------
pub fn crear_modulo() -> Objeto {
    let mut funcs = HashMap::new();

    // -----------------------------------------------------------------------
    // ui.tema(config) — Cambiar colores globales
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("tema".to_string()),
        Objeto::Nativa(|args| {
            if args.is_empty() {
                return Objeto::Error(
                    "ui.tema: se esperaba 1 argumento (diccionario de colores)".to_string(),
                    Vec::new(),
                );
            }
            let config = match &args[0] {
                Objeto::Diccionario(d) => d.clone(),
                _ => {
                    return Objeto::Error(
                        "ui.tema: argumento debe ser un diccionario".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            for (key, value) in &config {
                if let LlaveHash::Cadena(k) = key {
                    let color = match value {
                        Objeto::Cadena(hex) => parsear_color(hex),
                        _ => continue,
                    };
                    match k.as_str() {
                        "bg" => st.arbol.theme.bg = color,
                        "fg" => st.arbol.theme.fg = color,
                        "primary" => st.arbol.theme.primary = color,
                        "secondary" => st.arbol.theme.secondary = color,
                        "accent" => st.arbol.theme.accent = color,
                        "border" => st.arbol.theme.border = color,
                        "text_bg" => st.arbol.theme.text_bg = color,
                        "error" => st.arbol.theme.error = color,
                        _ => {}
                    }
                }
            }

            Objeto::Nulo
        }),
    );

    // -----------------------------------------------------------------------
    // ui.columna(padre, opts?) -> id
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("columna".to_string()),
        Objeto::Nativa(|args| {
            if args.is_empty() {
                return Objeto::Error(
                    "ui.columna: se esperaba al menos 1 argumento (padre)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "padre") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let gap = if args.len() > 1 {
                extraer_entero(&args[1], "gap").unwrap_or(1)
            } else {
                1
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.arbol.siguiente_id();
            let mut nodo = widgets::NodoUI::new(id, TipoLayout::Columna);
            nodo.gap = gap;

            match st.arbol.insertar_nodo(id_padre, nodo) {
                Ok(new_id) => Objeto::Entero(new_id as i64),
                Err(e) => Objeto::Error(e, Vec::new()),
            }
        }),
    );

    // -----------------------------------------------------------------------
    // ui.fila(padre, opts?) -> id
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("fila".to_string()),
        Objeto::Nativa(|args| {
            if args.is_empty() {
                return Objeto::Error(
                    "ui.fila: se esperaba al menos 1 argumento (padre)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "padre") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let gap = if args.len() > 1 {
                extraer_entero(&args[1], "gap").unwrap_or(1)
            } else {
                1
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.arbol.siguiente_id();
            let mut nodo = widgets::NodoUI::new(id, TipoLayout::Fila);
            nodo.gap = gap;

            match st.arbol.insertar_nodo(id_padre, nodo) {
                Ok(new_id) => Objeto::Entero(new_id as i64),
                Err(e) => Objeto::Error(e, Vec::new()),
            }
        }),
    );

    // -----------------------------------------------------------------------
    // ui.texto(padre, texto, opts?) -> id
    // opts: { color: "#ffffff", grande: true }
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("texto".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 2 {
                return Objeto::Error(
                    "ui.texto: se esperaban 2 argumentos (padre, texto)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "padre") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let texto = match extraer_cadena(&args[1], "texto") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let mut color = None;
            let mut grande = false;

            if args.len() > 2 {
                if let Objeto::Diccionario(opts) = &args[2] {
                    for (key, val) in opts {
                        if let LlaveHash::Cadena(k) = key {
                            match k.as_str() {
                                "color" => {
                                    if let Objeto::Cadena(hex) = val {
                                        color = Some(parsear_color(hex));
                                    }
                                }
                                "grande" => {
                                    grande = matches!(val, Objeto::Booleano(true));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.arbol.siguiente_id();
            let mut nodo = widgets::NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Texto { valor: texto, color, grande });

            match st.arbol.insertar_nodo(id_padre, nodo) {
                Ok(new_id) => Objeto::Entero(new_id as i64),
                Err(e) => Objeto::Error(e, Vec::new()),
            }
        }),
    );

    // -----------------------------------------------------------------------
    // ui.boton(padre, texto, opts?) -> id
    // opts: { callback: 1 }
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("boton".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 2 {
                return Objeto::Error(
                    "ui.boton: se esperaban 2 argumentos (padre, texto)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "padre") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let texto = match extraer_cadena(&args[1], "texto") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let id_callback = if args.len() > 2 {
                if let Objeto::Diccionario(opts) = &args[2] {
                    if let Some(Objeto::Entero(cb)) = opts.get(&LlaveHash::Cadena("callback".to_string())) {
                        *cb as usize
                    } else {
                        0
                    }
                } else {
                    extraer_entero(&args[2], "callback").unwrap_or(0)
                }
            } else {
                0
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.arbol.siguiente_id();
            let mut nodo = widgets::NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Boton { texto, id_callback });

            match st.arbol.insertar_nodo(id_padre, nodo) {
                Ok(new_id) => Objeto::Entero(new_id as i64),
                Err(e) => Objeto::Error(e, Vec::new()),
            }
        }),
    );

    // -----------------------------------------------------------------------
    // ui.barra_progreso(padre, valor, opts?) -> id
    // opts: { color: "#10b981", animar: true }
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("barra_progreso".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 2 {
                return Objeto::Error(
                    "ui.barra_progreso: se esperaban 2 argumentos (padre, valor)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "padre") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let valor = match &args[1] {
                Objeto::Entero(n) => *n as f32,
                Objeto::Flotante(f) => *f as f32,
                _ => {
                    return Objeto::Error(
                        "ui.barra_progreso: valor debe ser numérico".to_string(),
                        Vec::new(),
                    )
                }
            };

            let mut color = (16, 185, 129); // accent por defecto
            let mut ancho = 30usize;

            if args.len() > 2 {
                if let Objeto::Diccionario(opts) = &args[2] {
                    for (key, val) in opts {
                        if let LlaveHash::Cadena(k) = key {
                            match k.as_str() {
                                "color" => {
                                    if let Objeto::Cadena(hex) = val {
                                        color = parsear_color(hex);
                                    }
                                }
                                "ancho" => {
                                    if let Objeto::Entero(w) = val {
                                        ancho = *w as usize;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.arbol.siguiente_id();
            let mut nodo = widgets::NodoUI::new(id, TipoLayout::Area);
            nodo.ancho = ancho;
            nodo.widget = Some(DatosWidget::BarraProgreso { valor, color });

            match st.arbol.insertar_nodo(id_padre, nodo) {
                Ok(new_id) => Objeto::Entero(new_id as i64),
                Err(e) => Objeto::Error(e, Vec::new()),
            }
        }),
    );

    // -----------------------------------------------------------------------
    // ui.input(padre, placeholder) -> id
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("input".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 2 {
                return Objeto::Error(
                    "ui.input: se esperaban 2 argumentos (padre, placeholder)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "padre") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let placeholder = match extraer_cadena(&args[1], "placeholder") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.arbol.siguiente_id();
            let mut nodo = widgets::NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Input { placeholder });

            match st.arbol.insertar_nodo(id_padre, nodo) {
                Ok(new_id) => Objeto::Entero(new_id as i64),
                Err(e) => Objeto::Error(e, Vec::new()),
            }
        }),
    );

    // -----------------------------------------------------------------------
    // ui.separador(padre) -> id
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("separador".to_string()),
        Objeto::Nativa(|args| {
            if args.is_empty() {
                return Objeto::Error(
                    "ui.separador: se esperaba 1 argumento (padre)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "padre") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.arbol.siguiente_id();
            let mut nodo = widgets::NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Separador);

            match st.arbol.insertar_nodo(id_padre, nodo) {
                Ok(new_id) => Objeto::Entero(new_id as i64),
                Err(e) => Objeto::Error(e, Vec::new()),
            }
        }),
    );

    // -----------------------------------------------------------------------
    // ui.rectangulo(padre, ancho, alto, opts?) -> id
    // opts: { color: "#ff0000", relleno: true }
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("rectangulo".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 3 {
                return Objeto::Error(
                    "ui.rectangulo: se esperaban 3 argumentos (padre, ancho, alto)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "padre") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let ancho = match extraer_entero(&args[1], "ancho") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let alto = match extraer_entero(&args[2], "alto") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let mut color = (75, 85, 99); // border
            let mut relleno = false;

            if args.len() > 3 {
                if let Objeto::Diccionario(opts) = &args[3] {
                    for (key, val) in opts {
                        if let LlaveHash::Cadena(k) = key {
                            match k.as_str() {
                                "color" => {
                                    if let Objeto::Cadena(hex) = val {
                                        color = parsear_color(hex);
                                    }
                                }
                                "relleno" => {
                                    relleno = matches!(val, Objeto::Booleano(true));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.arbol.siguiente_id();
            let mut nodo = widgets::NodoUI::new(id, TipoLayout::Area);
            nodo.ancho = ancho;
            nodo.alto = alto;
            nodo.widget = Some(DatosWidget::Rectangulo { color, relleno });

            match st.arbol.insertar_nodo(id_padre, nodo) {
                Ok(new_id) => Objeto::Entero(new_id as i64),
                Err(e) => Objeto::Error(e, Vec::new()),
            }
        }),
    );

    // -----------------------------------------------------------------------
    // ui.animar(nodo_id, propiedad, fin, duracion_ms) — Lanza una animación
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("animar".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 4 {
                return Objeto::Error(
                    "ui.animar: se esperaban 4 argumentos (nodo_id, propiedad, fin, duracion_ms)"
                        .to_string(),
                    Vec::new(),
                );
            }
            let nodo_id = match extraer_entero(&args[0], "nodo_id") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let propiedad = match extraer_cadena(&args[1], "propiedad") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let fin = match &args[2] {
                Objeto::Entero(n) => *n as f32,
                Objeto::Flotante(f) => *f as f32,
                _ => {
                    return Objeto::Error(
                        "ui.animar: fin debe ser numérico".to_string(),
                        Vec::new(),
                    )
                }
            };
            let duracion_ms = match extraer_entero(&args[3], "duracion_ms") {
                Ok(v) => v as u64,
                Err(e) => return e,
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            // Obtener valor actual para hacer lerp desde ahí
            let inicio = 0.0f32;
            st.animaciones.lanzar(nodo_id, &propiedad, inicio, fin, duracion_ms);

            Objeto::Nulo
        }),
    );

    // -----------------------------------------------------------------------
    // ui.limpiar() — Limpia la pantalla
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("limpiar".to_string()),
        Objeto::Nativa(|_args| {
            let state = get_state();
            let mut st = state.lock().unwrap();
            st.engine.limpiar();
            let mut stdout = BufWriter::new(std::io::stdout());
            st.engine.flush(&mut stdout);
            Objeto::Nulo
        }),
    );

    // -----------------------------------------------------------------------
    // ui.reset() — Reinicia el árbol de UI
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("reset".to_string()),
        Objeto::Nativa(|_args| {
            let state = get_state();
            let mut st = state.lock().unwrap();
            st.arbol = ArbolUI::new();
            st.animaciones = animaciones::MotorAnimaciones::new();
            st.engine.limpiar();
            let mut stdout = BufWriter::new(std::io::stdout());
            MotorRenderizado::restaurar(&mut stdout);
            st.engine.flush(&mut stdout);
            st.primer_frame = true;
            Objeto::Nulo
        }),
    );

    // -----------------------------------------------------------------------
    // ui.ejecutar() — GAME LOOP PRINCIPAL ~30 FPS
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("ejecutar".to_string()),
        Objeto::Nativa(|_args| {
            // Preparar: recoger focusables
            {
                let state = get_state();
                let mut st = state.lock().unwrap();
                st.arbol.recoger_focusables();
                if st.arbol.foco_id.is_none() && !st.arbol.widgets_focusable.is_empty() {
                    st.arbol.foco_id = Some(st.arbol.widgets_focusable[0]);
                }
            }

            // Ocultar cursor
            {
                let mut stdout = BufWriter::new(std::io::stdout());
                MotorRenderizado::ocultar_cursor(&mut stdout);
            }

            // GAME LOOP — 30 FPS (33ms por frame)
            loop {
                std::thread::sleep(Duration::from_millis(33));

                // 1. Calcular animaciones
                let frame_values = {
                    let state = get_state();
                    let mut st = state.lock().unwrap();
                    st.animaciones.calcular_frame()
                };

                // 2. Aplicar valores de animación al árbol
                if !frame_values.is_empty() {
                    let state = get_state();
                    let mut st = state.lock().unwrap();
                    for (nodo_id, propiedad, valor) in &frame_values {
                        aplicar_animacion(&mut st.arbol, *nodo_id, propiedad, *valor);
                    }
                }

                // 3. Layout + Render
                {
                    let state = get_state();
                    let mut st = state.lock().unwrap();

                    let max_ancho = st.engine.ancho.saturating_sub(2);
                    st.arbol.calcular_layout(max_ancho);

                    if st.primer_frame {
                        st.engine.limpiar();
                        st.primer_frame = false;
                    }

                    let theme = st.arbol.theme.clone();
                    let foco = st.arbol.foco_id;
                    let raiz = st.arbol.raiz;

                    if raiz != 0 {
                        let UiGlobal { arbol, engine, .. } = &mut *st;
                        dibujar_nodo(arbol, raiz, engine, &theme, foco);
                    }

                    let mut stdout = BufWriter::new(std::io::stdout());
                    st.engine.flush(&mut stdout);
                }

                // 4. Input (no bloqueante)
                match leer_tecla_no_bloqueante() {
                    "q" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        st.engine.limpiar();
                        let mut stdout = BufWriter::new(std::io::stdout());
                        st.engine.flush(&mut stdout);
                        MotorRenderizado::restaurar(&mut stdout);
                        st.primer_frame = true;
                        break;
                    }
                    "Tab" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        st.arbol.siguiente_foco();
                    }
                    "Backspace" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        st.arbol.anterior_foco();
                    }
                    "Enter" => {
                        let state = get_state();
                        let st = state.lock().unwrap();
                        if let Some(callback_id) = st.arbol.callback_foco() {
                            let mut stdout = BufWriter::new(std::io::stdout());
                            MotorRenderizado::restaurar(&mut stdout);
                            return Objeto::Entero(callback_id as i64);
                        }
                    }
                    _ => {}
                }
            }

            Objeto::Nulo
        }),
    );

    Objeto::Diccionario(funcs)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
/// Aplica un valor de animación a una propiedad del nodo.
fn aplicar_animacion(arbol: &mut ArbolUI, nodo_id: usize, propiedad: &str, valor: f32) {
    if let Some(nodo) = arbol.nodos.get_mut(&nodo_id) {
        if let Some(ref mut widget) = nodo.widget {
            match widget {
                DatosWidget::BarraProgreso { valor: v, .. } => {
                    if propiedad == "valor" {
                        *v = valor;
                    }
                }
                DatosWidget::Texto { color, .. } => {
                    if propiedad.starts_with("fg_") {
                        if let Some(c) = color {
                            match propiedad {
                                "fg_r" => c.0 = valor as u8,
                                "fg_g" => c.1 = valor as u8,
                                "fg_b" => c.2 = valor as u8,
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[allow(clippy::result_large_err)]
fn extraer_entero(obj: &Objeto, campo: &str) -> Result<usize, Objeto> {
    match obj {
        Objeto::Entero(n) => Ok(*n as usize),
        _ => Err(Objeto::Error(
            format!("ui: {} debe ser un entero", campo),
            Vec::new(),
        )),
    }
}

#[allow(clippy::result_large_err)]
fn extraer_cadena(obj: &Objeto, campo: &str) -> Result<String, Objeto> {
    match obj {
        Objeto::Cadena(s) => Ok(s.clone()),
        _ => Err(Objeto::Error(
            format!("ui: {} debe ser una cadena", campo),
            Vec::new(),
        )),
    }
}
