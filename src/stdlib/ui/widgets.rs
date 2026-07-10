use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Theme — Colores globales de la UI (con soporte para nombres y hex)
// ---------------------------------------------------------------------------
#[derive(Clone)]
pub struct Theme {
    pub bg: (u8, u8, u8),
    pub fg: (u8, u8, u8),
    pub primary: (u8, u8, u8),
    pub secondary: (u8, u8, u8),
    pub accent: (u8, u8, u8),
    pub border: (u8, u8, u8),
    pub text_bg: (u8, u8, u8),
    pub error: (u8, u8, u8),
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            bg: (43, 43, 43),
            fg: (200, 200, 200),
            primary: (59, 130, 246),
            secondary: (107, 114, 128),
            accent: (16, 185, 129),
            border: (75, 85, 99),
            text_bg: (31, 41, 55),
            error: (239, 68, 68),
        }
    }
}

// ---------------------------------------------------------------------------
// parsear_color — Convierte "#RRGGBB" o nombres a RGB
// ---------------------------------------------------------------------------
pub fn parsear_color(s: &str) -> (u8, u8, u8) {
    let lower = s.trim().to_lowercase();

    // Intentar hex primero
    if let Some(rgb) = parsear_hex(&lower) {
        return rgb;
    }

    // Nombres comunes
    match lower.as_str() {
        "rojo" | "red" => (220, 50, 47),
        "verde" | "green" => (39, 174, 96),
        "azul" | "blue" => (38, 139, 216),
        "amarillo" | "yellow" => (255, 200, 0),
        "naranja" | "orange" => (255, 165, 0),
        "morado" | "purple" => (168, 130, 255),
        "rosa" | "pink" => (255, 105, 180),
        "cian" | "cyan" => (0, 200, 200),
        "blanco" | "white" => (255, 255, 255),
        "negro" | "black" => (0, 0, 0),
        "gris" | "gray" | "grey" => (128, 128, 128),
        _ => (200, 200, 200),
    }
}

fn parsear_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some((r, g, b))
}

// ---------------------------------------------------------------------------
// DatosWidget — Qué dibujar en un nodo hoja
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum DatosWidget {
    Texto {
        valor: String,
        color: Option<(u8, u8, u8)>,
        grande: bool,
    },
    Boton {
        texto: String,
        id_callback: usize,
    },
    Rectangulo {
        color: (u8, u8, u8),
        relleno: bool,
    },
    BarraProgreso {
        valor: f32,
        color: (u8, u8, u8),
    },
    Input {
        placeholder: String,
    },
    Separador,
}

// ---------------------------------------------------------------------------
// TipoLayout — Tipo de nodo en el árbol
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum TipoLayout {
    Fila,
    Columna,
    Area,
}

// ---------------------------------------------------------------------------
// NodoUI — Nodo en el árbol de widgets
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct NodoUI {
    pub id: usize,
    pub tipo: TipoLayout,
    pub hijos: Vec<NodoUI>,
    pub ancho: usize,
    pub alto: usize,
    pub gap: usize,
    pub widget: Option<DatosWidget>,
}

