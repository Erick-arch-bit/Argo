pub mod layout;
pub mod widgets;

use std::io::BufWriter;
use std::sync::Mutex;
use std::sync::OnceLock;

use crate::evaluator::object::LlaveHash;
use crate::evaluator::Objeto;
use crate::stdlib::render::RenderEngine;

use layout::{calcular_layout, dibujar_arbol, LayoutCache};
use widgets::{DatosWidget, NodoUI, TipoLayout, UiState};

// ---------------------------------------------------------------------------
// Estado global de la UI
// ---------------------------------------------------------------------------
pub struct AppState {
    pub ui: UiState,
    pub engine: RenderEngine,
    pub layout_cache: LayoutCache,
}

static APP_STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

fn get_state() -> &'static Mutex<AppState> {
    APP_STATE.get_or_init(|| {
        let (ancho, alto) = RenderEngine::detectar_terminal();
        Mutex::new(AppState {
            ui: UiState::new(),
            engine: RenderEngine::new(ancho, alto),
            layout_cache: LayoutCache::new(),
        })
    })
}

// ---------------------------------------------------------------------------
// Input — Lectura de teclado (zero-alloc)
// ---------------------------------------------------------------------------
fn leer_tecla() -> &'static str {
    use std::io::Read;
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
        9 => "Tab",
        127 => "Backspace",
        b'q' | b'Q' => "q",
        _ => "q",
    }
}

