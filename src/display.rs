use std::fmt::{Display, Formatter};

use crate::{
    expr::{Cast, Expr, Literal, AST},
    types::{DataType, Type},
    values::Value,
};

impl Display for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AST::Literal(literal) => write!(f, "{literal}"),
            AST::ColumnRef { name, data_type } => write!(f, "{name}::{data_type}"),
            AST::FunctionCall { name, args, params } => {
                write!(f, "{name}")?;
                if !params.is_empty() {
                    write!(f, "(")?;
                    for (i, param) in params.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{param}")?;
                    }
                    write!(f, ")")?;
                }
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
        }
    }
}

impl<T: Display> Display for Literal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Null => write!(f, "NULL"),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::UInt8(u) => write!(f, "{}", u),
            Literal::UInt16(u) => write!(f, "{}", u),
            Literal::Int8(i) => write!(f, "{}", i),
            Literal::Int16(i) => write!(f, "{}", i),
            Literal::String(s) => write!(f, "{}", s),
            Literal::Array(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &self {
            DataType::Any => write!(f, "Any"),
            DataType::Hole => write!(f, "_"),
            DataType::Nullable(inner) => write!(f, "Nullable<{inner}>"),
            DataType::Array(inner) => write!(f, "Array<{inner}>"),
            DataType::Boolean => write!(f, "Boolean"),
            DataType::String => write!(f, "String"),
            DataType::UInt8 => write!(f, "UInt8"),
            DataType::UInt16 => write!(f, "UInt16"),
            DataType::Int8 => write!(f, "Int8"),
            DataType::Int16 => write!(f, "Int16"),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(literal) => write!(f, "{literal}"),
            Expr::ColumnRef { name, .. } => write!(f, "{name}"),
            Expr::FunctionCall { function, args } => {
                write!(f, "{}<", function.signature.name)?;
                for (i, ty) in function.signature.args_type.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                write!(f, ">(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            Expr::Cast { expr, casts } => {
                write!(f, "cast<")?;
                for (i, cast) in casts.iter().enumerate() {
                    if i > 0 {
                        write!(f, " -> ")?;
                    }
                    write!(f, "{cast}")?;
                }
                write!(f, ">({expr})", expr = expr)
            }
        }
    }
}

impl Display for Cast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cast::ToNullable => write!(f, "ToNullable"),
            Cast::Int8ToInt16 => write!(f, "Int8ToInt16"),
            Cast::UInt8ToUInt16 => write!(f, "UInt8ToUInt16"),
            Cast::UInt8ToInt16 => write!(f, "UInt8ToInt16"),
            Cast::MapNullable(casts) => {
                write!(f, "MapNullable<")?;
                for (i, cast) in casts.iter().enumerate() {
                    if i > 0 {
                        write!(f, " -> ")?;
                    }
                    write!(f, "{cast}", cast = cast)?;
                }
                write!(f, ">")
            }
        }
    }
}

impl<T: Type> Display for Value<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::Scalar(scalar) => write!(f, "{:?}", scalar),
            Value::Column(col) => write!(f, "{:?}", col),
        }
    }
}
