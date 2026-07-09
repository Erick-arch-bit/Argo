#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TipoWidget {
    Ventana,
    Boton,
    Texto,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Widget {
    pub id: usize,
    pub tipo: TipoWidget,
    pub x: usize,
    pub y: usize,
    pub ancho: usize,
    pub alto: usize,
    pub texto: String,
    pub callback_id: Option<usize>,
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub struct UiState {
    pub widgets: Vec<Widget>,
    pub contador_ids: usize,
    pub ultimo_boton: Option<usize>,
    pub foco: Option<usize>,
}

#[allow(dead_code)]
impl UiState {
    pub fn new() -> Self {
        UiState {
            widgets: Vec::new(),
            contador_ids: 0,
            ultimo_boton: None,
            foco: None,
        }
    }

    pub fn siguiente_id(&mut self) -> usize {
        self.contador_ids += 1;
        self.contador_ids
    }

    pub fn agregar_widget(&mut self, widget: Widget) {
        if matches!(widget.tipo, TipoWidget::Boton) {
            self.ultimo_boton = Some(widget.id);
        }
        self.widgets.push(widget);
    }

    pub fn obtener_widget(&self, id: usize) -> Option<&Widget> {
        self.widgets.iter().find(|w| w.id == id)
    }

    pub fn obtener_callback_ultimo_boton(&self) -> Option<usize> {
        self.ultimo_boton.and_then(|id| {
            self.obtener_widget(id)
                .and_then(|w| w.callback_id)
        })
    }
}
