#![allow(dead_code)]

//
// Serializador binario .argbc para Argo.
// Convierte nodos del AST a/desde bytes (formato little-endian).
// Cada valor se precede de un OpCode de 1 byte.
//
// OpCodes definidos:
//   0x01  OP_INT        → 8 bytes i64 LE
//   0x02  OP_STR        → 8 bytes longitud LE + N bytes UTF-8
//   0x03  OP_BOOL       → 1 byte (1 = true, 0 = false)
//   0x04  OP_IDENTIFIER → 8 bytes longitud LE + N bytes UTF-8
//   0x05  OP_ARRAY      → 4 bytes count u32 LE + N expresiones
//   0x06  OP_DICT       → 4 bytes count u32 LE + N pares (clave + valor)
//   0x07  OP_LET        → identificador + expresion
//

use crate::ast::Expression;
use crate::ast::Statement;

const OP_INT: u8 = 0x01;
const OP_STR: u8 = 0x02;
const OP_BOOL: u8 = 0x03;
const OP_IDENTIFIER: u8 = 0x04;
const OP_ARRAY: u8 = 0x05;
const OP_DICT: u8 = 0x06;
const OP_LET: u8 = 0x07;

// ---------------------------------------------------------------------------
// string_a_bytes / bytes_a_string — helpers para cadenas con prefijo de
// longitud (8 bytes LE). Reutilizados por OP_STR y OP_IDENTIFIER.
// ---------------------------------------------------------------------------
fn string_a_bytes(bytes: &mut Vec<u8>, s: &str) {
    let len = s.len();
    bytes.extend_from_slice(&(len as u64).to_le_bytes());
    bytes.extend_from_slice(s.as_bytes());
}

fn bytes_a_string(buf: &[u8], cursor: &mut usize) -> Result<String, String> {
    if *cursor + 8 > buf.len() {
        return Err(format!(
            "bytecode: se esperaban 8 bytes para longitud de cadena en cursor {}",
            *cursor
        ));
    }
    let len_arr: [u8; 8] = buf[*cursor..*cursor + 8]
        .try_into()
        .map_err(|_| "bytecode: error al leer longitud de cadena".to_string())?;
    *cursor += 8;

    let str_len = u64::from_le_bytes(len_arr) as usize;

    if *cursor + str_len > buf.len() {
        return Err(format!(
            "bytecode: cadena de {} bytes excede el buffer en cursor {}",
            str_len, *cursor
        ));
    }

    let slice = &buf[*cursor..*cursor + str_len];
    *cursor += str_len;

    String::from_utf8(slice.to_vec())
        .map_err(|e| format!("bytecode: UTF-8 inválido en cadena: {}", e))
}

