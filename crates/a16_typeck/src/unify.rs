//! Type unification for Hindley-Milner style type inference

use crate::context::TypeContext;
use crate::error::TypeError;
use crate::types::{Type, TypeVarId};
use a16_ast::Span;

impl TypeContext {
    /// Unify two types, recording substitutions for type variables.
    /// Returns the unified type on success, or records an error and returns Error type.
    pub fn unify(&mut self, t1: &Type, t2: &Type, span: Span) -> Type {
        let t1 = self.resolve(t1);
        let t2 = self.resolve(t2);
        
        match (&t1, &t2) {
            // Same types unify trivially
            _ if t1 == t2 => t1,
            
            // Error types propagate without additional errors
            (Type::Error, _) | (_, Type::Error) => Type::Error,
            
            // Any unifies with anything
            (Type::Any, _) => t2,
            (_, Type::Any) => t1,
            
            // Unknown (gradual typing) accepts anything
            (Type::Unknown, _) => t2,
            (_, Type::Unknown) => t1,
            
            // Type variables unify by substitution
            (Type::TypeVar(id), other) | (other, Type::TypeVar(id)) => {
                // Occurs check to prevent infinite types
                if self.occurs_check(*id, other) {
                    self.error(TypeError::RecursiveType {
                        span: miette::SourceSpan::new(
                            (span.start as usize).into(),
                            (span.end.saturating_sub(span.start)) as usize,
                        ),
                    });
                    Type::Error
                } else {
                    self.set_substitution(*id, other.clone());
                    other.clone()
                }
            }
            
            // Optional unification
            (Type::Optional(inner1), Type::Optional(inner2)) => {
                let inner = self.unify(inner1, inner2, span);
                Type::Optional(Box::new(inner))
            }
            // T unifies with Optional[T]
            (Type::Optional(inner), other) | (other, Type::Optional(inner)) => {
                if matches!(other, Type::None) {
                    t1.clone()
                } else {
                    let unified = self.unify(inner, other, span);
                    Type::Optional(Box::new(unified))
                }
            }
            
            // List unification
            (Type::List(elem1), Type::List(elem2)) => {
                let elem = self.unify(elem1, elem2, span);
                Type::List(Box::new(elem))
            }
            
            // Set unification
            (Type::Set(elem1), Type::Set(elem2)) => {
                let elem = self.unify(elem1, elem2, span);
                Type::Set(Box::new(elem))
            }
            
            // Dict unification
            (Type::Dict(k1, v1), Type::Dict(k2, v2)) => {
                let key = self.unify(k1, k2, span);
                let val = self.unify(v1, v2, span);
                Type::Dict(Box::new(key), Box::new(val))
            }
            
            // Tuple unification
            (Type::Tuple(elems1), Type::Tuple(elems2)) if elems1.len() == elems2.len() => {
                let elems: Vec<_> = elems1.iter()
                    .zip(elems2.iter())
                    .map(|(e1, e2)| self.unify(e1, e2, span))
                    .collect();
                Type::Tuple(elems)
            }
            
            // Function unification
            (Type::Function { params: p1, ret: r1 }, Type::Function { params: p2, ret: r2 }) 
                if p1.len() == p2.len() => 
            {
                let params: Vec<_> = p1.iter()
                    .zip(p2.iter())
                    .map(|(a, b)| self.unify(a, b, span))
                    .collect();
                let ret = self.unify(r1, r2, span);
                Type::Function { params, ret: Box::new(ret) }
            }
            
            // Async function unification
            (Type::AsyncFunction { params: p1, ret: r1 }, Type::AsyncFunction { params: p2, ret: r2 }) 
                if p1.len() == p2.len() => 
            {
                let params: Vec<_> = p1.iter()
                    .zip(p2.iter())
                    .map(|(a, b)| self.unify(a, b, span))
                    .collect();
                let ret = self.unify(r1, r2, span);
                Type::AsyncFunction { params, ret: Box::new(ret) }
            }
            
            // Union types
            (Type::Union(types1), Type::Union(types2)) => {
                // Merge union types
                let mut all_types = types1.clone();
                for t in types2 {
                    if !all_types.contains(t) {
                        all_types.push(t.clone());
                    }
                }
                if all_types.len() == 1 {
                    all_types.pop().unwrap()
                } else {
                    Type::Union(all_types)
                }
            }
            (Type::Union(types), other) | (other, Type::Union(types)) => {
                // Check if other is a member of the union
                for t in types {
                    if self.types_compatible(t, other) {
                        return other.clone();
                    }
                }
                // Add to union
                let mut new_types = types.clone();
                new_types.push(other.clone());
                Type::Union(new_types)
            }
            
            // Result type unification
            (Type::Result(ok1, err1), Type::Result(ok2, err2)) => {
                let ok = self.unify(ok1, ok2, span);
                let err = self.unify(err1, err2, span);
                Type::Result(Box::new(ok), Box::new(err))
            }
            
            // Iterator unification
            (Type::Iterator(elem1), Type::Iterator(elem2)) => {
                let elem = self.unify(elem1, elem2, span);
                Type::Iterator(Box::new(elem))
            }
            
            // Schema unification
            (Type::Schema(inner1), Type::Schema(inner2)) => {
                let inner = self.unify(inner1, inner2, span);
                Type::Schema(Box::new(inner))
            }
            
            // Numeric widening: Int can become Float
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
            
            // Named types must match exactly
            (Type::Class(n1), Type::Class(n2)) if n1 == n2 => t1,
            (Type::Struct(n1), Type::Struct(n2)) if n1 == n2 => t1,
            (Type::Enum(n1), Type::Enum(n2)) if n1 == n2 => t1,
            (Type::Agent(n1), Type::Agent(n2)) if n1 == n2 => t1,
            (Type::Tool(n1), Type::Tool(n2)) if n1 == n2 => t1,
            (Type::Memory(n1), Type::Memory(n2)) if n1 == n2 => t1,
            (Type::Prompt(n1), Type::Prompt(n2)) if n1 == n2 => t1,
            
            // Unification failed
            _ => {
                self.error(TypeError::unification_failed(&t1.to_string(), &t2.to_string(), span));
                Type::Error
            }
        }
    }
    
