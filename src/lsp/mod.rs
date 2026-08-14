use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};
use std::sync::{Arc, Mutex};

use crate::lexer::Lexer;
use crate::parser::Parser;

#[derive(Clone)]
enum ValorJson {
    Nulo,
    Booleano(bool),
    Entero(i64),
    Cadena(String),
    Arreglo(Vec<ValorJson>),
    Objeto(HashMap<String, ValorJson>),
}

struct Lsp {
    salida: Arc<Mutex<io::Stdout>>,
    documentos: HashMap<String, String>,
}

impl Lsp {
    fn nuevo() -> Self {
        Lsp {
            salida: Arc::new(Mutex::new(io::stdout())),
            documentos: HashMap::new(),
        }
    }

    fn ejecutar(&mut self) {
        let mut lector = io::stdin().lock();
        let mut buf_cuerpo = Vec::with_capacity(65536);

        loop {
            let mut header = String::new();
            let mut longitud: Option<usize> = None;

            loop {
                header.clear();
                if lector.read_line(&mut header).unwrap_or(0) == 0 {
                    return;
                }
                let header = header.trim();
                if header.is_empty() {
                    break;
                }
                if let Some(resto) = header.strip_prefix("Content-Length: ") {
                    longitud = resto.trim().parse::<usize>().ok();
                }
            }

            let n = match longitud {
                Some(n) => n,
                None => continue,
            };

            buf_cuerpo.resize(n, 0u8);
            if lector.read_exact(&mut buf_cuerpo).is_err() {
                return;
            }

            let mensaje = String::from_utf8_lossy(&buf_cuerpo).to_string();
            self.procesar_mensaje(&mensaje);
        }
    }

    fn enviar(&self, json: &str) {
        let mut salida = self.salida.lock().unwrap();
        let _ = write!(salida, "Content-Length: {}\r\n\r\n{}", json.len(), json);
        let _ = salida.flush();
    }

    fn responder(&self, id: &ValorJson, resultado: HashMap<String, ValorJson>) {
        let mut resp = HashMap::new();
        resp.insert("jsonrpc".to_string(), ValorJson::Cadena("2.0".to_string()));
        resp.insert("id".to_string(), id.clone());
        resp.insert("result".to_string(), ValorJson::Objeto(resultado));
        self.enviar(&formatear_json(&ValorJson::Objeto(resp)));
    }

    fn responder_error(&self, id: &ValorJson, codigo: i64, mensaje: &str) {
        let mut err = HashMap::new();
        err.insert("code".to_string(), ValorJson::Entero(codigo));
        err.insert("message".to_string(), ValorJson::Cadena(mensaje.to_string()));
        let mut resp = HashMap::new();
        resp.insert("jsonrpc".to_string(), ValorJson::Cadena("2.0".to_string()));
        resp.insert("id".to_string(), id.clone());
        resp.insert("error".to_string(), ValorJson::Objeto(err));
        self.enviar(&formatear_json(&ValorJson::Objeto(resp)));
    }

    fn notificar(&self, metodo: &str, params: HashMap<String, ValorJson>) {
        let mut notif = HashMap::new();
        notif.insert("jsonrpc".to_string(), ValorJson::Cadena("2.0".to_string()));
        notif.insert("method".to_string(), ValorJson::Cadena(metodo.to_string()));
        notif.insert("params".to_string(), ValorJson::Objeto(params));
        self.enviar(&formatear_json(&ValorJson::Objeto(notif)));
    }

    fn diagnosticar(&self, uri: &str, texto: &str) {
        let mut diagnosticos = Vec::new();
        let lexer = Lexer::nuevo(texto);
        let mut parser = Parser::nuevo(lexer);
        let _programa = parser.parsear_programa();

        for err in &parser.errores {
            let mut diag = HashMap::new();
            let mut rango = HashMap::new();
            let mut inicio = HashMap::new();
            inicio.insert("line".to_string(), ValorJson::Entero(0));
            inicio.insert("character".to_string(), ValorJson::Entero(0));
            let mut fin = HashMap::new();
            fin.insert("line".to_string(), ValorJson::Entero(999));
            fin.insert("character".to_string(), ValorJson::Entero(999));
            rango.insert("start".to_string(), ValorJson::Objeto(inicio));
            rango.insert("end".to_string(), ValorJson::Objeto(fin));
            diag.insert("range".to_string(), ValorJson::Objeto(rango));
            diag.insert("severity".to_string(), ValorJson::Entero(1));
            diag.insert("message".to_string(), ValorJson::Cadena(err.clone()));
            diag.insert("source".to_string(), ValorJson::Cadena("argo".to_string()));
            diagnosticos.push(ValorJson::Objeto(diag));
        }

        let mut params = HashMap::new();
        params.insert("uri".to_string(), ValorJson::Cadena(uri.to_string()));
        params.insert("diagnostics".to_string(), ValorJson::Arreglo(diagnosticos));
        self.notificar("textDocument/publishDiagnostics", params);
    }