// ---------------------------------------------------------------------------
// ast_a_bytes — Serializa una Expression a binario
// ---------------------------------------------------------------------------
pub fn ast_a_bytes(expr: &Expression) -> Vec<u8> {
    match expr {
        Expression::Entero(n) => {
            let mut bytes = Vec::with_capacity(9);
            bytes.push(OP_INT);
            bytes.extend_from_slice(&n.to_le_bytes());
            bytes
        }
        Expression::Booleano(b) => {
            vec![OP_BOOL, if *b { 1 } else { 0 }]
        }
        Expression::Cadena(s) => {
            let mut bytes = Vec::with_capacity(9 + s.len());
            bytes.push(OP_STR);
            string_a_bytes(&mut bytes, s);
            bytes
        }
        Expression::Identificador(s) => {
            let mut bytes = Vec::with_capacity(9 + s.len());
            bytes.push(OP_IDENTIFIER);
            string_a_bytes(&mut bytes, s);
            bytes
        }
        Expression::Arreglo(elementos) => {
            let mut bytes = vec![OP_ARRAY];
            bytes.extend_from_slice(&(elementos.len() as u32).to_le_bytes());
            for elem in elementos {
                bytes.extend_from_slice(&ast_a_bytes(elem));
            }
            bytes
        }
        Expression::Diccionario(pares) => {
            let mut bytes = vec![OP_DICT];
            bytes.extend_from_slice(&(pares.len() as u32).to_le_bytes());
            for (clave, valor) in pares {
                bytes.extend_from_slice(&ast_a_bytes(clave));
                bytes.extend_from_slice(&ast_a_bytes(valor));
            }
            bytes
        }
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// sentencia_a_bytes — Serializa una Statement a binario
// ---------------------------------------------------------------------------
pub fn sentencia_a_bytes(stmt: &Statement) -> Vec<u8> {
    match stmt {
        Statement::DeclaracionVariable { nombre, valor } => {
            let valor_bytes = ast_a_bytes(valor);
            let mut bytes = Vec::with_capacity(10 + nombre.len() + valor_bytes.len());
            bytes.push(OP_LET);
            string_a_bytes(&mut bytes, nombre);
            bytes.extend_from_slice(&valor_bytes);
            bytes
        }
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// bytes_a_ast — Deserializa una Expression desde un slice de bytes
// ---------------------------------------------------------------------------
pub fn bytes_a_ast(buf: &[u8], cursor: &mut usize) -> Result<Expression, String> {
    if *cursor >= buf.len() {
        return Err(format!(
            "bytecode: buffer insuficiente en cursor {} (longitud {})",
            cursor,
            buf.len()
        ));
    }

    let opcode = buf[*cursor];
    *cursor += 1;

    match opcode {
        OP_INT => {
            if *cursor + 8 > buf.len() {
                return Err(format!(
                    "bytecode: se esperaban 8 bytes para entero en cursor {}",
                    *cursor
                ));
            }
            let arr: [u8; 8] = buf[*cursor..*cursor + 8]
                .try_into()
                .map_err(|_| "bytecode: error al leer entero".to_string())?;
            *cursor += 8;
            Ok(Expression::Entero(i64::from_le_bytes(arr)))
        }

        OP_BOOL => {
            if *cursor >= buf.len() {
                return Err(format!(
                    "bytecode: se esperaba 1 byte para booleano en cursor {}",
                    *cursor
                ));
            }
            let valor = buf[*cursor];
            *cursor += 1;
            match valor {
                0 => Ok(Expression::Booleano(false)),
                1 => Ok(Expression::Booleano(true)),
                _ => Err(format!(
                    "bytecode: valor booleano inválido {} en cursor {}",
                    valor,
                    *cursor - 1
                )),
            }
        }

        OP_STR => {
            let s = bytes_a_string(buf, cursor)?;
            Ok(Expression::Cadena(s))
        }

        OP_IDENTIFIER => {
            let s = bytes_a_string(buf, cursor)?;
            Ok(Expression::Identificador(s))
        }

        OP_ARRAY => {
            if *cursor + 4 > buf.len() {
                return Err(format!(
                    "bytecode: se esperaban 4 bytes para longitud de arreglo en cursor {}",
                    *cursor
                ));
            }
            let count_arr: [u8; 4] = buf[*cursor..*cursor + 4]
                .try_into()
                .map_err(|_| "bytecode: error al leer longitud de arreglo".to_string())?;
            *cursor += 4;

            let count = u32::from_le_bytes(count_arr) as usize;
            let mut elementos = Vec::with_capacity(count);

            for _ in 0..count {
                elementos.push(bytes_a_ast(buf, cursor)?);
            }

            Ok(Expression::Arreglo(elementos))
        }

        OP_DICT => {
            if *cursor + 4 > buf.len() {
                return Err(format!(
                    "bytecode: se esperaban 4 bytes para longitud de diccionario en cursor {}",
                    *cursor
                ));
            }
            let count_arr: [u8; 4] = buf[*cursor..*cursor + 4]
                .try_into()
                .map_err(|_| "bytecode: error al leer longitud de diccionario".to_string())?;
            *cursor += 4;

            let count = u32::from_le_bytes(count_arr) as usize;
            let mut pares = Vec::with_capacity(count);

            for _ in 0..count {
                let clave = bytes_a_ast(buf, cursor)?;
                let valor = bytes_a_ast(buf, cursor)?;
                pares.push((clave, valor));
            }

            Ok(Expression::Diccionario(pares))
        }

        _ => Err(format!(
            "bytecode: opcode desconocido 0x{:02x} en cursor {}",
            opcode,
            *cursor - 1
        )),
    }
}

// ---------------------------------------------------------------------------
// bytes_a_sentencia — Deserializa una Statement desde un slice de bytes
// ---------------------------------------------------------------------------
pub fn bytes_a_sentencia(buf: &[u8], cursor: &mut usize) -> Result<Statement, String> {
    if *cursor >= buf.len() {
        return Err(format!(
            "bytecode: buffer insuficiente en cursor {} (longitud {})",
            cursor,
            buf.len()
        ));
    }

    let opcode = buf[*cursor];
    *cursor += 1;

    match opcode {
        OP_LET => {
            let nombre = bytes_a_string(buf, cursor)?;
            let valor = bytes_a_ast(buf, cursor)?;
            Ok(Statement::DeclaracionVariable { nombre, valor })
        }

        _ => Err(format!(
            "bytecode: opcode de sentencia desconocido 0x{:02x} en cursor {}",
            opcode,
            *cursor - 1
        )),
    }
}
