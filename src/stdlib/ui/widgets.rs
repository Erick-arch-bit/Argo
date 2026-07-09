// ---------------------------------------------------------------------------
// Widgets — Sistema de árbol de widgets para TUI
// ---------------------------------------------------------------------------

/// Tipo de layout de un nodo.
#[derive(Debug, Clone, PartialEq)]
pub enum TipoLayout {
    /// Fila: hijos lado a lado (horizontal).
    Fila,
    /// Columna: hijos apilados (vertical).
    Columna,
    /// Area: contenedor con tamaño fijo, hijo único.
    Area,
}

/// Datos de un widget que se dibuja en pantalla.
#[derive(Debug, Clone)]
pub enum DatosWidget {
    Texto {
        valor: String,
        color: (u8, u8, u8),
    },
    Boton {
        texto: String,
        id_callback: usize,
    },
    Input {
        placeholder: String,
    },
}

/// Nodo en el árbol de UI.
#[derive(Debug, Clone)]
pub struct NodoUI {
    pub id: usize,
    pub tipo: TipoLayout,
    pub hijos: Vec<NodoUI>,
    pub ancho: usize,
    pub alto: usize,
    pub margen: usize,
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
            margen: 0,
            gap: 0,
            widget: None,
        }
    }

    pub fn agregar_hijo(&mut self, hijo: NodoUI) {
        self.hijos.push(hijo);
    }
}

// ---------------------------------------------------------------------------
// UiState — Estado global del sistema de UI
// ---------------------------------------------------------------------------
pub struct UiState {
    pub raiz: Option<NodoUI>,
    pub contador_ids: usize,
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
            foco_id: None,
            widgets_focusable: Vec::new(),
        }
    }

    pub fn siguiente_id(&mut self) -> usize {
        self.contador_ids += 1;
        self.contador_ids
    }

    /// Busca un nodo por ID en el árbol (recursivo).
    pub fn buscar_nodo_mut(
        nodo: &mut NodoUI,
        id: usize,
    ) -> Option<&mut NodoUI> {
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

    /// Recoge todos los IDs focusable (botones, inputs) en orden.
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

    /// Avanza el foco al siguiente widget.
    pub fn siguiente_foco(&mut self) {
        if self.widgets_focusable.is_empty() {
            return;
        }
        let actual = self.foco_id;
        let idx = match actual {
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

    /// Retrocede el foco al widget anterior.
    pub fn anterior_foco(&mut self) {
        if self.widgets_focusable.is_empty() {
            return;
        }
        let actual = self.foco_id;
        let idx = match actual {
            Some(id) => self
                .widgets_focusable
                .iter()
                .position(|&wid| wid == id)
                .map(|i| {
                    if i == 0 {
                        self.widgets_focusable.len() - 1
                    } else {
                        i - 1
                    }
                })
                .unwrap_or(0),
            None => 0,
        };
        self.foco_id = Some(self.widgets_focusable[idx]);
    }

    /// Retorna el callback_id del widget con foco (si es botón).
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
