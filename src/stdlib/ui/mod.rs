pub mod buffer;
pub mod layout;
pub mod widgets;

use std::io::Read;
use std::sync::Mutex;
use std::sync::OnceLock;

use crate::evaluator::object::LlaveHash;
use crate::evaluator::Objeto;

use buffer::PantallaBuffer;
use layout::{calcular_layout, dibujar_arbol};
use widgets::{DatosWidget, NodoUI, TipoLayout, UiState};

// ---------------------------------------------------------------------------
// Estado global de la UI
// ---------------------------------------------------------------------------
pub struct AppState {
    pub ui: UiState,
    pub buffer: PantallaBuffer,
}

static APP_STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

fn get_state() -> &'static Mutex<AppState> {
    APP_STATE.get_or_init(|| {
        let (ancho, alto) = PantallaBuffer::detectar_terminal();
        Mutex::new(AppState {
            ui: UiState::new(),
            buffer: PantallaBuffer::new(ancho, alto),
        })
    })
}

// ---------------------------------------------------------------------------
// Input — Lectura de teclado (zero-alloc, retorna &'static str)
// ---------------------------------------------------------------------------
fn leer_tecla() -> &'static str {
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
    // ui.columna(id_padre, margen, gap) -> id del nodo creado
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("columna".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 3 {
                return Objeto::Error(
                    "ui.columna: se esperaban 3 argumentos (id_padre, margen, gap)".to_string(),
                    Vec::new(),
                );
            }
            let id_padre = match &args[0] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.columna: id_padre debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let margen = match &args[1] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.columna: margen debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let gap = match &args[2] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.columna: gap debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Columna);
            nodo.margen = margen;
            nodo.gap = gap;

            // Si id_padre == 0, es la raíz
            if id_padre == 0 {
                st.ui.raiz = Some(nodo);
            } else {
                if let Some(ref mut raiz) = st.ui.raiz {
                    if let Some(padre) = UiState::buscar_nodo_mut(raiz, id_padre) {
                        padre.agregar_hijo(nodo);
                    } else {
                        return Objeto::Error(
                            format!("ui.columna: nodo padre {} no encontrado", id_padre),
                            Vec::new(),
                        );
                    }
                } else {
                    return Objeto::Error(
                        "ui.columna: no hay árbol de UI (llama a ui.columna(0, ...) primero)"
                            .to_string(),
                        Vec::new(),
                    );
                }
            }

            Objeto::Entero(id as i64)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.fila(id_padre, gap) -> id del nodo creado
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
            let id_padre = match &args[0] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.fila: id_padre debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let gap = match &args[1] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.fila: gap debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Fila);
            nodo.gap = gap;

            if id_padre == 0 {
                st.ui.raiz = Some(nodo);
            } else {
                if let Some(ref mut raiz) = st.ui.raiz {
                    if let Some(padre) = UiState::buscar_nodo_mut(raiz, id_padre) {
                        padre.agregar_hijo(nodo);
                    } else {
                        return Objeto::Error(
                            format!("ui.fila: nodo padre {} no encontrado", id_padre),
                            Vec::new(),
                        );
                    }
                } else {
                    return Objeto::Error(
                        "ui.fila: no hay árbol de UI (llama a ui.fila(0, ...) primero)".to_string(),
                        Vec::new(),
                    );
                }
            }

            Objeto::Entero(id as i64)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.texto(id_padre, texto) -> id del nodo creado
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
            let id_padre = match &args[0] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.texto: id_padre debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let texto = match &args[1] {
                Objeto::Cadena(s) => s.clone(),
                _ => {
                    return Objeto::Error(
                        "ui.texto: texto debe ser una cadena".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Texto {
                valor: texto,
                color: (255, 255, 255),
            });

            if id_padre == 0 {
                st.ui.raiz = Some(nodo);
            } else {
                if let Some(ref mut raiz) = st.ui.raiz {
                    if let Some(padre) = UiState::buscar_nodo_mut(raiz, id_padre) {
                        padre.agregar_hijo(nodo);
                    } else {
                        return Objeto::Error(
                            format!("ui.texto: nodo padre {} no encontrado", id_padre),
                            Vec::new(),
                        );
                    }
                } else {
                    return Objeto::Error(
                        "ui.texto: no hay árbol de UI (llama a ui.texto(0, ...) primero)"
                            .to_string(),
                        Vec::new(),
                    );
                }
            }

            Objeto::Entero(id as i64)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.boton(id_padre, texto, id_callback) -> id del nodo creado
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
            let id_padre = match &args[0] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.boton: id_padre debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let texto = match &args[1] {
                Objeto::Cadena(s) => s.clone(),
                _ => {
                    return Objeto::Error(
                        "ui.boton: texto debe ser una cadena".to_string(),
                        Vec::new(),
                    )
                }
            };
            let id_callback = match &args[2] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.boton: id_callback debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Boton {
                texto,
                id_callback,
            });

            if id_padre == 0 {
                st.ui.raiz = Some(nodo);
            } else {
                if let Some(ref mut raiz) = st.ui.raiz {
                    if let Some(padre) = UiState::buscar_nodo_mut(raiz, id_padre) {
                        padre.agregar_hijo(nodo);
                    } else {
                        return Objeto::Error(
                            format!("ui.boton: nodo padre {} no encontrado", id_padre),
                            Vec::new(),
                        );
                    }
                } else {
                    return Objeto::Error(
                        "ui.boton: no hay árbol de UI (llama a ui.boton(0, ...) primero)"
                            .to_string(),
                        Vec::new(),
                    );
                }
            }

            Objeto::Entero(id as i64)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.input(id_padre, placeholder) -> id del nodo creado
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
            let id_padre = match &args[0] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.input: id_padre debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let placeholder = match &args[1] {
                Objeto::Cadena(s) => s.clone(),
                _ => {
                    return Objeto::Error(
                        "ui.input: placeholder debe ser una cadena".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.ui.siguiente_id();
            let mut nodo = NodoUI::new(id, TipoLayout::Area);
            nodo.widget = Some(DatosWidget::Input { placeholder });

            if id_padre == 0 {
                st.ui.raiz = Some(nodo);
            } else {
                if let Some(ref mut raiz) = st.ui.raiz {
                    if let Some(padre) = UiState::buscar_nodo_mut(raiz, id_padre) {
                        padre.agregar_hijo(nodo);
                    } else {
                        return Objeto::Error(
                            format!("ui.input: nodo padre {} no encontrado", id_padre),
                            Vec::new(),
                        );
                    }
                } else {
                    return Objeto::Error(
                        "ui.input: no hay árbol de UI (llama a ui.input(0, ...) primero)"
                            .to_string(),
                        Vec::new(),
                    );
                }
            }

            Objeto::Entero(id as i64)
        }),
    );

    // -----------------------------------------------------------------------
    // ui.ejecutar() -> Nulo | callback_id al presionar Enter
    //
    // 1. Calcula layout del árbol
    // 2. Dibuja todo en el buffer
    // 3. Flush a la terminal
    // 4. Loop de input:
    //    - Tab: avanza foco, re-dibuja
    //    - Shift+Tab (Backspace): retrocede foco
    //    - Enter: dispara callback del widget con foco
    //    - q: sale del loop
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("ejecutar".to_string()),
        Objeto::Nativa(|args| {
            if !args.is_empty() {
                // Si se pasa un argumento, es el id de un nodo raíz específico
                let _id = match &args[0] {
                    Objeto::Entero(n) => *n as usize,
                    _ => {
                        return Objeto::Error(
                            "ui.ejecutar: argumento debe ser un entero".to_string(),
                            Vec::new(),
                        )
                    }
                };
            }

            // Recoger widgets focusables
            {
                let state = get_state();
                let mut st = state.lock().unwrap();
                st.ui.recoger_focusables();
                // Foco inicial: primer widget focusable
                if st.ui.foco_id.is_none() && !st.ui.widgets_focusable.is_empty() {
                    st.ui.foco_id = Some(st.ui.widgets_focusable[0]);
                }
            }

            loop {
                // Renderizar: extraer raiz para evitar borrow conflicts
                {
                    let state = get_state();
                    let mut st = state.lock().unwrap();

                    let foco = st.ui.foco_id;
                    let buf_ancho = st.buffer.ancho;

                    // Tomar raiz temporalmente
                    if let Some(raiz) = st.ui.raiz.take() {
                        st.buffer.limpiar();

                        let resultados =
                            calcular_layout(&raiz, 1, 1, buf_ancho.saturating_sub(2));
                        dibujar_arbol(&raiz, &resultados, &mut st.buffer, foco);
                        st.buffer.flush();

                        // Devolver raiz al estado
                        st.ui.raiz = Some(raiz);
                    }
                }

                // Leer tecla
                match leer_tecla() {
                    "q" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        st.buffer.limpiar();
                        st.buffer.flush();
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
    // ui.limpiar() -> Limpia el buffer y la pantalla
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("limpiar".to_string()),
        Objeto::Nativa(|_args| {
            let state = get_state();
            let mut st = state.lock().unwrap();
            st.buffer.limpiar();
            st.buffer.flush();
            Objeto::Nulo
        }),
    );

    // -----------------------------------------------------------------------
    // ui.reset() -> Reinicia el árbol de UI
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
            st.buffer.limpiar();
            st.buffer.flush();
            Objeto::Nulo
        }),
    );

    Objeto::Diccionario(funcs)
}
