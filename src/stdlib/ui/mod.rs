pub mod input;
pub mod renderer;
pub mod widgets;

use std::sync::Mutex;
use std::sync::OnceLock;

use crate::evaluator::object::LlaveHash;
use crate::evaluator::Objeto;

use renderer::TerminalRenderer;
use widgets::{TipoWidget, UiState, Widget};

pub struct AppState {
    pub ui: UiState,
    pub renderer: TerminalRenderer,
}

static APP_STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

fn get_state() -> &'static Mutex<AppState> {
    APP_STATE.get_or_init(|| {
        Mutex::new(AppState {
            ui: UiState::new(),
            renderer: TerminalRenderer::new(),
        })
    })
}

pub fn crear_modulo() -> Objeto {
    let mut funcs = std::collections::HashMap::new();

    funcs.insert(
        LlaveHash::Cadena("ventana".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 3 {
                return Objeto::Error(
                    "ui.ventana: se esperaban 3 argumentos (nombre, ancho, alto)".to_string(),
                    Vec::new(),
                );
            }
            let nombre = match &args[0] {
                Objeto::Cadena(s) => s.clone(),
                _ => {
                    return Objeto::Error(
                        "ui.ventana: el nombre debe ser una cadena".to_string(),
                        Vec::new(),
                    )
                }
            };
            let ancho = match &args[1] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.ventana: el ancho debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let alto = match &args[2] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.ventana: el alto debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            // Batch: limpiar + borde + título en un solo flush
            st.renderer.limpiar_pantalla();
            st.renderer.dibujar_borde(0, 0, ancho, alto);
            if !nombre.is_empty() {
                st.renderer.dibujar_texto(2, 0, &nombre, 255, 255, 255);
            }
            st.renderer.flush();

            let id = st.ui.siguiente_id();
            st.ui.agregar_widget(Widget {
                id,
                tipo: TipoWidget::Ventana,
                x: 0,
                y: 0,
                ancho,
                alto,
                texto: nombre,
                callback_id: None,
            });

            Objeto::Entero(id as i64)
        }),
    );

    funcs.insert(
        LlaveHash::Cadena("texto".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 4 {
                return Objeto::Error(
                    "ui.texto: se esperaban 4 argumentos (app_id, texto, x, y)".to_string(),
                    Vec::new(),
                );
            }
            let _app_id = match &args[0] {
                Objeto::Entero(n) => *n,
                _ => {
                    return Objeto::Error(
                        "ui.texto: app_id debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let texto = match &args[1] {
                Objeto::Cadena(s) => s.clone(),
                _ => {
                    return Objeto::Error(
                        "ui.texto: el texto debe ser una cadena".to_string(),
                        Vec::new(),
                    )
                }
            };
            let x = match &args[2] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.texto: x debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let y = match &args[3] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.texto: y debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            st.renderer
                .dibujar_texto(x, y, &texto, 255, 255, 255);
            st.renderer.flush();

            let id = st.ui.siguiente_id();
            st.ui.agregar_widget(Widget {
                id,
                tipo: TipoWidget::Texto,
                x,
                y,
                ancho: texto.len(),
                alto: 1,
                texto,
                callback_id: None,
            });

            Objeto::Entero(id as i64)
        }),
    );

    funcs.insert(
        LlaveHash::Cadena("boton".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 5 {
                return Objeto::Error(
                    "ui.boton: se esperaban 5 argumentos (app_id, texto, x, y, callback_id)"
                        .to_string(),
                    Vec::new(),
                );
            }
            let _app_id = match &args[0] {
                Objeto::Entero(n) => *n,
                _ => {
                    return Objeto::Error(
                        "ui.boton: app_id debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let texto = match &args[1] {
                Objeto::Cadena(s) => s.clone(),
                _ => {
                    return Objeto::Error(
                        "ui.boton: el texto debe ser una cadena".to_string(),
                        Vec::new(),
                    )
                }
            };
            let x = match &args[2] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.boton: x debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let y = match &args[3] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.boton: y debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let callback_id = match &args[4] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ui.boton: callback_id debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            let ancho_btn = texto.len() + 4;

            // Batch: fondo + cursor + texto + cursor en un solo flush
            st.renderer.color_fondo(0, 50, 100);
            st.renderer.mover_cursor(x, y);
            print!(" ");
            st.renderer.color_texto(255, 255, 255);
            print!(" {} ", texto);
            st.renderer.resetear_colores();
            st.renderer.mover_cursor(x, y);
            st.renderer.flush();

            let id = st.ui.siguiente_id();
            st.ui.agregar_widget(Widget {
                id,
                tipo: TipoWidget::Boton,
                x,
                y,
                ancho: ancho_btn,
                alto: 1,
                texto,
                callback_id: Some(callback_id),
            });

            Objeto::Entero(id as i64)
        }),
    );

    funcs.insert(
        LlaveHash::Cadena("ejecutar".to_string()),
        Objeto::Nativa(|args| {
            if args.is_empty() {
                return Objeto::Error(
                    "ui.ejecutar: se esperaba 1 argumento (app_id)".to_string(),
                    Vec::new(),
                );
            }
            let _app_id = match &args[0] {
                Objeto::Entero(n) => *n,
                _ => {
                    return Objeto::Error(
                        "ui.ejecutar: app_id debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            loop {
                match input::leer_tecla() {
                    "q" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        st.renderer.resetear_colores();
                        st.renderer.limpiar_pantalla();
                        break;
                    }
                    "Enter" => {
                        let state = get_state();
                        let st = state.lock().unwrap();
                        if let Some(callback_id) = st.ui.obtener_callback_ultimo_boton() {
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