    fn procesar_mensaje(&mut self, json: &str) {
        let (valor, _) = parsear_json(json);
        let objeto = match valor {
            ValorJson::Objeto(o) => o,
            _ => return,
        };

        let metodo = match objeto.get("method").and_then(|v| match v {
            ValorJson::Cadena(s) => Some(s.clone()),
            _ => None,
        }) {
            Some(m) => m,
            None => return,
        };

        let id = objeto.get("id").cloned().unwrap_or(ValorJson::Nulo);

        let params = match objeto.get("params") {
            Some(ValorJson::Objeto(p)) => p.clone(),
            _ => HashMap::new(),
        };

        match metodo.as_str() {
            "initialize" => {
                let mut cap = HashMap::new();
                cap.insert(
                    "textDocumentSync".to_string(),
                    ValorJson::Entero(1),
                );
                cap.insert(
                    "completionProvider".to_string(),
                    ValorJson::Objeto({
                        let mut c = HashMap::new();
                        c.insert(
                            "triggerCharacters".to_string(),
                            ValorJson::Arreglo(vec![
                                ValorJson::Cadena(".".to_string()),
                            ]),
                        );
                        c
                    }),
                );
                cap.insert(
                    "hoverProvider".to_string(),
                    ValorJson::Booleano(true),
                );
                self.responder(&id, cap);
            }

            "initialized" => {}

            "textDocument/didOpen" => {
                let uri = match params.get("textDocument").and_then(|v| match v {
                    ValorJson::Objeto(o) => o.get("uri").and_then(|u| match u {
                        ValorJson::Cadena(s) => Some(s.clone()),
                        _ => None,
                    }),
                    _ => None,
                }) {
                    Some(u) => u,
                    None => return,
                };
                let texto = match params.get("textDocument").and_then(|v| match v {
                    ValorJson::Objeto(o) => o.get("text").and_then(|t| match t {
                        ValorJson::Cadena(s) => Some(s.clone()),
                        _ => None,
                    }),
                    _ => None,
                }) {
                    Some(t) => t,
                    None => return,
                };
                self.documentos.insert(uri.clone(), texto.clone());
                self.diagnosticar(&uri, &texto);
            }

            "textDocument/didChange" => {
                let uri = match params.get("textDocument").and_then(|v| match v {
                    ValorJson::Objeto(o) => o.get("uri").and_then(|u| match u {
                        ValorJson::Cadena(s) => Some(s.clone()),
                        _ => None,
                    }),
                    _ => None,
                }) {
                    Some(u) => u,
                    None => return,
                };
                let texto = match params.get("contentChanges").and_then(|v| match v {
                    ValorJson::Arreglo(a) => a.first().and_then(|c| match c {
                        ValorJson::Objeto(o) => o.get("text").and_then(|t| match t {
                            ValorJson::Cadena(s) => Some(s.clone()),
                            _ => None,
                        }),
                        _ => None,
                    }),
                    _ => None,
                }) {
                    Some(t) => t,
                    None => return,
                };
                self.documentos.insert(uri.clone(), texto.clone());
                self.diagnosticar(&uri, &texto);
            }

            "textDocument/completion" => {
                let mut items = vec![
                    crear_item("fn", "Palabra clave", "fn"),
                    crear_item("let", "Palabra clave", "let"),
                    crear_item("const", "Palabra clave", "const"),
                    crear_item("if", "Palabra clave", "if"),
                    crear_item("else", "Palabra clave", "else"),
                    crear_item("while", "Palabra clave", "while"),
                    crear_item("for", "Palabra clave", "for"),
                    crear_item("return", "Palabra clave", "return"),
                    crear_item("import", "Palabra clave", "import"),
                    crear_item("match", "Palabra clave", "match"),
                    crear_item("struct", "Palabra clave", "struct"),
                    crear_item("throw", "Palabra clave", "throw"),
                    crear_item("try", "Palabra clave", "try"),
                    crear_item("catch", "Palabra clave", "catch"),
                    crear_item("print", "Función nativa", "print("),
                    crear_item("len", "Función nativa", "len("),
                    crear_item("push", "Función nativa", "push("),
                    crear_item("tipo", "Función nativa", "tipo("),
                    crear_item("assert", "Función nativa", "assert("),
                    crear_item("canal", "Función nativa", "canal()"),
                    crear_item("enviar", "Función nativa", "enviar("),
                    crear_item("recibir", "Función nativa", "recibir("),
                    crear_item("true", "Literal", "true"),
                    crear_item("false", "Literal", "false"),
                    crear_item("null", "Literal", "null"),
                ];
                if let Some(ValorJson::Objeto(contexto)) = params.get("context")
                    && let Some(ValorJson::Cadena(trigger)) = contexto.get("triggerCharacter")
                    && trigger == "."
                {
                    items.push(crear_item(
                        "map",
                        "Método de arreglo",
                        "map(",
                    ));
                    items.push(crear_item(
                        "filter",
                        "Método de arreglo",
                        "filter(",
                    ));
                    items.push(crear_item(
                        "reduce",
                        "Método de arreglo",
                        "reduce(",
                    ));
                    items.push(crear_item(
                        "find",
                        "Método de arreglo",
                        "find(",
                    ));
                }
                let mut resultado = HashMap::new();
                resultado.insert(
                    "isIncomplete".to_string(),
                    ValorJson::Booleano(false),
                );
                resultado.insert(
                    "items".to_string(),
                    ValorJson::Arreglo(items),
                );
                self.responder(&id, resultado);
            }

            "textDocument/hover" => {
                let mut contenido = HashMap::new();
                contenido.insert(
                    "kind".to_string(),
                    ValorJson::Cadena("markdown".to_string()),
                );
                contenido.insert(
                    "value".to_string(),
                    ValorJson::Cadena(
                        "Argo v2.2.0 — Lenguaje de programación".to_string(),
                    ),
                );
                let mut resultado = HashMap::new();
                resultado.insert(
                    "contents".to_string(),
                    ValorJson::Objeto(contenido),
                );
                self.responder(&id, resultado);
            }

            "shutdown" => {
                self.responder(&id, HashMap::new());
            }

            "exit" => {
                std::process::exit(0);
            }

            _ => {
                self.responder_error(&id, -32601, &format!("Método no soportado: {}", metodo));
            }
        }
    }
}

