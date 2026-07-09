use crate::stdlib::render::RenderEngine;
use crate::stdlib::ui::widgets::{self, DatosWidget, NodoUI, TipoLayout};

// ---------------------------------------------------------------------------
// ResultadoLayout — Coordenadas calculadas para cada nodo
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct ResultadoLayout {
    pub x: usize,
    pub y: usize,
    pub ancho: usize,
    pub alto: usize,
}

/// Calcula las posiciones de todos los nodos del árbol.
pub fn calcular_layout(
    nodo: &NodoUI,
    x: usize,
    y: usize,
    max_ancho: usize,
) -> Vec<(usize, ResultadoLayout)> {
    let mut resultados = Vec::new();
    calcular_interno(nodo, x, y, max_ancho, &mut resultados);
    resultados
}

fn calcular_interno(
    nodo: &NodoUI,
    x: usize,
    y: usize,
    max_ancho: usize,
    resultados: &mut Vec<(usize, ResultadoLayout)>,
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

            resultados.push((nodo.id, ResultadoLayout { x, y, ancho, alto }));

            if !nodo.hijos.is_empty() {
                let hijo = &nodo.hijos[0];
                calcular_interno(
                    hijo,
                    x + 1,
                    y + 1,
                    ancho.saturating_sub(2),
                    resultados,
                );
            }

            (ancho, alto)
        }

        TipoLayout::Fila => {
            let mut total_ancho = 0;
            let mut max_alto = 0;

            for hijo in &nodo.hijos {
                let (ha, halto) = calcular_interno(
                    hijo,
                    x + total_ancho,
                    y,
                    max_ancho.saturating_sub(total_ancho),
                    resultados,
                );
                total_ancho += ha + nodo.gap;
                max_alto = max_alto.max(halto);
            }

            if !nodo.hijos.is_empty() && nodo.gap > 0 {
                total_ancho = total_ancho.saturating_sub(nodo.gap);
            }

            resultados.push((nodo.id, ResultadoLayout { x, y, ancho: total_ancho, alto: max_alto }));
            (total_ancho, max_alto)
        }

        TipoLayout::Columna => {
            let mut max_ancho_hijos = 0;
            let mut total_alto = 0;

            for hijo in &nodo.hijos {
                let (ha, halto) = calcular_interno(hijo, x, y + total_alto, max_ancho, resultados);
                max_ancho_hijos = max_ancho_hijos.max(ha);
                total_alto += halto + nodo.gap;
            }

            if !nodo.hijos.is_empty() && nodo.gap > 0 {
                total_alto = total_alto.saturating_sub(nodo.gap);
            }

            resultados.push((nodo.id, ResultadoLayout { x, y, ancho: max_ancho_hijos, alto: total_alto }));
            (max_ancho_hijos, total_alto)
        }
    }
}

/// Renderiza todo el árbol de widgets en el RenderEngine.
pub fn dibujar_arbol(
    nodo: &NodoUI,
    resultados: &[(usize, ResultadoLayout)],
    engine: &mut RenderEngine,
    theme: &widgets::Theme,
    foco_id: Option<usize>,
) {
    dibujar_nodo(nodo, resultados, engine, theme, foco_id);
}

fn dibujar_nodo(
    nodo: &NodoUI,
    resultados: &[(usize, ResultadoLayout)],
    engine: &mut RenderEngine,
    theme: &widgets::Theme,
    foco_id: Option<usize>,
) {
    if let Some((_, res)) = resultados.iter().find(|(id, _)| *id == nodo.id)
        && let Some(ref widget) = nodo.widget
    {
        let tiene_foco = foco_id == Some(nodo.id);

            match widget {
                DatosWidget::Texto { valor, color, grande } => {
                    let fg = color.unwrap_or(theme.fg);
                    if *grande {
                        // Negrita: envolver en códigos ANSI (esto es solo visual,
                        // el render engine no soporta atributos, usamos color accent)
                        engine.poner_texto(res.x, res.y, valor, theme.accent, theme.bg);
                    } else {
                        engine.poner_texto(res.x, res.y, valor, fg, theme.bg);
                    }
                }
                DatosWidget::Boton { texto, .. } => {
                    let (fx, fy, fw, fh) = (res.x, res.y, res.ancho, res.alto);

                    if tiene_foco {
                        // Foco: borde redondeado con color primary
                        engine.poner_borde_redondeado(fx, fy, fw, fh, theme.fg);
                        engine.poner_texto_centrado(fx, fy + 1, fw, texto, theme.bg, theme.fg);
                    } else {
                        // Sin foco: rectángulo sólido
                        engine.poner_rectangulo(fx, fy, fw, fh, theme.primary);
                        engine.poner_texto_centrado(fx, fy + 1, fw, texto, theme.fg, theme.primary);
                    }
                }
                DatosWidget::Input { placeholder } => {
                    let (fx, fy, fw, fh) = (res.x, res.y, res.ancho, res.alto);
                    engine.poner_borde(fx, fy, fw, fh, theme.border);
                    let interior = format!("{:<width$}", placeholder, width = fw.saturating_sub(2));
                    engine.poner_texto(fx + 1, fy + 1, &interior, theme.secondary, theme.text_bg);
                }
                DatosWidget::Separador => {
                    let (fx, fy, fw, _fh) = (res.x, res.y, res.ancho, res.alto);
                    engine.rellenar(fx, fy, fw, 1, '─', theme.border, theme.bg);
                }
            }
    }

    for hijo in &nodo.hijos {
        dibujar_nodo(hijo, resultados, engine, theme, foco_id);
    }
}
