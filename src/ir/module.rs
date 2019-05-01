// Copyright (c) 2017-2019 Fabian Schuiki

//! Representation of linked LLHD units.
//!
//! This module implements the `Module`, a collection of LLHD `Function`,
//! `Process`, and `Entity` objects linked together. A module acts as the root
//! node of an LLHD intermediate representation, and is the unit of information
//! ingested by the reader and emitted by the writer.

use crate::{
    impl_table_key,
    ir::{DataFlowGraph, Entity, ExtUnit, Function, Process, Signature, Unit, UnitName},
    table::PrimaryTable,
    verifier::Verifier,
};
use std::collections::{BTreeSet, HashMap};

/// A module.
///
/// This is the root node of an LLHD intermediate representation. Contains
/// `Function`, `Process`, and `Entity` declarations and definitions.
pub struct Module {
    /// The units in this module.
    units: PrimaryTable<ModUnit, ModUnitData>,
    /// The order of units in the module.
    unit_order: BTreeSet<ModUnit>,
    /// The local link table. Maps an external unit declared within a unit to a
    /// unit in the module.
    link_table: Option<HashMap<(ModUnit, ExtUnit), ModUnit>>,
}

impl std::ops::Index<ModUnit> for Module {
    type Output = ModUnitData;
    fn index(&self, idx: ModUnit) -> &ModUnitData {
        &self.units[idx]
    }
}

impl std::ops::IndexMut<ModUnit> for Module {
    fn index_mut(&mut self, idx: ModUnit) -> &mut ModUnitData {
        self.link_table = None;
        &mut self.units[idx]
    }
}

impl Module {
    /// Create a new empty module.
    pub fn new() -> Self {
        Self {
            units: PrimaryTable::new(),
            unit_order: BTreeSet::new(),
            link_table: None,
        }
    }

    /// Dump the module in human-readable form.
    pub fn dump(&self) -> ModuleDumper {
        ModuleDumper(self)
    }

    /// Add a function to the module.
    pub fn add_function(&mut self, func: Function) -> ModUnit {
        self.add_unit(ModUnitData::Function(func))
    }

    /// Add a process to the module.
    pub fn add_process(&mut self, prok: Process) -> ModUnit {
        self.add_unit(ModUnitData::Process(prok))
    }

    /// Add an entity to the module.
    pub fn add_entity(&mut self, ent: Entity) -> ModUnit {
        self.add_unit(ModUnitData::Entity(ent))
    }

    /// Declare an external unit.
    pub fn declare(&mut self, name: UnitName, sig: Signature) -> ModUnit {
        self.add_unit(ModUnitData::Declare { sig, name })
    }

    /// Add a unit to the module.
    fn add_unit(&mut self, data: ModUnitData) -> ModUnit {
        let unit = self.units.add(data);
        self.unit_order.insert(unit);
        self.link_table = None;
        unit
    }

    /// Remove a unit from the module.
    pub fn remove_unit(&mut self, unit: ModUnit) {
        self.units.remove(unit);
        self.unit_order.remove(&unit);
    }

