//
// Memoria local (Entorno) con alcance anidado léxico. Un Entorno es un
// mapa de variables (nombre → Objeto) más un puntero opcional al entorno
// padre. Esta estructura permite el scoping de bloques (if, while, funciones)
// y es la base para implementar closures en el futuro: una función captura
// el Entorno en el que fue definida a través del puntero `externo`.
//
// Gestión de memoria involucrada:
//   - HashMap: almacena claves (String) → valores (Objeto) en el heap.
//   - Box<Entorno>: el entorno padre se coloca en el heap para que la
//     estructura Entorno tenga tamaño fijo (evita recursión infinita).
//   - Option<Box<Entorno>>: permite que un Entorno global no tenga padre.

use std::collections::{HashMap, HashSet};

use crate::evaluator::object::Objeto;

// ---------------------------------------------------------------------------
// Entorno — Frame de ejecución con acceso al ámbito padre
// ---------------------------------------------------------------------------
// Cada vez que el intérprete entra en un bloque ({}) o invoca una función,
// se crea un nuevo Entorno. Las variables declaradas dentro del bloque se
// almacenan en el HashMap `almacen`. Si una variable no se encuentra en el
// almacén actual, la búsqueda delega en el entorno `externo` (ámbito padre).
//
// Tamaño en stack (64 bits):
//   - HashMap<String, Objeto>: 56 bytes (8 ptr + 8 len + 8 cap + etc.)
//   - Option<Box<Entorno>>: 16 bytes (8 discriminante + 8 ptr Box)
//   - Total: ~72 bytes en el stack, más el heap para el HashMap y el Box.
// #[derive(Debug, Clone)] funciona aquí aunque Entorno contiene
// Box<Entorno> (recursión). Debug itera la cadena y termina en None.
// Clone llama a Box::clone → Entorno::clone recursivamente, pero
// la cadena termina cuando externo == None (entorno raíz). La
// profundidad máxima es el anidamiento del programa, siempre finito.
#[derive(Debug, Clone)]
pub struct Entorno {
    // Almacén de variables. HashMap posee los Strings y Objetos que
    // contiene. Cuando el Entorno se dropea, el Drop de HashMap libera
    // todas las claves (String → libera heap) y valores (Objeto →
    // libera heap si son Cadena, Error, Retorno, etc.). No hay fuga
    // de memoria porque el ownership jerárquico garantiza la limpieza.
    pub almacen: HashMap<String, Objeto>,

    // Conjunto de nombres de variables declaradas como `const`.
    // Cuando `actualizar` intenta reasignar una variable cuyo nombre
    // está en este conjunto, retorna un error de reasignación de constante.
    pub constantes: HashSet<String>,

    // Entorno padre opcional. None en el entorno global (raíz).
    // Some(Box<Entorno>) cuando este entorno es un ámbito anidado.
    // Box<Entorno> coloca el Entorno padre en el heap. Esto es
    // necesario porque Entorno contiene un HashMap (56 bytes) y otro
    // Option<Box<Entorno>>, lo que hace que su tamaño no sea fijo si
    // se anidara recursivamente sin Box. Box rompe la recursión:
    // el Entorno solo almacena un puntero de 8 bytes al padre.
    //
    // Option<Box<Entorno>> es el "puntero al entorno padre" que
    // permite recorrer la cadena de ámbitos hacia arriba durante
    // la resolución de variables. La Option maneja el caso base
    // (entorno raíz) sin usar nullptr ni valores centinela.
    pub externo: Option<Box<Entorno>>,
}

impl Entorno {
    // -----------------------------------------------------------------------
    // Constructor del entorno global (raíz)
    // -----------------------------------------------------------------------
    // Crea un Entorno sin padre, usado como punto de partida para la
    // ejecución de un programa. Solo existe un entorno global por
    // ejecución. `externo` es None, indicando que no hay ámbito superior.
    //
    // Retorno: Self (Entorno por valor). La ownership se transfiere al
    // llamante. El HashMap se inicializa vacío (sin heap allocation
    // significativa; HashMap::new() asigna 0 buckets). Crece bajo demanda.
    pub fn nuevo() -> Self {
        // HashMap::new() crea un mapa vacío. No aloca en heap hasta la
        // primera inserción. Esto es importante para programas pequeños:
        // no hay costo fijo por crear entornos.
        //
        // `externo: None` asigna el discriminante None al Option.
        // No hay heap allocation aquí porque None no contiene datos.
        Entorno {
            almacen: HashMap::new(),
            constantes: HashSet::new(),
            externo: None,
        }
    }

    pub fn nuevo_local(externo: Entorno) -> Self {
        Entorno {
            almacen: HashMap::new(),
            constantes: HashSet::new(),
            externo: Some(Box::new(externo)),
        }
    }

