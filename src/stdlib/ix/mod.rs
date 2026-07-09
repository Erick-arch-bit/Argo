use std::io::BufWriter;
use std::sync::Mutex;
use std::sync::OnceLock;

use crate::evaluator::object::LlaveHash;
use crate::evaluator::Objeto;
use crate::stdlib::render::RenderEngine;

// ---------------------------------------------------------------------------
// IXWindow — Ventana flotante
// ---------------------------------------------------------------------------
pub struct IXWindow {
    pub id: usize,
    pub titulo: String,
    pub x: usize,
    pub y: usize,
    pub ancho: usize,
    pub alto: usize,
    pub z_index: usize,
    pub callback_id: Option<usize>,
    pub visible: bool,
}

// ---------------------------------------------------------------------------
// IXDesktop — Escritorio con ventanas
// ---------------------------------------------------------------------------
pub struct IXDesktop {
    pub ventanas: Vec<IXWindow>,
    pub ventana_foco: Option<usize>,
    pub contador_ids: usize,
    pub engine: RenderEngine,
    pub max_z: usize,
}

impl Default for IXDesktop {
    fn default() -> Self {
        Self::new()
    }
}

impl IXDesktop {
    pub fn new() -> Self {
        let (ancho, alto) = RenderEngine::detectar_terminal();
        IXDesktop {
            ventanas: Vec::new(),
            ventana_foco: None,
            contador_ids: 0,
            engine: RenderEngine::new(ancho, alto),
            max_z: 0,
        }
    }

    pub fn siguiente_id(&mut self) -> usize {
        self.contador_ids += 1;
        self.contador_ids
    }

    pub fn siguiente_z(&mut self) -> usize {
        self.max_z += 1;
        self.max_z
    }

    /// Busca una ventana por ID.
    pub fn buscar_ventana(&self, id: usize) -> Option<&IXWindow> {
        self.ventanas.iter().find(|v| v.id == id)
    }

    pub fn buscar_ventana_mut(&mut self, id: usize) -> Option<&mut IXWindow> {
        self.ventanas.iter_mut().find(|v| v.id == id)
    }

    /// Trae una ventana al frente (mayor z_index).
    pub fn traer_al_frente(&mut self, id: usize) {
        let z = self.siguiente_z();
        if let Some(v) = self.buscar_ventana_mut(id) {
            v.z_index = z;
        }
    }

    /// Ordena ventanas por z_index (menor primero = se dibuja primero).
    pub fn ordenar_por_z(&mut self) {
        self.ventanas.sort_by_key(|v| v.z_index);
    }

    /// Dibuja todas las ventanas en el engine.
    pub fn dibujar_todas(&mut self) {
        self.ordenar_por_z();

        // Copiar ids para evitar borrow issues
        let ids: Vec<usize> = self.ventanas.iter().map(|v| v.id).collect();

        for id in &ids {
            self.dibujar_ventana(*id);
        }
    }