    /// Return an iterator over the units in this module.
    pub fn units<'a>(&'a self) -> impl Iterator<Item = ModUnit> + 'a {
        self.unit_order.iter().cloned()
    }

    /// Return an iterator over the functions in this module.
    pub fn functions<'a>(&'a self) -> impl Iterator<Item = &'a Function> + 'a {
        self.units().flat_map(move |unit| self[unit].get_function())
    }

    /// Return an iterator over the processes in this module.
    pub fn processes<'a>(&'a self) -> impl Iterator<Item = &'a Process> + 'a {
        self.units().flat_map(move |unit| self[unit].get_process())
    }

    /// Return an iterator over the entities in this module.
    pub fn entities<'a>(&'a self) -> impl Iterator<Item = &'a Entity> + 'a {
        self.units().flat_map(move |unit| self[unit].get_entity())
    }

    /// Return an iterator over the external unit declarations in this module.
    pub fn declarations<'a>(&'a self) -> impl Iterator<Item = (&'a UnitName, &'a Signature)> + 'a {
        self.units()
            .flat_map(move |unit| self[unit].get_declaration())
    }

    /// Check whether a unit is a function.
    pub fn is_function(&self, unit: ModUnit) -> bool {
        self[unit].is_function()
    }

    /// Check whether a unit is a process.
    pub fn is_process(&self, unit: ModUnit) -> bool {
        self[unit].is_process()
    }

    /// Check whether a unit is an entity.
    pub fn is_entity(&self, unit: ModUnit) -> bool {
        self[unit].is_entity()
    }

    /// Check whether a unit is externally declared.
    pub fn is_declaration(&self, unit: ModUnit) -> bool {
        self[unit].is_declaration()
    }

    /// Get the name of a unit.
    pub fn unit_name(&self, unit: ModUnit) -> &UnitName {
        self[unit].name()
    }

    /// Get the signature of a unit.
    pub fn unit_sig(&self, unit: ModUnit) -> &Signature {
        self[unit].sig()
    }

    /// Return a function in the module, or `None` if the unit is not a
    /// function.
    pub fn get_function(&self, unit: ModUnit) -> Option<&Function> {
        self[unit].get_function()
    }

    /// Return a mutable function in the module, or `None` if the unit is not a
    /// function.
    pub fn get_function_mut(&mut self, unit: ModUnit) -> Option<&mut Function> {
        self.link_table = None;
        self[unit].get_function_mut()
    }

    /// Return a function in the module. Panic if the unit is not a function.
    pub fn function(&self, unit: ModUnit) -> &Function {
        self[unit].get_function().expect("unit is not a function")
    }

    /// Return a mutable function in the module. Panic if the unit is not a
    /// function.
    pub fn function_mut(&mut self, unit: ModUnit) -> &mut Function {
        self.link_table = None;
        self[unit]
            .get_function_mut()
            .expect("unit is not a function")
    }

    /// Return a process in the module, or `None` if the unit is not a
    /// process.
    pub fn get_process(&self, unit: ModUnit) -> Option<&Process> {
        self[unit].get_process()
    }

    /// Return a mutable process in the module, or `None` if the unit is not a
    /// process.
    pub fn get_process_mut(&mut self, unit: ModUnit) -> Option<&mut Process> {
        self.link_table = None;
        self[unit].get_process_mut()
    }

    /// Return a process in the module. Panic if the unit is not a process.
    pub fn process(&self, unit: ModUnit) -> &Process {
        self[unit].get_process().expect("unit is not a process")
    }

    /// Return a mutable process in the module. Panic if the unit is not a
    /// process.
    pub fn process_mut(&mut self, unit: ModUnit) -> &mut Process {
        self.link_table = None;
        self[unit].get_process_mut().expect("unit is not a process")
    }

    /// Return an entity in the module, or `None` if the unit is not an
    /// entity.
    pub fn get_entity(&self, unit: ModUnit) -> Option<&Entity> {
        self[unit].get_entity()
    }

    /// Return a mutable entity in the module, or `None` if the unit is not an
    /// entity.
    pub fn get_entity_mut(&mut self, unit: ModUnit) -> Option<&mut Entity> {
        self.link_table = None;
        self[unit].get_entity_mut()
    }

    /// Return an entity in the module. Panic if the unit is not an entity.
    pub fn entity(&self, unit: ModUnit) -> &Entity {
        self[unit].get_entity().expect("unit is not an entity")
    }

    /// Return a mutable entity in the module. Panic if the unit is not an
    /// entity.
    pub fn entity_mut(&mut self, unit: ModUnit) -> &mut Entity {
        self.link_table = None;
        self[unit].get_entity_mut().expect("unit is not an entity")
    }

    /// Return an iterator over the symbols in the module.
    pub fn symbols<'a>(&'a self) -> impl Iterator<Item = (&UnitName, ModUnit)> + 'a {
        self.units().map(move |unit| (self[unit].name(), unit))
    }

    /// Return an iterator over the local symbols in the module.
    pub fn local_symbols<'a>(&'a self) -> impl Iterator<Item = (&UnitName, ModUnit)> + 'a {
        self.symbols().filter(|&(name, _)| name.is_local())
    }

    /// Return an iterator over the global symbols in the module.
    pub fn global_symbols<'a>(&'a self) -> impl Iterator<Item = (&UnitName, ModUnit)> + 'a {
        self.symbols().filter(|&(name, _)| name.is_global())
    }

    /// Check whether the module is internally linked.
    ///
    /// Adding or modifying a unit invalidates the linkage within the module.
    pub fn is_linked(&self) -> bool {
        self.link_table.is_some()
    }

    /// Locally link the module.
    pub fn link(&mut self) {
        let mut failed = false;

        // Collect a table of symbols that we can resolve against.
        let mut symbols = HashMap::new();
        for (name, unit) in self.symbols() {
            if let Some(existing) = symbols.insert(name, unit) {
                if !self[existing].is_declaration() {
                    eprintln!("unit {} declared multiple times", name);
                    failed = true;
                }
            }
        }
        if failed {
            panic!("linking failed; multiple uses of the same name");
        }

        // Resolve the external units in each unit.
        let mut linked = HashMap::new();
        for unit in self.units() {
            let dfg = match self[unit].get_dfg() {
                Some(dfg) => dfg,
                None => continue,
            };
            for (ext_unit, data) in dfg.ext_units.iter() {
                let to = match symbols.get(&data.name).cloned() {
                    Some(to) => to,
                    None => {
                        eprintln!(
                            "unit {} not found; referenced in {}",
                            data.name,
                            self.unit_name(unit)
                        );
                        failed = true;
                        continue;
                    }
                };
                if self.unit_sig(to) != &data.sig {
                    eprintln!(
                        "signature mismatch: {} has {}, but reference in {} expects {}",
                        data.name,
                        self.unit_sig(to),
                        self.unit_name(unit),
                        data.sig
                    );
                    failed = true;
                    continue;
                }
                linked.insert((unit, ext_unit), to);
            }
        }
        if failed {
            panic!("linking failed; unresolved references");
        }
        self.link_table = Some(linked);
    }

    /// Panic if the module is not well-formed.
    pub fn verify(&self) {
        let mut verifier = Verifier::new();
        verifier.verify_module(self);
        match verifier.finish() {
            Ok(()) => (),
            Err(errs) => {
                eprintln!("");
                eprintln!("Verified module:");
                eprintln!("{}", self.dump());
                eprintln!("");
                eprintln!("Verification errors:");
                eprintln!("{}", errs);
                panic!("verification failed");
            }
        }
    }
}

