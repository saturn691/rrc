use core::fmt;

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveType {
    Void,
    I1,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveType::Void => write!(f, "void"),
            PrimitiveType::I1 => write!(f, "i1"),
            PrimitiveType::I8 => write!(f, "i8"),
            PrimitiveType::I16 => write!(f, "i16"),
            PrimitiveType::I32 => write!(f, "i32"),
            PrimitiveType::I64 => write!(f, "i64"),
            PrimitiveType::U8 => write!(f, "u8"),
            PrimitiveType::U16 => write!(f, "u16"),
            PrimitiveType::U32 => write!(f, "u32"),
            PrimitiveType::U64 => write!(f, "u64"),
            PrimitiveType::F32 => write!(f, "f32"),
            PrimitiveType::F64 => write!(f, "f64")
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Type {
    Primitive(PrimitiveType)
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Type::Primitive(PrimitiveType::Void) => 0,
            Type::Primitive(PrimitiveType::I1) => 1,
            Type::Primitive(PrimitiveType::I8) => 1,
            Type::Primitive(PrimitiveType::I16) => 2,
            Type::Primitive(PrimitiveType::I32) => 4,
            Type::Primitive(PrimitiveType::I64) => 8,
            Type::Primitive(PrimitiveType::U8) => 1,
            Type::Primitive(PrimitiveType::U16) => 2,
            Type::Primitive(PrimitiveType::U32) => 4,
            Type::Primitive(PrimitiveType::U64) => 8,
            Type::Primitive(PrimitiveType::F32) => 4,
            Type::Primitive(PrimitiveType::F64) => 8
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(ty) => write!(f, "{}", ty)
        }
    }
}