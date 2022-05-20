use crate::{
    expr::{Cast, Expr, Literal, AST},
    function::FunctionRegistry,
    types::DataType,
};

pub fn check(ast: &AST, fn_registry: &FunctionRegistry) -> Option<(Expr, DataType)> {
    match ast {
        AST::Literal(lit) => {
            let (lit, ty) = check_literal(lit);
            Some((Expr::Literal(lit), ty))
        }
        AST::ColumnRef { name, data_type } => {
            Some((Expr::ColumnRef { name: name.clone() }, data_type.clone()))
        }
        AST::FunctionCall { name, args } => {
            let args = args
                .iter()
                .map(|arg| check(arg, fn_registry))
                .collect::<Option<Vec<_>>>()?;

            try_search_function(name, &args, fn_registry)
        }
    }
}

pub fn check_literal(literal: &Literal<AST>) -> (Literal<Expr>, DataType) {
    match literal {
        Literal::Null => (Literal::Null, DataType::Nullable(Box::new(DataType::Hole))),
        Literal::Int8(val) => (Literal::Int8(*val), DataType::Int8),
        Literal::Int16(val) => (Literal::Int16(*val), DataType::Int16),
        Literal::UInt8(val) => (Literal::UInt8(*val), DataType::UInt8),
        Literal::UInt16(val) => (Literal::UInt16(*val), DataType::UInt16),
        Literal::Boolean(val) => (Literal::Boolean(*val), DataType::Boolean),
        Literal::Array(_items) => todo!(),
        Literal::String(val) => (Literal::String(val.clone()), DataType::String),
    }
}

pub fn try_search_function(
    name: &str,
    args: &[(Expr, DataType)],
    fn_registry: &FunctionRegistry,
) -> Option<(Expr, DataType)> {
    for func in fn_registry.search(name, args.len()) {
        if let Some(checked_args) = try_check_function(args, &func.signature.args_type) {
            return Some((
                Expr::FunctionCall {
                    function: func.clone(),
                    args: checked_args,
                },
                func.signature.return_type.clone(),
            ));
        }
    }

    None
}

pub fn try_check_function<'a, 'b>(
    args: &[(Expr, DataType)],
    sig_types: &[DataType],
) -> Option<Vec<Expr>> {
    let mut checked_args = Vec::new();
    for ((arg, arg_type), sig_type) in args.iter().zip(sig_types) {
        match try_coerce(arg_type, sig_type) {
            Some(casts) => {
                if casts.is_empty() {
                    checked_args.push(arg.clone());
                } else {
                    checked_args.push(Expr::Cast {
                        expr: Box::new(arg.clone()),
                        casts,
                    });
                }
            }
            None => {
                return None;
            }
        }
    }
    Some(checked_args)
}

pub fn try_coerce(src_ty: &DataType, dest_ty: &DataType) -> Option<Vec<Cast>> {
    // println!("try_coerce {:?}, {:?}, {:?}", &expr, &ty, required_type);
    match (src_ty, dest_ty) {
        (src_ty, dest_ty) if subtype(&src_ty, dest_ty) => Some(vec![]),
        (DataType::Nullable(src_ty), DataType::Nullable(dest_ty)) => {
            let casts = try_coerce(src_ty, dest_ty)?;
            Some(vec![Cast::MapNullable(casts)])
        }
        (src_ty, DataType::Nullable(dest_ty)) => {
            let mut casts = try_coerce(src_ty, dest_ty)?;
            casts.push(Cast::ToNullable);
            Some(casts)
        }
        (DataType::UInt8, DataType::UInt16) => Some(vec![Cast::UInt8ToUInt16]),
        (DataType::Int8, DataType::Int16) => Some(vec![Cast::Int8ToInt16]),
        (DataType::UInt8, DataType::Int16) => Some(vec![Cast::UInt8ToInt16]),
        _ => None,
    }
}

pub fn subtype(src: &DataType, dest: &DataType) -> bool {
    match (src, dest) {
        (src, dest) if src == dest => true,
        (_, DataType::Any) => true,
        (DataType::Hole, _) => true,
        (DataType::Array(src), DataType::Array(dest)) => subtype(src, dest),
        (DataType::Nullable(src), DataType::Nullable(dest)) => subtype(src, dest),
        _ => false,
    }
}