    // -----------------------------------------------------------------------
    // Búsqueda de variable con resolución de ámbito
    // -----------------------------------------------------------------------
    // Busca una variable por nombre en el almacén actual. Si no existe,
    // delega la búsqueda al entorno padre, y así recursivamente hasta
    // llegar al entorno global (donde `externo` es None).
    //
    // Parámetros:
    //   `&self` — préstamo inmutable del Entorno. No modificamos el
    //             almacén durante la búsqueda.
    //   `nombre: &str` — préstamo (&) del nombre de la variable.
    //                    No tomamos ownership del String; solo lo
    //                    prestamos para comparar claves en el HashMap.
    //
    // Retorno:
    //   `Option<Objeto>` — Some(clon) si la variable existe, None si no.
    //   Se retorna una copia (clone) del Objeto para que el llamante
    //   tenga ownership independiente. Esto es necesario porque el
    //   Objeto original debe permanecer en el HashMap del Entorno
    //   (no podemos moverlo fuera — estamos con &self, préstamo
    //   inmutable, así que no podemos tomar ownership).
    //
    // # Gestión de memoria (clonación)
    //   Clone sobre Objeto puede ser costoso:
    //     - Entero, Flotante, Booleano, Nulo: clonación trivial (Copy).
    //     - Cadena, Error: allocación en heap (copia O(n) del String).
    //     - Retorno: clonación recursiva del Objeto interno (heap).
    //   En un futuro, podríamos optimizar con Rc<Objeto> o Cow para
    //   compartir valores inmutables sin copiar. Por ahora, clone()
    //   es suficiente y seguro: ningún valor se escapa del préstamo.
    //
    // # Borrow checker
    //   El HashMap::get(&self, key) devuelve Option<&Objeto> (referencia
    //   prestada al valor dentro del HashMap). No podemos retornar esa
    //   referencia directamente porque el Entorno podría dropearse y
    //   la referencia quedaría colgando. Clone() produce un valor
    //   con ownership propio (independiente del HashMap), eliminando
    //   la dependencia del lifetime del Entorno.
    pub fn obtener(&self, nombre: &str) -> Option<Objeto> {
        // Buscar en el almacén local. `self.almacen.get(nombre)` devuelve
        // `Option<&Objeto>`. El `?` propaga None (variable no encontrada).
        // Si encontramos el valor, `.clone()` produce una copia con
        // ownership propio que retornamos dentro de Some(...).
        //
        // Alternativamente, si no está en el almacén local, miramos el
        // padre: `self.externo.as_ref()` convierte `&Option<Box<Entorno>>`
        // en `Option<&Box<Entorno>>`. Luego `.and_then(...)` aplica la
        // función si Some, o propaga None si el padre no existe.
        //
        // `as_ref()` es necesario porque &Option<T> y Option<&T> son
        // tipos distintos. as_ref() transforma Option<Box<Entorno>> en
        // Option<&Box<Entorno>> sin mover el Box.
        self.almacen
            .get(nombre)
            .cloned()
            .or_else(|| {
                // Si no encuentra en local, busca recursivamente en el
                // padre. `self.externo` es &Option<Box<Entorno>>.
                // `.as_ref()` → Option<&Box<Entorno>>.
                // `.and_then(|padre| padre.obtener(nombre))` delega la
                // búsqueda al entorno padre. La recursión termina cuando
                // `externo` es None (entorno raíz).
                //
                // `padre` es &Box<Entorno>. Rust aplica deref coercion:
                // `padre.obtener(nombre)` trata el Box como &Entorno
                // automáticamente gracias a Deref. La llamada a obtener
                // en el padre también es con &self, así que no hay
                // conflicto de mutabilidad.
                self.externo
                    .as_ref()
                    .and_then(|padre| padre.obtener(nombre))
            })
    }

    // -----------------------------------------------------------------------
    // Asignación de variable (inserción o actualización)
    // -----------------------------------------------------------------------
    // Inserta una nueva variable en el almacén actual o actualiza una
    // existente. Esta operación siempre actúa sobre el almacén local;
    // no busca en el padre para reasignar (a diferencia de lenguajes
    // como Python, Argo asigna en el ámbito más interno). Si se desea
    // modificar una variable del padre desde un ámbito hijo, se requiere
    // un mecanismo explícito (como `mut` o reasignación desde afuera).
    //
    // Parámetros:
    //   `&mut self` — préstamo mutable del Entorno. Necesario para
    //                 modificar el HashMap (insertar/actualizar entradas).
    //   `nombre: String` — nombre de la variable, recibido por ownership.
    //                      El String se mueve al HashMap como clave.
    //   `valor: Objeto` — valor a asignar, recibido por ownership.
    //                     El Objeto se mueve al HashMap como valor.
    //
    // # Gestión de memoria
    // La ownership de `nombre` y `valor` se transfiere al HashMap.
    // El llamante ya no puede usar estas variables después de la
    // llamada (el borrow checker lo impide). Si la clave ya existía,
    // el valor antiguo se dropea (drop) y su heap se libera.
    // HashMap::insert retorna Option<Objeto> con el valor anterior;
    // como hacemos `let _ = ...`, descartamos ese valor, ejecutando
    // su Drop. En un futuro podríamos retornarlo para decisiones de
    // shadowing.
    //
    // No hay panic! posible aquí: HashMap::insert no falla a menos
    // que haya falta de memoria del sistema (OutOfMemory), en cuyo
    // caso Rust aborta el proceso (no es un panic! recuperable).
    pub fn asignar(&mut self, nombre: String, valor: Objeto) {
        let _ = self.almacen.insert(nombre, valor);
    }