fn crear_item(etiqueta: &str, detalle: &str, texto_insert: &str) -> ValorJson {
    let mut item = HashMap::new();
    item.insert(
        "label".to_string(),
        ValorJson::Cadena(etiqueta.to_string()),
    );
    item.insert(
        "detail".to_string(),
        ValorJson::Cadena(detalle.to_string()),
    );
    item.insert(
        "insertText".to_string(),
        ValorJson::Cadena(texto_insert.to_string()),
    );
    ValorJson::Objeto(item)
}

pub fn ejecutar_lsp() {
    let mut servidor = Lsp::nuevo();
    servidor.ejecutar();
}

fn parsear_json(entrada: &str) -> (ValorJson, usize) {
    let chars: Vec<char> = entrada.chars().collect();
    match saltar_espacios(&chars, 0) {
        Some(i) => parsear_valor(&chars, i),
        None => (ValorJson::Nulo, 0),
    }
}

fn saltar_espacios(chars: &[char], i: usize) -> Option<usize> {
    let mut j = i;
    while j < chars.len() && chars[j].is_ascii_whitespace() {
        j += 1;
    }
    if j < chars.len() { Some(j) } else { None }
}

fn parsear_valor(chars: &[char], i: usize) -> (ValorJson, usize) {
    let i = saltar_espacios(chars, i).unwrap_or(i);
    if i >= chars.len() {
        return (ValorJson::Nulo, i);
    }
    match chars[i] {
        '"' => parsear_cadena(chars, i),
        '{' => parsear_objeto(chars, i),
        '[' => parsear_arreglo(chars, i),
        't' | 'f' => parsear_booleano(chars, i),
        'n' => parsear_nulo(chars, i),
        '-' | '0'..='9' => parsear_numero(chars, i),
        _ => (ValorJson::Nulo, i),
    }
}

fn parsear_cadena(chars: &[char], i: usize) -> (ValorJson, usize) {
    let mut j = i + 1;
    let mut s = String::new();
    while j < chars.len() {
        if chars[j] == '"' {
            j += 1;
            break;
        }
        if chars[j] == '\\' && j + 1 < chars.len() {
            j += 1;
            match chars[j] {
                '"' => s.push('"'),
                '\\' => s.push('\\'),
                '/' => s.push('/'),
                'n' => s.push('\n'),
                'r' => s.push('\r'),
                't' => s.push('\t'),
                'u' => {
                    if j + 4 < chars.len() {
                        let hex: String = chars[j+1..j+5].iter().collect();
                        if let Ok(c) = u32::from_str_radix(&hex, 16)
                            && let Some(c) = char::from_u32(c)
                        {
                            s.push(c);
                        }
                        j += 4;
                    }
                }
                _ => s.push(chars[j]),
            }
        } else {
            s.push(chars[j]);
        }
        j += 1;
    }
    (ValorJson::Cadena(s), j)
}

