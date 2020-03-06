use std::rc::Rc;

use crate::common::Result;
use crate::xmile;
use crate::Project;

pub struct Var {
    direct_deps: Vec<String>,
}

pub struct Module {
    model: Rc<xmile::Model>,
    vars: Vec<Rc<Var>>,
}

impl Module {
    fn new(
        _project: &Project,
        _parent: Option<Rc<Var>>,
        _model: Rc<xmile::Model>,
        _vmodule: Option<Rc<Var>>,
    ) -> Result<Module> {
        return err!("Module::new not implemented");
    }
}

pub struct Simulation {
    module: Rc<Module>,
    // spec
    // slab
    // curr
    // next
    // nvars
    // nsaves
    // nsteps
    // step
    // save_step
    // save_every
}

impl Simulation {
    pub fn new(project: &Project, model: Rc<xmile::Model>) -> Result<Simulation> {
        let _module = Module::new(project, None, model, None);
        // create AModule for model
        // creates avars for all vars in model + recursive submodules

        // avar_init(module)

        // module assign offsets

        // sort runlists

        // reset

        err!("Simulation::new not implemented")
    }
}