use std::time::Instant;

// ---------------------------------------------------------------------------
// Animacion — Una animación individual con interpolación lineal
// ---------------------------------------------------------------------------
pub struct Animacion {
    /// Identificador del nodo afectado
    pub nodo_id: usize,
    /// Propiedad a animar: "valor", "x", "y", "fg_r", "fg_g", "fg_b", etc.
    pub propiedad: String,
    /// Valor inicial
    pub inicio: f32,
    /// Valor final
    pub fin: f32,
    /// Duración en milisegundos
    pub duracion_ms: u64,
    /// Tick de inicio (en ms desde referencia)
    pub inicio_tick: u64,
    /// Si la animación está activa
    pub activa: bool,
}

// ---------------------------------------------------------------------------
// MotorAnimaciones — Motor de animaciones con Lerp
// ---------------------------------------------------------------------------
pub struct MotorAnimaciones {
    pub animaciones: Vec<Animacion>,
    pub referencia: Instant,
}

impl Default for MotorAnimaciones {
    fn default() -> Self {
        Self::new()
    }
}

impl MotorAnimaciones {
    pub fn new() -> Self {
        MotorAnimaciones {
            animaciones: Vec::new(),
            referencia: Instant::now(),
        }
    }

    /// Obtiene el tick actual en milisegundos desde la referencia.
    #[inline]
    pub fn tick_actual(&self) -> u64 {
        self.referencia.elapsed().as_millis() as u64
    }

    /// Lanza una nueva animación.
    pub fn lanzar(
        &mut self,
        nodo_id: usize,
        propiedad: &str,
        inicio: f32,
        fin: f32,
        duracion_ms: u64,
    ) {
        // Si ya hay una animación para esta propiedad de este nodo, reemplazar
        self.animaciones.retain(|a| !(a.nodo_id == nodo_id && a.propiedad == propiedad));

        self.animaciones.push(Animacion {
            nodo_id,
            propiedad: propiedad.to_string(),
            inicio,
            fin,
            duracion_ms,
            inicio_tick: self.tick_actual(),
            activa: true,
        });
    }

    /// Calcula el frame actual: retorna lista de (nodo_id, propiedad, valor).
    pub fn calcular_frame(&mut self) -> Vec<(usize, String, f32)> {
        let tick = self.tick_actual();
        let mut resultados = Vec::new();

        for anim in &mut self.animaciones {
            if !anim.activa {
                continue;
            }

            let elapsed = tick.saturating_sub(anim.inicio_tick);

            if anim.duracion_ms == 0 || elapsed >= anim.duracion_ms {
                // Animación terminada: valor final
                resultados.push((anim.nodo_id, anim.propiedad.clone(), anim.fin));
                anim.activa = false;
                continue;
            }

            // Progreso t en [0.0, 1.0]
            let t = elapsed as f32 / anim.duracion_ms as f32;
            // Lerp: inicio + (fin - inicio) * t
            let valor = anim.inicio + (anim.fin - anim.inicio) * t;
            resultados.push((anim.nodo_id, anim.propiedad.clone(), valor));
        }

        // Limpiar animaciones inactivas
        self.animaciones.retain(|a| a.activa);

        resultados
    }

    /// Retorna true si hay animaciones activas.
    pub fn tiene_animaciones_activas(&self) -> bool {
        self.animaciones.iter().any(|a| a.activa)
    }
}

// ---------------------------------------------------------------------------
// Lerp — Funciones de interpolación lineal
// ---------------------------------------------------------------------------

/// Interpolación lineal entre dos valores f32.
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Interpolación lineal entre dos colores RGB.
#[inline]
pub fn lerp_color(
    a: (u8, u8, u8),
    b: (u8, u8, u8),
    t: f32,
) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let r = a.0 as f32 + (b.0 as f32 - a.0 as f32) * t;
    let g = a.1 as f32 + (b.1 as f32 - a.1 as f32) * t;
    let bl = a.2 as f32 + (b.2 as f32 - a.2 as f32) * t;
    (r as u8, g as u8, bl as u8)
}

/// Ease-in-out cuadrático para animaciones más suaves.
#[inline]
pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}
