// Copyright 2024 RisingLight Project Authors. Licensed under Apache-2.0.

//! Analyze the output schema of plans.

use super::*;

/// The data type of schema analysis.
pub type Schema = Vec<Id>;

/// Returns the output expressions for plan node.
pub fn analyze_schema(enode: &Expr, x: impl Fn(&Id) -> Schema) -> Schema {
    use Expr::*;
    let concat = |v1: Vec<Id>, v2: Vec<Id>| v1.into_iter().chain(v2).collect();
    match enode {
        // equal to child
        Filter([_, c]) | Order([_, c]) | Limit([_, _, c]) | TopN([_, _, _, c]) => x(c),

        // concat 2 children
        Join([_, _, l, r]) | HashJoin([_, _, _, l, r]) | MergeJoin([_, _, _, l, r]) => {
            concat(x(l), x(r))
        }

        // list is the source for the following nodes
        List(ids) => ids.to_vec(),

        // plans that change schema
        Scan([_, columns, _]) | Internal([_, columns]) => x(columns),
        Values(vs) => x(&vs[0]),
        Proj([exprs, _]) | Agg([exprs, _]) => x(exprs),
        Window([exprs, child]) => concat(x(child), x(exprs)),
        HashAgg([exprs, group_keys, _]) | SortAgg([exprs, group_keys, _]) => {
            concat(x(exprs), x(group_keys))
        }
        Empty(ids) => {
            let mut s = vec![];
            for id in ids.iter() {
                s.extend(x(id));
            }
            s
        }

        // not plan node
        _ => vec![],
    }
}

/// Returns true if the schema of two nodes is equal.
pub fn schema_is_eq(v1: &str, v2: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    let v1 = var(v1);
    let v2 = var(v2);
    move |egraph, _, subst| {
        let s1 = &egraph[subst[v1]].data.schema;
        let s2 = &egraph[subst[v2]].data.schema;
        s1 == s2
    }
}
