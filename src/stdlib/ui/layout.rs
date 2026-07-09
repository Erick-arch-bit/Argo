use crate::stdlib::ui::widgets::{DatosWidget, NodoUI, TipoLayout};

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

// ---------------------------------------------------------------------------
// calcular_layout — Motor de layout recursivo
// ---------------------------------------------------------------------------

/// Calcula las posiciones y tamaños de todos los nodos del árbol.
/// Retorna un vector plano de (nodo_id, resultado) para dibujar.
pub fn calcular_layout(
    nodo: &NodoUI,
    x: usize,
    y: usize,
    max_ancho: usize,
) -> Vec<(usize, ResultadoLayout)> {
    let mut resultados = Vec::new();
    calcular_layout_interno(nodo, x, y, max_ancho, &mut resultados);
    resultados
}

fn calcular_layout_interno(
    nodo: &NodoUI,
    x: usize,
    y: usize,
    max_ancho: usize,
    resultados: &mut Vec<(usize, ResultadoLayout)>,
) -> (usize, usize) {
    // Si tiene widget propio, calcular su tamaño base
    let (widget_ancho, widget_alto) = match &nodo.widget {
        Some(DatosWidget::Texto { valor, .. }) => (valor.len() + 2, 1), // +2 margen
        Some(DatosWidget::Boton { texto, .. }) => (texto.len() + 4, 1), // [ texto ]
        Some(DatosWidget::Input { placeholder }) => {
            let w = placeholder.len() + 4;
            (w.max(20), 3) // mínimo 20 cols, 3 filas (borde + input + borde)
        }
        None => (0, 0),
    };

    match nodo.tipo {
        TipoLayout::Area => {
            // Area: tamaño fijo, hijo único
            let ancho = if nodo.ancho > 0 {
                nodo.ancho
            } else {
                widget_ancho
            };
            let alto = if nodo.alto > 0 {
                nodo.alto
            } else {
                widget_alto
            };

            resultados.push((
                nodo.id,
                ResultadoLayout {
                    x,
                    y,
                    ancho,
                    alto,
                },
            ));

            // Calcular hijo si existe
            if !nodo.hijos.is_empty() {
                let hijo = &nodo.hijos[0];
                calcular_layout_interno(
                    hijo,
                    x + nodo.margen,
                    y + nodo.margen,
                    ancho.saturating_sub(nodo.margen * 2),
                    resultados,
                );
            }

            (ancho, alto)
        }

        TipoLayout::Fila => {
            // Fila: hijos lado a lado
            let mut total_ancho = 0;
            let mut max_alto = 0;

            // Primera pasada: calcular tamaños
            let mut tamaños = Vec::new();
            for hijo in &nodo.hijos {
                let (ha, halto) = calcular_layout_interno(
                    hijo,
                    x + total_ancho,
                    y,
                    max_ancho.saturating_sub(total_ancho),
                    resultados,
                );
                tamaños.push((ha, halto));
                total_ancho += ha + nodo.gap;
                max_alto = max_alto.max(halto);
            }

            // Descontar el último gap
            if !nodo.hijos.is_empty() && nodo.gap > 0 {
                total_ancho = total_ancho.saturating_sub(nodo.gap);
            }

            resultados.push((
                nodo.id,
                ResultadoLayout {
                    x,
                    y,
                    ancho: total_ancho,
                    alto: max_alto,
                },
            ));

            (total_ancho, max_alto)
        }

        TipoLayout::Columna => {
            // Columna: hijos apilados
            let mut max_ancho_hijos = 0;
            let mut total_alto = 0;

            for hijo in &nodo.hijos {
                let (ha, halto) = calcular_layout_interno(
                    hijo,
                    x,
                    y + total_alto,
                    max_ancho,
                    resultados,
                );
                max_ancho_hijos = max_ancho_hijos.max(ha);
                total_alto += halto + nodo.gap;
            }

            // Descontar el último gap
            if !nodo.hijos.is_empty() && nodo.gap > 0 {
                total_alto = total_alto.saturating_sub(nodo.gap);
            }

            resultados.push((
                nodo.id,
                ResultadoLayout {
                    x,
                    y,
                    ancho: max_ancho_hijos,
                    alto: total_alto,
                },
            ));

            (max_ancho_hijos, total_alto)
        }
    }
}

// ---------------------------------------------------------------------------
// dibujar_arbol — Renderiza todo el árbol en el buffer
// ---------------------------------------------------------------------------

use crate::stdlib::ui::buffer::PantallaBuffer;

/// Recorre el árbol y dibuja cada widget en el buffer.
pub fn dibujar_arbol(
    nodo: &NodoUI,
    resultados: &[(usize, ResultadoLayout)],
    buffer: &mut PantallaBuffer,
    foco_id: Option<usize>,
) {
    dibujar_nodo(nodo, resultados, buffer, foco_id);
}

fn dibujar_nodo(
    nodo: &NodoUI,
    resultados: &[(usize, ResultadoLayout)],
    buffer: &mut PantallaBuffer,
    foco_id: Option<usize>,
) {
    // Buscar resultado de este nodo
    if let Some((_, res)) = resultados.iter().find(|(id, _)| *id == nodo.id)
        && let Some(ref widget) = nodo.widget
    {
        let tiene_foco = foco_id == Some(nodo.id);

        match widget {
            DatosWidget::Texto { valor, color } => {
                let (r, g, b) = *color;
                buffer.dibujar_texto(res.x, res.y, valor, (0, 0, 0), (r, g, b));
            }
            DatosWidget::Boton { texto, .. } => {
                let (fx, fy, fw, fh) = (res.x, res.y, res.ancho, res.alto);

                if tiene_foco {
                    // Foco: colores invertidos
                    buffer.dibujar_rectangulo(fx, fy, fw, fh, (255, 255, 255));
                    let etiqueta = format!("[{}]", texto);
                    buffer.dibujar_texto(
                        fx + 1,
                        fy,
                        &etiqueta,
                        (255, 255, 255),
                        (0, 0, 0),
                    );
                } else {
                    // Sin foco: fondo oscuro, texto claro
                    buffer.dibujar_rectangulo(fx, fy, fw, fh, (30, 60, 120));
                    let etiqueta = format!(" {} ", texto);
                    buffer.dibujar_texto(
                        fx + 1,
                        fy,
                        &etiqueta,
                        (30, 60, 120),
                        (255, 255, 255),
                    );
                }
            }
            DatosWidget::Input { placeholder } => {
                let (fx, fy, fw, fh) = (res.x, res.y, res.ancho, res.alto);

                // Borde
                buffer.dibujar_borde(fx, fy, fw, fh, (0, 0, 0), (100, 100, 100));
                // Interior
                let interior = format!("{:<width$}", placeholder, width = fw.saturating_sub(2));
                buffer.dibujar_texto(
                    fx + 1,
                    fy + 1,
                    &interior,
                    (40, 40, 40),
                    (150, 150, 150),
                );
            }
        }
    }

    // Dibujar hijos recursivamente
    for hijo in &nodo.hijos {
        dibujar_nodo(hijo, resultados, buffer, foco_id);
    }
}