impl NodoUI {
    pub fn new(id: usize, tipo: TipoLayout) -> Self {
        NodoUI {
            id,
            tipo,
            hijos: Vec::new(),
            ancho: 0,
            alto: 0,
            gap: 0,
            widget: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Bounds — Resultado del layout para un nodo
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
pub struct Bounds {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

// ---------------------------------------------------------------------------
// ArbolUI — Árbol de declaraciones con layout Flexbox simplificado
// ---------------------------------------------------------------------------
pub struct ArbolUI {
    pub nodos: HashMap<usize, NodoUI>,
    pub raiz: usize,
    pub contador_ids: usize,
    pub theme: Theme,
    pub foco_id: Option<usize>,
    pub widgets_focusable: Vec<usize>,
    pub bounds: HashMap<usize, Bounds>,
}

impl Default for ArbolUI {
    fn default() -> Self {
        Self::new()
    }
}

impl ArbolUI {
    pub fn new() -> Self {
        ArbolUI {
            nodos: HashMap::new(),
            raiz: 0,
            contador_ids: 0,
            theme: Theme::default(),
            foco_id: None,
            widgets_focusable: Vec::new(),
            bounds: HashMap::new(),
        }
    }

    pub fn siguiente_id(&mut self) -> usize {
        self.contador_ids += 1;
        self.contador_ids
    }

    pub fn insertar_nodo(&mut self, id_padre: usize, nodo: NodoUI) -> Result<usize, String> {
        let id = nodo.id;
        if id_padre == 0 {
            // Si ya hay raíz, mover la raíz actual como hijo de la nueva
            if self.raiz != 0 && self.nodos.contains_key(&self.raiz) {
                let raiz_antigua = self.nodos.remove(&self.raiz).unwrap();
                let mut nodo_copia = nodo.clone();
                nodo_copia.hijos.push(raiz_antigua);
                self.nodos.insert(id, nodo_copia);
                self.raiz = id;
            } else {
                self.nodos.insert(id, nodo);
                self.raiz = id;
            }
        } else if let Some(padre) = self.nodos.get_mut(&id_padre) {
            padre.hijos.push(nodo);
        } else {
            return Err(format!("ui: nodo padre {} no encontrado", id_padre));
        }
        Ok(id)
    }

    pub fn recoger_focusables(&mut self) {
        self.widgets_focusable.clear();
        let raiz = self.raiz;
        if raiz != 0 {
            Self::recoger_focusables_interno_static(&self.nodos, raiz, &mut self.widgets_focusable);
        }
    }

    fn recoger_focusables_interno_static(
        nodos: &HashMap<usize, NodoUI>,
        id: usize,
        ids: &mut Vec<usize>,
    ) {
        if let Some(nodo) = nodos.get(&id) {
            if let Some(DatosWidget::Boton { .. } | DatosWidget::Input { .. }) = &nodo.widget {
                ids.push(nodo.id);
            }
            for hijo in &nodo.hijos {
                Self::recoger_focusables_interno_static(nodos, hijo.id, ids);
            }
        }
    }

    pub fn siguiente_foco(&mut self) {
        if self.widgets_focusable.is_empty() {
            return;
        }
        let idx = match self.foco_id {
            Some(id) => self
                .widgets_focusable
                .iter()
                .position(|&wid| wid == id)
                .map(|i| (i + 1) % self.widgets_focusable.len())
                .unwrap_or(0),
            None => 0,
        };
        self.foco_id = Some(self.widgets_focusable[idx]);
    }

    pub fn anterior_foco(&mut self) {
        if self.widgets_focusable.is_empty() {
            return;
        }
        let idx = match self.foco_id {
            Some(id) => self
                .widgets_focusable
                .iter()
                .position(|&wid| wid == id)
                .map(|i| if i == 0 { self.widgets_focusable.len() - 1 } else { i - 1 })
                .unwrap_or(0),
            None => 0,
        };
        self.foco_id = Some(self.widgets_focusable[idx]);
    }

    pub fn callback_foco(&self) -> Option<usize> {
        let foco = self.foco_id?;
        Self::buscar_callback_static(&self.nodos, self.raiz, foco)
    }

    fn buscar_callback_static(
        nodos: &HashMap<usize, NodoUI>,
        id_actual: usize,
        id_buscado: usize,
    ) -> Option<usize> {
        if let Some(nodo) = nodos.get(&id_actual) {
            if nodo.id == id_buscado {
                if let Some(DatosWidget::Boton { id_callback, .. }) = &nodo.widget {
                    return Some(*id_callback);
                }
            }
            for hijo in &nodo.hijos {
                if let Some(cb) = Self::buscar_callback_static(nodos, hijo.id, id_buscado) {
                    return Some(cb);
                }
            }
        }
        None
    }

    /// Calcula layout recursivamente. Flexbox simplificado:
    /// - Columna: apila hijos verticalmente con gap
    /// - Fila: coloca hijos horizontalmente con gap
    /// - Area: nodo hoja con tamaño fijo
    pub fn calcular_layout(&mut self, max_ancho: usize) {
        self.bounds.clear();
        if self.raiz != 0 {
            self.calcular_nodo(self.raiz, 1, 1, max_ancho);
        }
    }

    fn calcular_nodo(&mut self, id: usize, x: usize, y: usize, max_ancho: usize) {
        let nodo = match self.nodos.get(&id) {
            Some(n) => n.clone(),
            None => return,
        };

        match nodo.tipo {
            TipoLayout::Columna => {
                let mut child_y = y;
                for hijo in &nodo.hijos {
                    self.calcular_nodo(hijo.id, x, child_y, max_ancho);
                    if let Some(hijo_bounds) = self.bounds.get(&hijo.id) {
                        child_y += hijo_bounds.h + nodo.gap;
                    }
                }
                let total_h = child_y.saturating_sub(y).saturating_sub(nodo.gap);
                self.bounds.insert(id, Bounds { x, y, w: max_ancho, h: total_h });
            }
            TipoLayout::Fila => {
                let mut child_x = x;
                let mut max_h = 0usize;
                for hijo in &nodo.hijos {
                    self.calcular_nodo(hijo.id, child_x, y, max_ancho.saturating_sub(child_x - x));
                    if let Some(hijo_bounds) = self.bounds.get(&hijo.id) {
                        child_x += hijo_bounds.w + nodo.gap;
                        if hijo_bounds.h > max_h {
                            max_h = hijo_bounds.h;
                        }
                    }
                }
                let total_w = child_x.saturating_sub(x).saturating_sub(nodo.gap);
                self.bounds.insert(id, Bounds { x, y, w: total_w, h: max_h });
            }
            TipoLayout::Area => {
                // Nodo hoja: tamaño determinado por el widget
                let (w, h) = self.tamanio_widget(&nodo);
                self.bounds.insert(id, Bounds { x, y, w, h });
            }
        }
    }

    fn tamanio_widget(&self, nodo: &NodoUI) -> (usize, usize) {
        match &nodo.widget {
            Some(DatosWidget::Texto { valor, grande, .. }) => {
                let len = valor.chars().count();
                let h = if *grande { 2 } else { 1 };
                (len.max(1), h)
            }
            Some(DatosWidget::Boton { texto, .. }) => {
                let len = texto.chars().count();
                (len + 4, 3) // [ texto ]
            }
            Some(DatosWidget::Rectangulo { .. }) => {
                (nodo.ancho.max(1), nodo.alto.max(1))
            }
            Some(DatosWidget::BarraProgreso { .. }) => {
                (nodo.ancho.max(20), 3)
            }
            Some(DatosWidget::Input { placeholder }) => {
                let len = placeholder.chars().count().max(10);
                (len + 4, 3)
            }
            Some(DatosWidget::Separador) => {
                (nodo.ancho.max(20), 1)
            }
            None => (0, 0),
        }
    }
}
