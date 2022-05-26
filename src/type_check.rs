use std::sync::Arc;

use crate::{
    expr::{Cast, Expr, Literal, AST},
    function::{Function, FunctionRegistry},
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
        AST::FunctionCall { name, args, params } => {
            let args = args
                .iter()
                .map(|arg| check(arg, fn_registry))
                .collect::<Option<Vec<_>>>()?;

            check_function(name, &args, params, fn_registry)
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

pub fn check_function(
    name: &str,
    args: &[(Expr, DataType)],
    params: &[usize],
    fn_registry: &FunctionRegistry,
) -> Option<(Expr, DataType)> {
    let args_ty = args.iter().map(|(_, ty)| ty).collect::<Vec<_>>();
    for func in search_function_candidates(name, &args_ty, params, fn_registry) {
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

pub fn search_function_candidates(
    name: &str,
    args_ty: &[&DataType],
    params: &[usize],
    fn_registry: &FunctionRegistry,
) -> Vec<Arc<Function>> {
    if params.is_empty() {
        let normal_funcs = fn_registry
            .funcs
            .iter()
            .filter(|func| {
                func.signature.name == name && func.signature.args_type.len() == args_ty.len()
            })
            .map(Arc::clone)
            .collect::<Vec<_>>();

        if !normal_funcs.is_empty() {
            return normal_funcs;
        }
    }

    fn_registry
        .factories
        .get(name)
        .and_then(|factory| factory(params, args_ty))
        .map(|func| vec![func])
        .unwrap_or(Vec::new())
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
