use std::collections::HashMap;
use crate::stdlib::render::RenderEngine;
use crate::stdlib::ui::widgets::{self, DatosWidget, NodoUI, TipoLayout};

// ---------------------------------------------------------------------------
// ResultadoLayout — Coordenadas calculadas para cada nodo
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
pub struct ResultadoLayout {
    pub x: usize,
    pub y: usize,
    pub ancho: usize,
    pub alto: usize,
}

// ---------------------------------------------------------------------------
// LayoutCache — Reutiliza buffers entre frames
// ---------------------------------------------------------------------------
pub struct LayoutCache {
    pub resultados: HashMap<usize, ResultadoLayout>,
}

impl Default for LayoutCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutCache {
    pub fn new() -> Self {
        LayoutCache {
            resultados: HashMap::with_capacity(64),
        }
    }

    pub fn limpiar(&mut self) {
        self.resultados.clear();
    }
}

/// Calcula las posiciones de todos los nodos del árbol.
/// Escribe directamente en el cache (HashMap) en lugar de Vec de tuples.
pub fn calcular_layout(
    nodo: &NodoUI,
    x: usize,
    y: usize,
    max_ancho: usize,
    cache: &mut LayoutCache,
) {
    cache.limpiar();
    calcular_interno(nodo, x, y, max_ancho, cache);
}

#[inline]
fn calcular_interno(
    nodo: &NodoUI,
    x: usize,
    y: usize,
    max_ancho: usize,
    cache: &mut LayoutCache,
) -> (usize, usize) {
    let (widget_ancho, widget_alto) = match &nodo.widget {
        Some(DatosWidget::Texto { valor, .. }) => (valor.len() + 2, 1),
        Some(DatosWidget::Boton { texto, .. }) => (texto.len() + 4, 3),
        Some(DatosWidget::Input { placeholder }) => (placeholder.len().max(20) + 4, 3),
        Some(DatosWidget::Separador) => (max_ancho.min(40), 1),
        None => (0, 0),
    };

    match nodo.tipo {
        TipoLayout::Area => {
            let ancho = if nodo.ancho > 0 { nodo.ancho } else { widget_ancho };
            let alto = if nodo.alto > 0 { nodo.alto } else { widget_alto };

            cache.resultados.insert(nodo.id, ResultadoLayout { x, y, ancho, alto });

            if !nodo.hijos.is_empty() {
                calcular_interno(&nodo.hijos[0], x + 1, y + 1, ancho.saturating_sub(2), cache);
            }

            (ancho, alto)
        }

        TipoLayout::Fila => {
            let mut total_ancho = 0;
            let mut max_alto = 0;

            for hijo in &nodo.hijos {
                let (ha, halto) = calcular_interno(hijo, x + total_ancho, y, max_ancho.saturating_sub(total_ancho), cache);
                total_ancho += ha + nodo.gap;
                max_alto = max_alto.max(halto);
            }

            if !nodo.hijos.is_empty() && nodo.gap > 0 {
                total_ancho = total_ancho.saturating_sub(nodo.gap);
            }

            cache.resultados.insert(nodo.id, ResultadoLayout { x, y, ancho: total_ancho, alto: max_alto });
            (total_ancho, max_alto)
        }

        TipoLayout::Columna => {
            let mut max_ancho_hijos = 0;
            let mut total_alto = 0;

            for hijo in &nodo.hijos {
                let (ha, halto) = calcular_interno(hijo, x, y + total_alto, max_ancho, cache);
                max_ancho_hijos = max_ancho_hijos.max(ha);
                total_alto += halto + nodo.gap;
            }

            if !nodo.hijos.is_empty() && nodo.gap > 0 {
                total_alto = total_alto.saturating_sub(nodo.gap);
            }

            cache.resultados.insert(nodo.id, ResultadoLayout { x, y, ancho: max_ancho_hijos, alto: total_alto });
            (max_ancho_hijos, total_alto)
        }
    }
}

/// Renderiza todo el árbol de widgets en el RenderEngine.
#[inline]
pub fn dibujar_arbol(
    nodo: &NodoUI,
    cache: &LayoutCache,
    engine: &mut RenderEngine,
    theme: &widgets::Theme,
    foco_id: Option<usize>,
) {
    dibujar_nodo(nodo, cache, engine, theme, foco_id);
}

fn dibujar_nodo(
    nodo: &NodoUI,
    cache: &LayoutCache,
    engine: &mut RenderEngine,
    theme: &widgets::Theme,
    foco_id: Option<usize>,
) {
    if let Some(res) = cache.resultados.get(&nodo.id)
        && let Some(ref widget) = nodo.widget
    {
        let tiene_foco = foco_id == Some(nodo.id);

        match widget {
            DatosWidget::Texto { valor, color, grande } => {
                let fg = color.unwrap_or(theme.fg);
                let (fg_r, fg_g, fg_b) = if *grande { theme.accent } else { fg };
                engine.poner_texto(res.x, res.y, valor, (fg_r, fg_g, fg_b), theme.bg);
            }
            DatosWidget::Boton { texto, .. } => {
                let fx = res.x;
                let fy = res.y;
                let fw = res.ancho;
                let fh = res.alto;

                if tiene_foco {
                    engine.poner_borde_redondeado(fx, fy, fw, fh, theme.fg);
                    engine.poner_texto_centrado(fx, fy + 1, fw, texto, theme.bg, theme.fg);
                } else {
                    engine.poner_rectangulo(fx, fy, fw, fh, theme.primary);
                    engine.poner_texto_centrado(fx, fy + 1, fw, texto, theme.fg, theme.primary);
                }
            }
            DatosWidget::Input { placeholder } => {
                let fx = res.x;
                let fy = res.y;
                let fw = res.ancho;
                let fh = res.alto;
                engine.poner_borde(fx, fy, fw, fh, theme.border);
                let interior_w = fw.saturating_sub(2);
                if interior_w > 0 {
                    let truncado: String = placeholder.chars().take(interior_w).collect();
                    let pad = interior_w.saturating_sub(truncado.len());
                    let mut interior = String::with_capacity(interior_w);
                    interior.push_str(&truncado);
                    for _ in 0..pad {
                        interior.push(' ');
                    }
                    engine.poner_texto(fx + 1, fy + 1, &interior, theme.secondary, theme.text_bg);
                }
            }
            DatosWidget::Separador => {
                engine.rellenar(res.x, res.y, res.ancho, 1, '─', theme.border, theme.bg);
            }
        }
    }

    for hijo in &nodo.hijos {
        dibujar_nodo(hijo, cache, engine, theme, foco_id);
    }
}