    fn dibujar_ventana(&mut self, id: usize) {
        // Extraer datos de la ventana primero
        let (x, y, ancho, alto, titulo, is_foco) = {
            let v = match self.buscar_ventana(id) {
                Some(v) => v,
                None => return,
            };
            if !v.visible {
                return;
            }
            let is_foco = self.ventana_foco == Some(id);
            (v.x, v.y, v.ancho, v.alto, v.titulo.clone(), is_foco)
        };

        // Fondo de la ventana
        self.engine.poner_rectangulo(x, y, ancho, alto, (30, 30, 30));

        // Barra de título
        let bar_color = if is_foco { (59, 130, 246) } else { (75, 85, 99) };
        self.engine.poner_rectangulo(x, y, ancho, 1, bar_color);

        // Texto de la título
        let max_titulo = ancho.saturating_sub(4);
        let titulo_corto: String = titulo.chars().take(max_titulo).collect();
        self.engine.poner_texto(x + 2, y, &titulo_corto, (255, 255, 255), bar_color);

        // Botones de la barra (● ● ●)
        if ancho > 6 {
            self.engine.poner_celda(x + ancho - 3, y, '×', (239, 68, 68), bar_color);
        }
        if ancho > 8 {
            self.engine.poner_celda(x + ancho - 5, y, '○', (234, 179, 8), bar_color);
        }
        if ancho > 10 {
            self.engine.poner_celda(x + ancho - 7, y, '●', (34, 197, 94), bar_color);
        }

        // Borde
        let borde_color = if is_foco { (59, 130, 246) } else { (75, 85, 99) };
        self.engine.poner_borde(x, y + 1, ancho, alto.saturating_sub(1), borde_color);

        // Contenido interior
        let interior_ancho = ancho.saturating_sub(2);
        let interior_alto = alto.saturating_sub(3);
        if interior_ancho > 0 && interior_alto > 0 {
            self.engine.rellenar(
                x + 1,
                y + 2,
                interior_ancho,
                interior_alto,
                ' ',
                (255, 255, 255),
                (30, 30, 30),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Estado global de ix
// ---------------------------------------------------------------------------
struct IXState {
    desktop: IXDesktop,
}

static IX_STATE: OnceLock<Mutex<IXState>> = OnceLock::new();

fn get_state() -> &'static Mutex<IXState> {
    IX_STATE.get_or_init(|| {
        Mutex::new(IXState {
            desktop: IXDesktop::new(),
        })
    })
}

// ---------------------------------------------------------------------------
// Input
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
                // Alt + Flechas: [27, 91, DIRECCION]
                // Pero stty raw puede solo dar 2 bytes, así que interpretamos
                [91, 65] => "ArrowUp",
                [91, 66] => "ArrowDown",
                [91, 67] => "ArrowRight",
                [91, 68] => "ArrowLeft",
                _ => "Escape",
            }
        }
        9 => "Tab",
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
    // ix.crear_ventana(titulo, ancho, alto, opts?) -> id
    // opts: { x: 10, y: 5, callback: 1 }
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("crear_ventana".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 3 {
                return Objeto::Error(
                    "ix.crear_ventana: se esperaban 3+ argumentos (titulo, ancho, alto)"
                        .to_string(),
                    Vec::new(),
                );
            }
            let titulo = match &args[0] {
                Objeto::Cadena(s) => s.clone(),
                _ => {
                    return Objeto::Error(
                        "ix.crear_ventana: titulo debe ser una cadena".to_string(),
                        Vec::new(),
                    )
                }
            };
            let ancho = match &args[1] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ix.crear_ventana: ancho debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let alto = match &args[2] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ix.crear_ventana: alto debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let mut x = 2;
            let mut y = 2;
            let mut callback_id = None;

            if args.len() > 3
                && let Objeto::Diccionario(opts) = &args[3]
            {
                for (key, val) in opts {
                    if let LlaveHash::Cadena(k) = key {
                        match k.as_str() {
                            "x" => {
                                if let Objeto::Entero(n) = val {
                                    x = *n as usize;
                                }
                            }
                            "y" => {
                                if let Objeto::Entero(n) = val {
                                    y = *n as usize;
                                }
                            }
                            "callback" => {
                                if let Objeto::Entero(n) = val {
                                    callback_id = Some(*n as usize);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            let state = get_state();
            let mut st = state.lock().unwrap();

            let id = st.desktop.siguiente_id();
            let z = st.desktop.siguiente_z();

            let ventana = IXWindow {
                id,
                titulo,
                x,
                y,
                ancho,
                alto,
                z_index: z,
                callback_id,
                visible: true,
            };

            st.desktop.ventanas.push(ventana);
            st.desktop.ventana_foco = Some(id);

            Objeto::Entero(id as i64)
        }),
    );

    // -----------------------------------------------------------------------
    // ix.mover_ventana(id, x, y) — Mueve una ventana
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("mover_ventana".to_string()),
        Objeto::Nativa(|args| {
            if args.len() < 3 {
                return Objeto::Error(
                    "ix.mover_ventana: se esperaban 3 argumentos (id, x, y)".to_string(),
                    Vec::new(),
                );
            }
            let id = match &args[0] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ix.mover_ventana: id debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let x = match &args[1] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ix.mover_ventana: x debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };
            let y = match &args[2] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ix.mover_ventana: y debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            // Extraer datos primero para evitar borrow conflicts
            let old = st.desktop.buscar_ventana(id).map(|v| (v.x, v.y, v.ancho, v.alto));
            if let Some((old_x, old_y, w, h)) = old {
                st.desktop.engine.limpiar_area(old_x, old_y, w, h);
                if let Some(v) = st.desktop.buscar_ventana_mut(id) {
                    v.x = x;
                    v.y = y;
                }
                st.desktop.engine.limpiar_area(x, y, w, h);
            } else {
                return Objeto::Error(
                    format!("ix.mover_ventana: ventana {} no encontrada", id),
                    Vec::new(),
                );
            }

            Objeto::Nulo
        }),
    );

    // -----------------------------------------------------------------------
    // ix.cambiar_foco(id) — Cambia el foco a una ventana
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("cambiar_foco".to_string()),
        Objeto::Nativa(|args| {
            if args.is_empty() {
                return Objeto::Error(
                    "ix.cambiar_foco: se esperaba 1 argumento (id)".to_string(),
                    Vec::new(),
                );
            }
            let id = match &args[0] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ix.cambiar_foco: id debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            if st.desktop.buscar_ventana(id).is_none() {
                return Objeto::Error(
                    format!("ix.cambiar_foco: ventana {} no encontrada", id),
                    Vec::new(),
                );
            }

            st.desktop.ventana_foco = Some(id);
            st.desktop.traer_al_frente(id);

            Objeto::Nulo
        }),
    );

    // -----------------------------------------------------------------------
    // ix.cerrar_ventana(id) — Cierra una ventana
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("cerrar_ventana".to_string()),
        Objeto::Nativa(|args| {
            if args.is_empty() {
                return Objeto::Error(
                    "ix.cerrar_ventana: se esperaba 1 argumento (id)".to_string(),
                    Vec::new(),
                );
            }
            let id = match &args[0] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ix.cerrar_ventana: id debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let mut st = state.lock().unwrap();

            // Extraer datos de la ventana antes de borrar
            let ventana_data = st.desktop.ventanas.iter().find(|v| v.id == id).map(|v| (v.x, v.y, v.ancho, v.alto));
            if let Some((vx, vy, vw, vh)) = ventana_data {
                st.desktop.engine.limpiar_area(vx, vy, vw, vh);
                st.desktop.ventanas.retain(|v| v.id != id);

                // Actualizar foco
                if st.desktop.ventana_foco == Some(id) {
                    st.desktop.ventana_foco = st.desktop.ventanas.last().map(|v| v.id);
                }
            } else {
                return Objeto::Error(
                    format!("ix.cerrar_ventana: ventana {} no encontrada", id),
                    Vec::new(),
                );
            }

            Objeto::Nulo
        }),
    );