    /// Resolve a type by following type variable substitutions
    pub fn resolve(&self, ty: &Type) -> Type {
        match ty {
            Type::TypeVar(id) => {
                if let Some(subst) = self.get_substitution(*id) {
                    self.resolve(subst)
                } else {
                    ty.clone()
                }
            }
            // Recursively resolve container types
            Type::List(elem) => Type::List(Box::new(self.resolve(elem))),
            Type::Set(elem) => Type::Set(Box::new(self.resolve(elem))),
            Type::Dict(k, v) => Type::Dict(Box::new(self.resolve(k)), Box::new(self.resolve(v))),
            Type::Tuple(elems) => Type::Tuple(elems.iter().map(|e| self.resolve(e)).collect()),
            Type::Optional(inner) => Type::Optional(Box::new(self.resolve(inner))),
            Type::Union(types) => Type::Union(types.iter().map(|t| self.resolve(t)).collect()),
            Type::Function { params, ret } => Type::Function {
                params: params.iter().map(|p| self.resolve(p)).collect(),
                ret: Box::new(self.resolve(ret)),
            },
            Type::AsyncFunction { params, ret } => Type::AsyncFunction {
                params: params.iter().map(|p| self.resolve(p)).collect(),
                ret: Box::new(self.resolve(ret)),
            },
            Type::Iterator(elem) => Type::Iterator(Box::new(self.resolve(elem))),
            Type::Result(ok, err) => Type::Result(Box::new(self.resolve(ok)), Box::new(self.resolve(err))),
            Type::Schema(inner) => Type::Schema(Box::new(self.resolve(inner))),
            _ => ty.clone(),
        }
    }
    
    /// Check if a type variable occurs in a type (occurs check for infinite type prevention)
    fn occurs_check(&self, var: TypeVarId, ty: &Type) -> bool {
        match ty {
            Type::TypeVar(id) => {
                if *id == var {
                    true
                } else if let Some(subst) = self.get_substitution(*id) {
                    self.occurs_check(var, subst)
                } else {
                    false
                }
            }
            Type::List(elem) | Type::Set(elem) | Type::Optional(elem) | 
            Type::Iterator(elem) | Type::Schema(elem) => {
                self.occurs_check(var, elem)
            }
            Type::Dict(k, v) | Type::Result(k, v) => {
                self.occurs_check(var, k) || self.occurs_check(var, v)
            }
            Type::Tuple(elems) | Type::Union(elems) => {
                elems.iter().any(|e| self.occurs_check(var, e))
            }
            Type::Function { params, ret } | Type::AsyncFunction { params, ret } => {
                params.iter().any(|p| self.occurs_check(var, p)) || self.occurs_check(var, ret)
            }
            _ => false,
        }
    }
    
    /// Check if two types are compatible (without unification)
    pub fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        let t1 = self.resolve(t1);
        let t2 = self.resolve(t2);
        
        match (&t1, &t2) {
            _ if t1 == t2 => true,
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (Type::Error, _) | (_, Type::Error) => true,
            (Type::TypeVar(_), _) | (_, Type::TypeVar(_)) => true,
            (Type::Optional(inner), other) | (other, Type::Optional(inner)) => {
                matches!(other, Type::None) || self.types_compatible(inner, other)
            }
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
            (Type::List(e1), Type::List(e2)) => self.types_compatible(e1, e2),
            (Type::Set(e1), Type::Set(e2)) => self.types_compatible(e1, e2),
            (Type::Dict(k1, v1), Type::Dict(k2, v2)) => {
                self.types_compatible(k1, k2) && self.types_compatible(v1, v2)
            }
            (Type::Tuple(e1), Type::Tuple(e2)) if e1.len() == e2.len() => {
                e1.iter().zip(e2.iter()).all(|(a, b)| self.types_compatible(a, b))
            }
            (Type::Union(types), other) | (other, Type::Union(types)) => {
                types.iter().any(|t| self.types_compatible(t, other))
            }
            _ => false,
        }
    }
    
    /// Check if a type is a subtype of another (for assignment/passing)
    pub fn is_subtype(&self, sub: &Type, sup: &Type) -> bool {
        let sub = self.resolve(sub);
        let sup = self.resolve(sup);
        
        match (&sub, &sup) {
            _ if sub == sup => true,
            (_, Type::Any) => true,
            (_, Type::Unknown) => true,
            (Type::Never, _) => true,
            (Type::None, Type::Optional(_)) => true,
            (inner, Type::Optional(opt_inner)) => self.is_subtype(inner, opt_inner),
            (Type::Int, Type::Float) => true, // Numeric widening
            (Type::List(e1), Type::List(e2)) => self.is_subtype(e1, e2),
            (_, Type::Union(types)) => types.iter().any(|t| self.is_subtype(&sub, t)),
            (Type::Union(types), _) => types.iter().all(|t| self.is_subtype(t, &sup)),
            _ => false,
        }
    }
}
