// Copyright 2020 The Model Authors. All rights reserved.
// Use of this source code is governed by the Apache License,
// Version 2.0, that can be found in the LICENSE file.

use std::fs::File;
use std::io::{BufReader, Write};
use std::rc::Rc;

use pico_args::Arguments;

use system_dynamics_compat::engine::datamodel::Equation;
use system_dynamics_compat::engine::{eprintln, serde, ErrorCode, Project, Simulation, VM};
use system_dynamics_compat::prost::Message;
use system_dynamics_compat::{open_vensim, open_xmile};

const VERSION: &str = "1.0";
const EXIT_FAILURE: i32 = 1;

#[macro_export]
macro_rules! die(
    ($($arg:tt)*) => { {
        use std;
        eprintln!($($arg)*);
        std::process::exit(EXIT_FAILURE)
    } }
);

fn usage() -> ! {
    let argv0 = std::env::args()
        .next()
        .unwrap_or_else(|| "<mdl>".to_string());
    die!(
        concat!(
            "mdl {}: Simulate system dynamics models.\n\
         \n\
         USAGE:\n",
            "    {} [SUBCOMMAND] [OPTION...] PATH\n",
            "\n\
         OPTIONS:\n",
            "    -h, --help    show this message\n",
            "    --vensim      model is a Vensim .mdl file\n",
            "    --model-only  for conversion, only output model instead of project\n",
            "    --output FILE path to write output file\n",
            "\n\
         SUBCOMMANDS:\n",
            "    simulate      Simulate a model and display output\n",
            "    convert       Convert an XMILE or Vensim model to protobuf\n"
        ),
        VERSION,
        argv0
    );
}

#[derive(Clone, Default, Debug)]
struct Args {
    path: Option<String>,
    output: Option<String>,
    is_vensim: bool,
    is_convert: bool,
    is_model_only: bool,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut parsed = Arguments::from_env();
    if parsed.contains(["-h", "--help"]) {
        usage();
    }

    let subcommand = parsed.subcommand()?;
    if subcommand.is_none() {
        eprintln!("error: subcommand required");
        usage();
    }

    let mut args: Args = Default::default();

    let subcommand = subcommand.unwrap();
    if subcommand == "convert" {
        args.is_convert = true;
    } else if subcommand == "simulate" {
    } else {
        eprintln!("error: unknown subcommand {}", subcommand);
        usage();
    }

    args.output = parsed.value_from_str("--output").ok();
    args.is_model_only = parsed.contains("--model-only");
    args.is_vensim = parsed.contains("--vensim");

    let free_arguments = parsed.free()?;
    if free_arguments.is_empty() {
        eprintln!("error: input path required");
        usage();
    }

    args.path = Some(free_arguments[0].clone());

    Ok(args)
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("error: {}", err);
            usage();
        }
    };
    let file_path = args.path.unwrap_or_else(|| "/dev/stdin".to_string());
    let file = File::open(&file_path).unwrap();
    let mut reader = BufReader::new(file);

    let project = if args.is_vensim {
        open_vensim(&mut reader)
    } else {
        open_xmile(&mut reader)
    };

    if project.is_err() {
        eprintln!("model '{}' error: {}", &file_path, project.err().unwrap());
        return;
    };

    let project = project.unwrap();

    if args.is_convert {
        let pb_project = serde::serialize(&project);

        let buf: Vec<u8> = if args.is_model_only {
            if pb_project.models.len() != 1 {
                die!("--model-only specified, but more than 1 model in this project");
            }
            let mut buf = Vec::with_capacity(pb_project.models[0].encoded_len());
            pb_project.models[0].encode(&mut buf).unwrap();
            buf
        } else {
            let mut buf = Vec::with_capacity(pb_project.encoded_len());
            pb_project.encode(&mut buf).unwrap();
            buf
        };

        let mut output_file =
            File::create(&args.output.unwrap_or_else(|| "/dev/stdout".to_string())).unwrap();
        output_file.write_all(&buf).unwrap();
    // eprintln!("{:?}", project);
    } else {
        let project_datamodel = project.clone();
        let project = Rc::new(Project::from(project));
        let mut found_model_error = false;
        for (model_name, model) in project.models.iter() {
            let model_datamodel = project_datamodel.get_model(model_name);
            if model_datamodel.is_none() {
                continue;
            }
            let model_datamodel = model_datamodel.unwrap();
            let mut found_var_error = false;
            for (ident, errors) in model.get_variable_errors() {
                assert!(!errors.is_empty());
                let var = model_datamodel.get_variable(&ident).unwrap();
                found_var_error = true;
                for error in errors {
                    eprintln!();
                    if let Some(Equation::Scalar(eqn)) = var.get_equation() {
                        eprintln!("    {}", eqn);
                        let space = std::iter::repeat(" ")
                            .take(error.start as usize)
                            .collect::<String>();
                        let underline = std::iter::repeat("~")
                            .take((error.end - error.start) as usize)
                            .collect::<String>();
                        eprintln!("    {}{}", space, underline);
                    }
                    eprintln!(
                        "error in model '{}' variable '{}': {}",
                        model_name, ident, error.code
                    );
                }
            }
            if let Some(errors) = &model.errors {
                for error in errors.iter() {
                    if error.code == ErrorCode::VariablesHaveErrors && found_var_error {
                        continue;
                    }
                    eprintln!("error in model {}: {}", model_name, error);
                    found_model_error = true;
                }
            }
        }
        let sim = match Simulation::new(&project, "main") {
            Ok(sim) => sim,
            Err(err) => {
                if !(err.code == ErrorCode::NotSimulatable && found_model_error) {
                    eprintln!("error: {}", err);
                }
                std::process::exit(1);
            }
        };
        let compiled = sim.compile().unwrap();
        let vm = VM::new(compiled).unwrap();
        let results = vm.run_to_end();
        let _results = results.unwrap();
        // results.print_tsv();
    }
}