    // -----------------------------------------------------------------------
    // ix.ejecutar() — Loop principal del window manager
    //
    // Flechas: mueve ventana con foco
    // Tab: cambia foco a siguiente ventana
    // Enter: dispara callback de ventana con foco
    // q: sale
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("ejecutar".to_string()),
        Objeto::Nativa(|_args| {
            // Render inicial
            {
                let state = get_state();
                let mut st = state.lock().unwrap();
                st.desktop.engine.limpiar_todo();
                st.desktop.dibujar_todas();
                let mut stdout = BufWriter::new(std::io::stdout());
                st.desktop.engine.flush(&mut stdout);
            }

            loop {
                match leer_tecla() {
                    "q" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        st.desktop.engine.limpiar_todo();
                        let mut stdout = BufWriter::new(std::io::stdout());
                        st.desktop.engine.flush(&mut stdout);
                        break;
                    }
                    "Tab" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();

                        // Siguiente ventana por ID
                        let current = st.desktop.ventana_foco;
                        let next = if let Some(cur) = current {
                            if let Some(pos) = st.desktop.ventanas.iter().position(|v| v.id == cur)
                            {
                                let next_pos = (pos + 1) % st.desktop.ventanas.len();
                                Some(st.desktop.ventanas[next_pos].id)
                            } else {
                                st.desktop.ventanas.first().map(|v| v.id)
                            }
                        } else {
                            st.desktop.ventanas.first().map(|v| v.id)
                        };

                        if let Some(nid) = next {
                            st.desktop.ventana_foco = Some(nid);
                            st.desktop.traer_al_frente(nid);
                        }

                        // Re-dibujar todo
                        st.desktop.engine.limpiar_todo();
                        st.desktop.dibujar_todas();
                        let mut stdout = BufWriter::new(std::io::stdout());
                        st.desktop.engine.flush(&mut stdout);
                    }
                    "ArrowUp" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        if let Some(fid) = st.desktop.ventana_foco {
                            let old = st.desktop.buscar_ventana(fid).map(|v| (v.x, v.y, v.ancho, v.alto));
                            if let Some((ox, oy, w, h)) = old {
                                if let Some(v) = st.desktop.buscar_ventana_mut(fid) {
                                    v.y = v.y.saturating_sub(1);
                                }
                                st.desktop.engine.limpiar_area(ox, oy, w, h);
                                let new_pos = st.desktop.buscar_ventana(fid).map(|v| (v.x, v.y));
                                if let Some((nx, ny)) = new_pos {
                                    st.desktop.engine.limpiar_area(nx, ny, w, h);
                                }
                            }
                            st.desktop.engine.limpiar_todo();
                            st.desktop.dibujar_todas();
                            let mut stdout = BufWriter::new(std::io::stdout());
                            st.desktop.engine.flush(&mut stdout);
                        }
                    }
                    "ArrowDown" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        if let Some(fid) = st.desktop.ventana_foco {
                            let old = st.desktop.buscar_ventana(fid).map(|v| (v.x, v.y, v.ancho, v.alto));
                            if let Some((ox, oy, w, h)) = old {
                                if let Some(v) = st.desktop.buscar_ventana_mut(fid) {
                                    v.y += 1;
                                }
                                st.desktop.engine.limpiar_area(ox, oy, w, h);
                                let new_pos = st.desktop.buscar_ventana(fid).map(|v| (v.x, v.y));
                                if let Some((nx, ny)) = new_pos {
                                    st.desktop.engine.limpiar_area(nx, ny, w, h);
                                }
                            }
                            st.desktop.engine.limpiar_todo();
                            st.desktop.dibujar_todas();
                            let mut stdout = BufWriter::new(std::io::stdout());
                            st.desktop.engine.flush(&mut stdout);
                        }
                    }
                    "ArrowLeft" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        if let Some(fid) = st.desktop.ventana_foco {
                            let old = st.desktop.buscar_ventana(fid).map(|v| (v.x, v.y, v.ancho, v.alto));
                            if let Some((ox, oy, w, h)) = old {
                                if let Some(v) = st.desktop.buscar_ventana_mut(fid) {
                                    v.x = v.x.saturating_sub(1);
                                }
                                st.desktop.engine.limpiar_area(ox, oy, w, h);
                                let new_pos = st.desktop.buscar_ventana(fid).map(|v| (v.x, v.y));
                                if let Some((nx, ny)) = new_pos {
                                    st.desktop.engine.limpiar_area(nx, ny, w, h);
                                }
                            }
                            st.desktop.engine.limpiar_todo();
                            st.desktop.dibujar_todas();
                            let mut stdout = BufWriter::new(std::io::stdout());
                            st.desktop.engine.flush(&mut stdout);
                        }
                    }
                    "ArrowRight" => {
                        let state = get_state();
                        let mut st = state.lock().unwrap();
                        if let Some(fid) = st.desktop.ventana_foco {
                            let old = st.desktop.buscar_ventana(fid).map(|v| (v.x, v.y, v.ancho, v.alto));
                            if let Some((ox, oy, w, h)) = old {
                                if let Some(v) = st.desktop.buscar_ventana_mut(fid) {
                                    v.x += 1;
                                }
                                st.desktop.engine.limpiar_area(ox, oy, w, h);
                                let new_pos = st.desktop.buscar_ventana(fid).map(|v| (v.x, v.y));
                                if let Some((nx, ny)) = new_pos {
                                    st.desktop.engine.limpiar_area(nx, ny, w, h);
                                }
                            }
                            st.desktop.engine.limpiar_todo();
                            st.desktop.dibujar_todas();
                            let mut stdout = BufWriter::new(std::io::stdout());
                            st.desktop.engine.flush(&mut stdout);
                        }
                    }
                    "Enter" => {
                        let state = get_state();
                        let st = state.lock().unwrap();
                        if let Some(fid) = st.desktop.ventana_foco
                            && let Some(v) = st.desktop.buscar_ventana(fid)
                            && let Some(cb) = v.callback_id
                        {
                            return Objeto::Entero(cb as i64);
                        }
                    }
                    _ => {}
                }
            }

            Objeto::Nulo
        }),
    );

    // -----------------------------------------------------------------------
    // ix.listar_ventanas() -> Arreglo de ids
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("listar_ventanas".to_string()),
        Objeto::Nativa(|_args| {
            let state = get_state();
            let st = state.lock().unwrap();
            let ids: Vec<Objeto> = st
                .desktop
                .ventanas
                .iter()
                .map(|v| Objeto::Entero(v.id as i64))
                .collect();
            Objeto::Arreglo(ids)
        }),
    );

    // -----------------------------------------------------------------------
    // ix.info_ventana(id) -> Diccionario con info
    // -----------------------------------------------------------------------
    funcs.insert(
        LlaveHash::Cadena("info_ventana".to_string()),
        Objeto::Nativa(|args| {
            if args.is_empty() {
                return Objeto::Error(
                    "ix.info_ventana: se esperaba 1 argumento (id)".to_string(),
                    Vec::new(),
                );
            }
            let id = match &args[0] {
                Objeto::Entero(n) => *n as usize,
                _ => {
                    return Objeto::Error(
                        "ix.info_ventana: id debe ser un entero".to_string(),
                        Vec::new(),
                    )
                }
            };

            let state = get_state();
            let st = state.lock().unwrap();

            let v = match st.desktop.buscar_ventana(id) {
                Some(v) => v,
                None => {
                    return Objeto::Error(
                        format!("ix.info_ventana: ventana {} no encontrada", id),
                        Vec::new(),
                    )
                }
            };

            let mut info = std::collections::HashMap::new();
            info.insert(
                LlaveHash::Cadena("titulo".to_string()),
                Objeto::Cadena(v.titulo.clone()),
            );
            info.insert(LlaveHash::Cadena("x".to_string()), Objeto::Entero(v.x as i64));
            info.insert(LlaveHash::Cadena("y".to_string()), Objeto::Entero(v.y as i64));
            info.insert(
                LlaveHash::Cadena("ancho".to_string()),
                Objeto::Entero(v.ancho as i64),
            );
            info.insert(
                LlaveHash::Cadena("alto".to_string()),
                Objeto::Entero(v.alto as i64),
            );
            info.insert(
                LlaveHash::Cadena("z_index".to_string()),
                Objeto::Entero(v.z_index as i64),
            );

            Objeto::Diccionario(info)
        }),
    );

    Objeto::Diccionario(funcs)
}