    pub fn declarar(&mut self, nombre: String, valor: Objeto, es_constante: bool) {
        if es_constante {
            self.constantes.insert(nombre.clone());
        }
        let _ = self.almacen.insert(nombre, valor);
    }

    // -----------------------------------------------------------------------
    // actualizar — Reasignación de variable con resolución de ámbito
    // -----------------------------------------------------------------------
    // Busca una variable por nombre en la cadena de ámbitos y actualiza su
    // valor. A diferencia de `asignar` (que siempre opera en el almacén
    // local), `actualizar` recorre la jerarquía de padres hasta encontrar
    // la variable o llegar al entorno raíz. Esto permite reasignar variables
    // definidas en ámbitos superiores (como un contador de bucle en el
    // ámbito global o de una función contenedora).
    //
    // Parámetros:
    //   `&mut self` — préstamo mutable del Entorno. Necesario para
    //                 modificar el HashMap (reescribir el valor).
    //   `nombre: &str` — préstamo (&) del nombre de la variable.
    //   `valor: Objeto` — nuevo valor, recibido por ownership.
    //                     El Objeto se mueve al HashMap.
    //
    // Retorno:
    //   `Ok(())` si la variable existe y se actualizó.
    //   `Err(String)` con mensaje de error si la variable no existe
    //   en ningún ámbito de la cadena.
    //
    // # Borrow checker y recursión mutable
    //   El desafío aquí es que necesitamos &mut self para actualizar el
    //   HashMap, pero también necesitamos acceder a &mut self.externo
    //   para recorrer la cadena de padres. Esto NO es un problema porque
    //   el match es exhaustivo: si la clave existe localmente, no tocamos
    //   el padre. Si no existe localmente, accedemos a self.externo con
    //   &mut, pero ya no usamos self.almacen. El borrow checker acepta
    //   esto porque los prestamos son disjuntos (no hay superposición
    //   entre almacen y externo).
    //
    //   La recursión se implementa con un bucle `loop` en lugar de
    //   llamada recursiva para evitar el problema de "double borrow":
    //   si llamáramos `padre.actualizar(...)` dentro de un match sobre
    //   `&mut self`, el compilador no podría probar que el prestamo
    //   mutable de self ha terminado. El bucle itera sobre referencias
    //   &mut a cada Entorno de la cadena, moviéndose al padre cuando
    //   la variable no se encuentra en el nivel actual. Esto elimina
    //   la necesidad de recursión y mantiene el código prestable.
    pub fn actualizar(&mut self, nombre: &str, valor: Objeto) -> Result<(), String> {
        // Comenzar desde el entorno actual.
        let mut entorno_actual = Some(self);

        // Recorrer la cadena de ámbitos usando un bucle. En cada iteración,
        // `entorno_actual` es &mut Entorno del nivel actual.
        while let Some(entorno) = entorno_actual {
            if let Some(entry) = entorno.almacen.get_mut(nombre) {
                if entorno.constantes.contains(nombre) {
                    return Err(format!(
                        "No se puede reasignar una constante: {}", nombre
                    ));
                }
                *entry = valor;
                return Ok(());
            }
            entorno_actual = entorno
                .externo
                .as_mut()
                .map(|box_padre| box_padre.as_mut());
        }

        // La variable no se encontró en ningún ámbito.
        Err(format!("Variable no definida: {}", nombre))
    }

    // -----------------------------------------------------------------------
    // exportar — Extrae las variables del entorno para construir un módulo
    // -----------------------------------------------------------------------
    // Consume el Entorno por ownership (self) y retorna su HashMap interno.
    // Útil en la evaluación de `import`: después de ejecutar un módulo en
    // un entorno aislado, se extrae su almacén para construir el diccionario
    // que se retorna al llamante. Solo se exportan las variables del ámbito
    // raíz (las declaradas con `let` a nivel de módulo); las variables
    // locales de funciones y bloques anidados no se exportan.
    //
    // # Gestión de memoria
    // La ownership del HashMap se transfiere al llamante. El Entorno se
    // consume (self, no &self). Después de esta llamada, el Entorno ya no
    // existe y su memoria (HashMap, Box<Entorno> padre) se libera.
    pub fn exportar(self) -> HashMap<String, Objeto> {
        self.almacen
    }
}