// ---------------------------------------------------------------------------
// crear_modulo — API pública para Argo-Lang
// ---------------------------------------------------------------------------
pub fn crear_modulo() -> Objeto {
    let mut funcs = std::collections::HashMap::new();

    // -----------------------------------------------------------------------
    // ui.tema(config) — Cambiar colores globales
    // config es un diccionario: { bg: "#2b2b2b", fg: "#ffffff", primary: "#3b82f6" }
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

            // Parsear colores del diccionario
            for (key, value) in &config {
                let color = match value {
                    Objeto::Cadena(hex) => parsear_hex(hex),
                    _ => continue,
                };
                if let Some(c) = color {
                    match key {
                        LlaveHash::Cadena(k) if k == "bg" => st.ui.theme.bg = c,
                        LlaveHash::Cadena(k) if k == "fg" => st.ui.theme.fg = c,
                        LlaveHash::Cadena(k) if k == "primary" => st.ui.theme.primary = c,
                        LlaveHash::Cadena(k) if k == "secondary" => st.ui.theme.secondary = c,
                        LlaveHash::Cadena(k) if k == "accent" => st.ui.theme.accent = c,
                        LlaveHash::Cadena(k) if k == "border" => st.ui.theme.border = c,
                        LlaveHash::Cadena(k) if k == "text_bg" => st.ui.theme.text_bg = c,
                        LlaveHash::Cadena(k) if k == "error" => st.ui.theme.error = c,
                        _ => {}
                    }
                }
            }

            Objeto::Nulo
        }),
    );

    // -----------------------------------------------------------------------
    // ui.columna(id_padre, gap) -> id
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("columna".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 2 {
                return Objeto::Error(
                    "ui.columna: se esperaban 2 argumentos (id_padre, gap)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "id_padre") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let gap = match extraer_entero(&args[1], "gap") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Columna);
            nodo.gap = gap;

            insertar_nodo(&mut st.ui, id_padre, nodo)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.fila(id_padre, gap) -> id
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("fila".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 2 {
                return Objeto::Error(
                    "ui.fila: se esperaban 2 argumentos (id_padre, gap)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "id_padre") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let gap = match extraer_entero(&args[1], "gap") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Fila);
            nodo.gap = gap;

            insertar_nodo(&mut st.ui, id_padre, nodo)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.texto(id_padre, texto, opts?) -> id
    // opts: { color: "#ffffff", grande: true }
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("texto".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 2 {
                return Objeto::Error(
                    "ui.texto: se esperaban 2 argumentos (id_padre, texto)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "id_padre") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let texto = match extraer_cadena(&args[1], "texto") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let mut color = None;
            let mut grande = false;

            if args.len() > 2
                && let Objeto::Diccionario(opts) = &args[2]
            {
                for (key, val) in opts {
                    if let LlaveHash::Cadena(k) = key {
                        match k.as_str() {
                            "color" => {
                                if let Objeto::Cadena(hex) = val {
                                    color = parsear_hex(hex);
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

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Texto { valor: texto, color, grande });

            insertar_nodo(&mut st.ui, id_padre, nodo)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.boton(id_padre, texto, id_callback) -> id
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("boton".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 3 {
                return Objeto::Error(
                    "ui.boton: se esperaban 3 argumentos (id_padre, texto, id_callback)"
                        .to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "id_padre") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let texto = match extraer_cadena(&args[1], "texto") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let id_callback = match extraer_entero(&args[2], "id_callback") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Boton { texto, id_callback });

            insertar_nodo(&mut st.ui, id_padre, nodo)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.input(id_padre, placeholder) -> id
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("input".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 2 {
                return Objeto::Error(
                    "ui.input: se esperaban 2 argumentos (id_padre, placeholder)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "id_padre") {
                Ok(v) => v,
                Err(e) => return e,
            };
            let placeholder = match extraer_cadena(&args[1], "placeholder") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Input { placeholder });

            insertar_nodo(&mut st.ui, id_padre, nodo)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.separador(id_padre) -> id
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("separador".to_string()),
        Objeto::Nativa(|args| {
            if args.is_empty() {
                return Objeto::Error(
                    "ui.separador: se esperaba 1 argumento (id_padre)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match extraer_entero(&args[0], "id_padre") {
                Ok(v) => v,
                Err(e) => return e,
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Separador);

            insertar_nodo(&mut st.ui, id_padre, nodo)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.ejecutar() — Loop principal con doble buffering
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("ejecutar".to_string()),
        Objeto::Nativa(|_args| {
            {
                let state = get_state();
                let mut st = state.lock().unwrap();
                st.ui.recoger_focusables();
                if st.ui.foco_id.is_none() && !st.ui.widgets_focusable.is_empty() {
                    st.ui.foco_id = Some(st.ui.widgets_focusable[0]);
                }
            }

            loop {
                // Renderizar
                {
                    let state = get_state();
                    let mut st = state.lock().unwrap();

                    let foco = st.ui.foco_id;
                    let buf_ancho = st.engine.ancho;
                    let theme = widgets::Theme {
                        bg: st.ui.theme.bg,
                        fg: st.ui.theme.fg,
                        primary: st.ui.theme.primary,
                        secondary: st.ui.theme.secondary,
                        accent: st.ui.theme.accent,
                        border: st.ui.theme.border,
                        text_bg: st.ui.theme.text_bg,
                        error: st.ui.theme.error,
                    };

                    if let Some(raiz) = st.ui.raiz.take() {
                        st.engine.limpiar_todo();

                        calcular_layout(&raiz, 1, 1, buf_ancho.saturating_sub(2), &mut st.layout_cache);

                        // Tomar layout_cache temporalmente para evitar borrow conflict
                        let cache = std::mem::take(&mut st.layout_cache);
                        dibujar_arbol(&raiz, &cache, &mut st.engine, &theme, foco);
                        st.layout_cache = cache;

                        let mut stdout = BufWriter::new(std::io::stdout());
                        st.engine.flush(&mut stdout);

                        st.ui.raiz = Some(raiz);
                    }
                }

                // Input
                match leer_tecla() {
                    "q" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        st.engine.limpiar_todo();
                        let mut stdout = BufWriter::new(std::io::stdout());
                        st.engine.flush(&mut stdout);
                        break;
                    }
                    "Tab" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        st.ui.siguiente_foco();
                    }
                    "Backspace" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        st.ui.anterior_foco();
                    }
                    "Enter" => {
                        let state = get_state();
                        let st = state.lock().unwrap();
                        if let Some(callback_id) = st.ui.callback_foco() {
                            return Objeto::Entero(callback_id as i64);
                        }
                    }
                    _ => {}
                }
            }

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
            st.engine.limpiar_todo();
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
            st.ui.raiz = None;
            st.ui.foco_id = None;
            st.ui.widgets_focusable.clear();
            st.ui.contador_ids = 0;
            st.engine.limpiar_todo();
            let mut stdout = BufWriter::new(std::io::stdout());
            st.engine.flush(&mut stdout);
            Objeto::Nulo
        }),
    );

    Objeto::Diccionario(funcs)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

/// Parsea "#RRGGBB" a (R, G, B).
fn parsear_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Inserta un nodo en el árbol. Retorna Objeto directamente (no Result).
fn insertar_nodo(ui: &mut UiState, id_padre: usize, nodo: NodoUI) -> Objeto {
    let id = nodo.id;
    if id_padre == 0 {
        ui.raiz = Some(nodo);
    } else if let Some(ref mut raiz) = ui.raiz {
        if let Some(padre) = UiState::buscar_nodo_mut(raiz, id_padre) {
            padre.hijos.push(nodo);
        } else {
            return Objeto::Error(
                format!("ui: nodo padre {} no encontrado", id_padre),
                Vec::new(),
            );
        }
    } else {
        return Objeto::Error(
            "ui: no hay árbol de UI (llama con id_padre=0 primero)".to_string(),
            Vec::new(),
        );
    }
    Objeto::Entero(id as i64)
}
