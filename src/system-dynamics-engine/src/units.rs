// Copyright 2021 The Model Authors. All rights reserved.
// Use of this source code is governed by the Apache License,
// Version 2.0, that can be found in the LICENSE file.

use std::collections::{BTreeMap, HashMap};
use std::result::Result as StdResult;

use crate::ast::{parse_equation, Expr};
use crate::common::{EquationError, Result};
use crate::datamodel::Unit;

type UnitMap = BTreeMap<String, i32>;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub struct Context {
    aliases: HashMap<String, String>,
    units: HashMap<String, UnitMap>,
}

impl Context {
    #[allow(dead_code)]
    fn new(units: &[Unit]) -> StdResult<Self, Vec<(String, Vec<EquationError>)>> {
        // step 1: build our base context consisting of all prime units
        let mut aliases = HashMap::new();
        let mut parsed_units = HashMap::new();
        for unit in units.iter().filter(|unit| unit.equation.is_none()) {
            for alias in unit.aliases.iter() {
                aliases.insert(alias.clone(), unit.name.clone());
            }
            parsed_units.insert(
                unit.name.clone(),
                [(unit.name.clone(), 1)].iter().cloned().collect(),
            );
        }

        let mut ctx = Context {
            aliases,
            units: parsed_units,
        };

        let mut unit_errors: Vec<(String, Vec<EquationError>)> = Vec::new();

        // step 2: use this base context to parse our units with equations
        for unit in units.iter().filter(|unit| unit.equation.is_some()) {
            for alias in unit.aliases.iter() {
                ctx.aliases.insert(alias.clone(), unit.name.clone());
            }

            let eqn = unit.equation.as_ref().unwrap();

            let ast = match parse_equation(eqn) {
                Ok(ast) => ast,
                Err(errors) => {
                    unit_errors.push((unit.name.clone(), errors));
                    continue;
                }
            };

            let unit_components = match ast {
                Some(ref ast) => match build_unit_components(&ctx, ast) {
                    Ok(unit_components) => unit_components,
                    Err(err) => {
                        unit_errors.push((
                            unit.name.clone(),
                            vec![EquationError {
                                start: 0,
                                end: 0,
                                code: err.code,
                            }],
                        ));
                        continue;
                    }
                },
                None => [(unit.name.clone(), 1)].iter().cloned().collect(),
            };

            ctx.units.insert(unit.name.clone(), unit_components);
        }

        if unit_errors.is_empty() {
            Ok(ctx)
        } else {
            Err(unit_errors)
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn build_unit_components(_ctx: &Context, _ast: &Expr) -> Result<UnitMap> {
    Ok(UnitMap::new())
}

#[test]
fn test_context_creation() {
    let simple_units = &[
        Unit {
            name: "time".to_owned(),
            equation: None,
            disabled: false,
            aliases: vec![],
        },
        Unit {
            name: "people".to_owned(),
            equation: None,
            disabled: false,
            aliases: vec!["person".to_owned(), "persons".to_owned()],
        },
    ];

    let expected = Context {
        aliases: [
            ("person".to_owned(), "people".to_owned()),
            ("persons".to_owned(), "people".to_owned()),
        ]
        .iter()
        .cloned()
        .collect(),
        units: [
            (
                "time".to_owned(),
                [("time".to_owned(), 1)].iter().cloned().collect(),
            ),
            (
                "people".to_owned(),
                [("people".to_owned(), 1)].iter().cloned().collect(),
            ),
        ]
        .iter()
        .cloned()
        .collect(),
    };

    assert_eq!(expected, Context::new(simple_units).unwrap());

    let _more_units = &[
        Unit {
            name: "time".to_owned(),
            equation: None,
            disabled: false,
            aliases: vec![],
        },
        Unit {
            name: "invtime".to_owned(),
            equation: Some("1/time".to_owned()),
            disabled: false,
            aliases: vec![],
        },
    ];

    let _expected2 = Context {
        aliases: HashMap::new(),
        units: [
            (
                "time".to_owned(),
                [("time".to_owned(), 1)].iter().cloned().collect(),
            ),
            (
                "invtime".to_owned(),
                [("time".to_owned(), -1)].iter().cloned().collect(),
            ),
        ]
        .iter()
        .cloned()
        .collect(),
    };

    // assert_eq!(expected2, Context::new(more_units).unwrap());
}

#[test]
fn test_basic_unit_checks() {
    // from a set of datamodel::Units build a Context

    // with a context, check if a set of variables unit checks
}