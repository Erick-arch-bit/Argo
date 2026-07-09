// ---------------------------------------------------------------------------
// Theme — Colores globales de la UI
// ---------------------------------------------------------------------------
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
            bg: (43, 43, 43),         // #2b2b2b
            fg: (255, 255, 255),      // #ffffff
            primary: (59, 130, 246),   // #3b82f6
            secondary: (107, 114, 128), // #6b7280
            accent: (16, 185, 129),    // #10b981
            border: (75, 85, 99),      // #4b5563
            text_bg: (31, 41, 55),     // #1f2937
            error: (239, 68, 68),      // #ef4444
        }
    }
}

// ---------------------------------------------------------------------------
// DatosWidget — Qué dibujar
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
    Input {
        placeholder: String,
    },
    Separador,
}

// ---------------------------------------------------------------------------
// NodoUI — Nodo en el árbol de widgets
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum TipoLayout {
    Fila,
    Columna,
    Area,
}

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
// UiState — Estado global de la UI
// ---------------------------------------------------------------------------
pub struct UiState {
    pub raiz: Option<NodoUI>,
    pub contador_ids: usize,
    pub theme: Theme,
    pub foco_id: Option<usize>,
    pub widgets_focusable: Vec<usize>,
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            raiz: None,
            contador_ids: 0,
            theme: Theme::default(),
            foco_id: None,
            widgets_focusable: Vec::new(),
        }
    }

    pub fn siguiente_id(&mut self) -> usize {
        self.contador_ids += 1;
        self.contador_ids
    }

    pub fn buscar_nodo_mut(nodo: &mut NodoUI, id: usize) -> Option<&mut NodoUI> {
        if nodo.id == id {
            return Some(nodo);
        }
        for hijo in &mut nodo.hijos {
            if let Some(encontrado) = Self::buscar_nodo_mut(hijo, id) {
                return Some(encontrado);
            }
        }
        None
    }

    pub fn recoger_focusables(&mut self) {
        self.widgets_focusable.clear();
        if let Some(ref raiz) = self.raiz {
            Self::recoger_focusables_interno(raiz, &mut self.widgets_focusable);
        }
    }

    fn recoger_focusables_interno(nodo: &NodoUI, ids: &mut Vec<usize>) {
        if let Some(DatosWidget::Boton { .. } | DatosWidget::Input { .. }) = &nodo.widget {
            ids.push(nodo.id);
        }
        for hijo in &nodo.hijos {
            Self::recoger_focusables_interno(hijo, ids);
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
        if let Some(ref raiz) = self.raiz {
            Self::buscar_callback(raiz, foco)
        } else {
            None
        }
    }

    fn buscar_callback(nodo: &NodoUI, id: usize) -> Option<usize> {
        if nodo.id == id
            && let Some(DatosWidget::Boton { id_callback, .. }) = &nodo.widget
        {
            return Some(*id_callback);
        }
        for hijo in &nodo.hijos {
            if let Some(cb) = Self::buscar_callback(hijo, id) {
                return Some(cb);
            }
        }
        None
    }
}