fn parsear_numero(chars: &[char], i: usize) -> (ValorJson, usize) {
    let mut j = i;
    if chars[j] == '-' { j += 1; }
    while j < chars.len() && chars[j].is_ascii_digit() { j += 1; }
    if j < chars.len() && chars[j] == '.' {
        j += 1;
        while j < chars.len() && chars[j].is_ascii_digit() { j += 1; }
        let s: String = chars[i..j].iter().collect();
        (ValorJson::Entero(s.parse::<f64>().unwrap_or(0.0) as i64), j)
    } else {
        let s: String = chars[i..j].iter().collect();
        (ValorJson::Entero(s.parse::<i64>().unwrap_or(0)), j)
    }
}

fn parsear_booleano(chars: &[char], i: usize) -> (ValorJson, usize) {
    if chars[i..].starts_with(&['t', 'r', 'u', 'e']) {
        (ValorJson::Booleano(true), i + 4)
    } else if chars[i..].starts_with(&['f', 'a', 'l', 's', 'e']) {
        (ValorJson::Booleano(false), i + 5)
    } else {
        (ValorJson::Nulo, i)
    }
}

fn parsear_nulo(chars: &[char], i: usize) -> (ValorJson, usize) {
    if chars[i..].starts_with(&['n', 'u', 'l', 'l']) {
        (ValorJson::Nulo, i + 4)
    } else {
        (ValorJson::Nulo, i)
    }
}

fn parsear_arreglo(chars: &[char], i: usize) -> (ValorJson, usize) {
    let mut j = i + 1;
    let mut arr = Vec::new();
    while let Some(p) = saltar_espacios(chars, j) {
        j = p;
        if j >= chars.len() || chars[j] == ']' {
            j += 1;
            break;
        }
        let (val, k) = parsear_valor(chars, j);
        arr.push(val);
        j = k;
        if let Some(p) = saltar_espacios(chars, j) {
            j = p;
        } else {
            break;
        }
        if j < chars.len() && chars[j] == ',' {
            j += 1;
        }
    }
    (ValorJson::Arreglo(arr), j)
}

fn parsear_objeto(chars: &[char], i: usize) -> (ValorJson, usize) {
    let mut j = i + 1;
    let mut obj = HashMap::new();
    while let Some(p) = saltar_espacios(chars, j) {
        j = p;
        if j >= chars.len() || chars[j] == '}' {
            j += 1;
            break;
        }
        let (clave, k) = parsear_cadena(chars, j);
        j = k;
        if let Some(p) = saltar_espacios(chars, j) {
            j = p;
        } else {
            break;
        }
        if j < chars.len() && chars[j] == ':' {
            j += 1;
        }
        let (valor, k) = parsear_valor(chars, j);
        if let ValorJson::Cadena(c) = clave {
            obj.insert(c, valor);
        }
        j = k;
        if let Some(p) = saltar_espacios(chars, j) {
            j = p;
        } else {
            break;
        }
        if j < chars.len() && chars[j] == ',' {
            j += 1;
        }
    }
    (ValorJson::Objeto(obj), j)
}

fn formatear_json(valor: &ValorJson) -> String {
    match valor {
        ValorJson::Nulo => "null".to_string(),
        ValorJson::Booleano(b) => b.to_string(),
        ValorJson::Entero(n) => n.to_string(),
        ValorJson::Cadena(s) => {
            let mut r = String::with_capacity(s.len() + 2);
            r.push('"');
            for c in s.chars() {
                match c {
                    '"' => r.push_str("\\\""),
                    '\\' => r.push_str("\\\\"),
                    '\n' => r.push_str("\\n"),
                    '\r' => r.push_str("\\r"),
                    '\t' => r.push_str("\\t"),
                    _ => r.push(c),
                }
            }
            r.push('"');
            r
        }
        ValorJson::Arreglo(a) => {
            let mut r = String::from("[");
            for (i, v) in a.iter().enumerate() {
                if i > 0 { r.push_str(", "); }
                r.push_str(&formatear_json(v));
            }
            r.push(']');
            r
        }
        ValorJson::Objeto(o) => {
            let mut r = String::from("{");
            for (i, (k, v)) in o.iter().enumerate() {
                if i > 0 { r.push_str(", "); }
                r.push_str(&formatear_json(&ValorJson::Cadena(k.clone())));
                r.push_str(": ");
                r.push_str(&formatear_json(v));
            }
            r.push('}');
            r
        }
    }
}
