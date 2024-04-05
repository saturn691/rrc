//! Converts the HIR to LIR (Low-level Intermediate Representation)

use hir::*;
use std::collections::HashMap;

const INDENT: &str = "    ";

/// Entry point for lowering the HIR to the LIR
pub fn lir_build(body: Body) -> Result<String, String> {
    let mut codegen: Codegen = Codegen::new(body);    
    codegen.build_lir()
}

struct Codegen {
    code: String,
    body: Body,

    /// The return type of the function
    return_type: String,

    /// Used for SSA purposes
    reg_id: usize,
    
    /// Due to the SSA nature of the codegen, we need to keep track of the
    /// mapping between places and registers
    place_map: HashMap<Place, String>,
    const_map: HashMap<Const, String>,
}

impl Codegen {
    pub fn new(body: Body) -> Self {
        Codegen {
            code: String::new(),
            body: body,
            return_type: String::new(),
            reg_id: 0,
            place_map: HashMap::new(),
            const_map: HashMap::new(),
        }
    }

    pub fn build_lir(&mut self) -> Result<String, String> {
        // Housekeeping
        self.return_type = self.body.local_decls[0].ty.to_string();

        self.build_fn()
    }

    fn build_fn(&mut self) -> Result<String, String> {
        let name = &self.body.name;
        
        self.code += format!("define {} @{}()", self.return_type, name)
            .as_str();
        self.code += " {\n";
        self.code += "start:\n";

        for block in self.body.basic_blocks.clone() {
            self.build_basic_block(&block);
        }
        
        self.code += "}\n";

        Ok(self.code.clone())
    }

    fn build_basic_block(&mut self, block: &BasicBlock) {
        let mut return_reg: String = String::new();

        // Statements
        for statement in &block.statements {
            match statement {
                Statement::Assign(place, rvalue) => {
                    return_reg = self.build_assign(place, rvalue);
                }
                _ => unimplemented!()
            }
        }

        // Terminator
        match block.terminator {
            Some(Terminator::Return) => {
                self.code += format!(
                    "{}ret {} {}\n", 
                    INDENT, self.return_type, return_reg
                ).as_str();
            },
            _ => {}
        }
    }

    fn build_assign(&mut self, place: &Place, rvalue: &Rvalue) -> String {
        let reg: String = self.get_unique_id();
        let mut ty: String = String::new();
        
        match place {
            Place::Local(id) => {
                ty = self.body.local_decls[*id].ty.to_string();
            }
        }
        
        match rvalue {
            Rvalue::Use(operand) => {
                // Update the hashmap
                self.place_map.insert(place.clone(), reg.clone());
                
                match operand {
                    Operand::Constant(constant) => {
                        self.code += format!("{}{} = add {} {}, 0\n", 
                            INDENT, reg, ty, constant.value
                        ).as_str();
                    }
                    _ => {}
                }
            },
            Rvalue::BinaryOp(op, operand1, operand2) => {
                self.build_binary(op, operand1, operand2, &reg, &ty);
                
                // Update the hashmap
                self.place_map.insert(place.clone(), reg.clone());
            }
        }

        reg
    }

    /// Lowers binary operations
    fn build_binary(
        &mut self, 
        op: &BinOp, 
        operand1: &Box<Operand>, 
        operand2: &Box<Operand>,
        reg: &String,
        ty: &String
    ) {
        let op_str = match op {
            BinOp::Add => "add",
            BinOp::Sub => "sub",
            BinOp::Mul => "mul",
            BinOp::Div => "sdiv",
            BinOp::Rem => "srem",
            _ => unimplemented!()
        };

        // Get the register name for the operands
        let reg1 = self.get_operand_reg(operand1);
        let reg2 = self.get_operand_reg(operand2);

        self.code += format!("{}{} = {} {} {}, {}\n", 
            INDENT, reg, op_str, ty, reg1, reg2
        ).as_str();
    }

    fn get_operand_reg(&self, operand: &Operand) -> String {
        match operand {
            Operand::Copy(place) => {
                match self.place_map.get(&place) {
                    Some(reg) => reg.clone(),
                    None => {
                        panic!("Place not found in place_map: {:?}", place)
                    }
                }
            },

            _ => panic!("Operand not supported")
        }
    }

    /// Get a unique identifier for a register for SSA
    fn get_unique_id(&mut self) -> String {
        let id = self.reg_id;
        self.reg_id += 1;
        format!("%{}", id)
    }
}