/// Temporary object to dump a `Module` in human-readable form for debugging.
pub struct ModuleDumper<'a>(&'a Module);

impl std::fmt::Display for ModuleDumper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut newline = false;
        for unit in self.0.units() {
            if newline {
                writeln!(f, "")?;
                writeln!(f, "")?;
            }
            newline = true;
            write!(f, "{}: ", unit)?;
            match &self.0[unit] {
                ModUnitData::Function(unit) => write!(f, "{}", unit.dump())?,
                ModUnitData::Process(unit) => write!(f, "{}", unit.dump())?,
                ModUnitData::Entity(unit) => write!(f, "{}", unit.dump())?,
                ModUnitData::Declare { sig, name } => write!(f, "declare {} {}", name, sig)?,
            }
        }
        Ok(())
    }
}

impl_table_key! {
    /// An unit definition or declaration in a module.
    struct ModUnit(u32) as "u";
}

/// Internal table storage for units in a module.
pub enum ModUnitData {
    /// The unit is a function.
    Function(Function),
    /// The unit is a process.
    Process(Process),
    /// The unit is an entity.
    Entity(Entity),
    /// The unit is a declaration of an external unit.
    Declare { sig: Signature, name: UnitName },
}

impl ModUnitData {
    /// If this unit is a function, return it. Otherwise return `None`.
    pub fn get_function(&self) -> Option<&Function> {
        match self {
            ModUnitData::Function(unit) => Some(unit),
            _ => None,
        }
    }

    /// If this unit is a function, return it. Otherwise return `None`.
    pub fn get_function_mut(&mut self) -> Option<&mut Function> {
        match self {
            ModUnitData::Function(unit) => Some(unit),
            _ => None,
        }
    }

    /// If this unit is a process, return it. Otherwise return `None`.
    pub fn get_process(&self) -> Option<&Process> {
        match self {
            ModUnitData::Process(unit) => Some(unit),
            _ => None,
        }
    }

    /// If this unit is a process, return it. Otherwise return `None`.
    pub fn get_process_mut(&mut self) -> Option<&mut Process> {
        match self {
            ModUnitData::Process(unit) => Some(unit),
            _ => None,
        }
    }

    /// If this unit is an entity, return it. Otherwise return `None`.
    pub fn get_entity(&self) -> Option<&Entity> {
        match self {
            ModUnitData::Entity(unit) => Some(unit),
            _ => None,
        }
    }

    /// If this unit is an entity, return it. Otherwise return `None`.
    pub fn get_entity_mut(&mut self) -> Option<&mut Entity> {
        match self {
            ModUnitData::Entity(unit) => Some(unit),
            _ => None,
        }
    }

    /// If this unit is an external declaration, return it. Otherwise return
    /// `None`.
    pub fn get_declaration(&self) -> Option<(&UnitName, &Signature)> {
        match self {
            ModUnitData::Declare { sig, name } => Some((name, sig)),
            _ => None,
        }
    }

    /// If this unit is an external declaration, return it. Otherwise return
    /// `None`.
    pub fn get_declaration_mut(&mut self) -> Option<(&mut UnitName, &mut Signature)> {
        match self {
            ModUnitData::Declare { sig, name } => Some((name, sig)),
            _ => None,
        }
    }

    /// Check whether this is a function.
    pub fn is_function(&self) -> bool {
        match self {
            ModUnitData::Function(..) => true,
            _ => false,
        }
    }

    /// Check whether this is a process.
    pub fn is_process(&self) -> bool {
        match self {
            ModUnitData::Process(..) => true,
            _ => false,
        }
    }

    /// Check whether this is an entity.
    pub fn is_entity(&self) -> bool {
        match self {
            ModUnitData::Entity(..) => true,
            _ => false,
        }
    }

    /// Check whether this is a declaration of an external unit.
    pub fn is_declaration(&self) -> bool {
        match self {
            ModUnitData::Declare { .. } => true,
            _ => false,
        }
    }

    /// Return the signature of the unit.
    pub fn sig(&self) -> &Signature {
        match self {
            ModUnitData::Function(unit) => unit.sig(),
            ModUnitData::Process(unit) => unit.sig(),
            ModUnitData::Entity(unit) => unit.sig(),
            ModUnitData::Declare { sig, .. } => sig,
        }
    }

    /// Return the name of the unit.
    pub fn name(&self) -> &UnitName {
        match self {
            ModUnitData::Function(unit) => unit.name(),
            ModUnitData::Process(unit) => unit.name(),
            ModUnitData::Entity(unit) => unit.name(),
            ModUnitData::Declare { name, .. } => name,
        }
    }

    /// Return the data flow graph of the unit, if there is one.
    pub fn get_dfg(&self) -> Option<&DataFlowGraph> {
        match self {
            ModUnitData::Function(unit) => Some(unit.dfg()),
            ModUnitData::Process(unit) => Some(unit.dfg()),
            ModUnitData::Entity(unit) => Some(unit.dfg()),
            _ => None,
        }
    }

    /// Return the mutable data flow graph of the unit, if there is one.
    pub fn get_dfg_mut(&mut self) -> Option<&mut DataFlowGraph> {
        match self {
            ModUnitData::Function(unit) => Some(unit.dfg_mut()),
            ModUnitData::Process(unit) => Some(unit.dfg_mut()),
            ModUnitData::Entity(unit) => Some(unit.dfg_mut()),
            _ => None,
        }
    }
}