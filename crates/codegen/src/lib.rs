//! Converts the HIR to LIR (Low-level Intermediate Representation)
use hir::*;
use std::collections::{HashMap, HashSet, VecDeque};

const INDENT: &str = "    ";

/// Entry point for lowering the HIR to the LIR
pub fn lir_build(body: Body) -> Result<String, String> {
    let mut codegen: Codegen = Codegen::new(body);    
    codegen.build_lir()
}

struct Codegen {
    code: String,
    body: Body,

    /// Used for SSA purposes
    reg_id: usize,

    /// Due to the SSA nature of the codegen, we need to keep track of the
    /// mapping between places and registers
    place_map: HashMap<Place, String>,
}

impl Codegen {
    pub fn new(body: Body) -> Self {
        Codegen {
            code: String::new(),
            body: body,
            reg_id: 0,
            place_map: HashMap::new(),
        }
    }

    pub fn build_lir(&mut self) -> Result<String, String> {
        self.build_fn()
    }

    fn build_fn(&mut self) -> Result<String, String> {
        let name = &self.body.name;
        
        // Header
        self.code += format!(
            "define {} @{}()", 
            self.return_decl().ty.to_string(), 
            name
        ).as_str();

        self.code += " {\n";
        self.build_fn_code();
        self.code += "}\n";

        Ok(self.code.clone())
    }

    /// Uses graph traversal to build the function code.
    /// Separated from the main function to improve readability.
    fn build_fn_code(&mut self) {
        let mut visited = HashSet::new();
        let mut q = VecDeque::new();
        q.push_back(BasicBlock(0));
        
        // Add local decls
        self.code += "start:\n";
        for (i, local_decl) in self.body.local_decls.iter().enumerate() {
            match &local_decl.local_info {
                LocalInfo::User(id) => {
                    self.code += format!(
                        "{}%{} = alloca {}, align {}\n", 
                        INDENT, 
                        id, 
                        local_decl.ty.to_string(),
                        local_decl.ty.size()
                    ).as_str();
                    self.place_map.insert(Place::Local(i), format!("%{}", id));
                }
                LocalInfo::Temp => {},
            }
        }
        self.code += format!("{}br label %bb0\n", INDENT).as_str();

        while let Some(block) = q.pop_front() {
            if visited.insert(block) {
                // Add header
                self.code += format!("bb{}:\n", block.0).as_str();
                self.build_basic_block(block);

                let blockdata = &self.body.basic_blocks[block.0];

                for successor in blockdata.terminator().successors() {
                    q.push_back(successor);
                }
            }
        }
    }

    /// Builds a basic block
    /// e.g.
    /// bb0:
    ///     %1 = add i32 %0, 0      (statement)
    ///     ret i32 %1              (terminator)
    fn build_basic_block(&mut self, block: BasicBlock) {
        let mut return_reg: String = String::new();
        let blockdata = self.body.basic_blocks[block.0].clone();
        
        // Statements
        for statement in blockdata.statements.iter() {
            match statement {
                Statement::Assign(place, rvalue) => {
                    return_reg = self.build_assign(place, rvalue);
                }
                _ => unimplemented!()
            }
        }

        // Terminator
        match &blockdata.terminator {
            Some(Terminator::Return) => {
                let return_reg = self.get_unique_reg();
                self.code += format!(
                    "{}{} = load {}, {}* {}\n", 
                    INDENT, 
                    return_reg, 
                    self.return_decl().ty.to_string(), 
                    self.return_decl().ty.to_string(),
                    self.place_map.get(&Place::Local(0)).unwrap()
                ).as_str();
                self.code += format!(
                    "{}ret {} {}\n", 
                    INDENT, 
                    self.return_decl().ty.to_string(), 
                    return_reg
                ).as_str();
            },
            Some(Terminator::SwitchInt { value, targets } ) => {
                let label_true = &targets.blocks[0];
                let label_false =  &targets.blocks[1];

                let reg = self.get_operand_reg(value);

                self.code += format!(
                    "{}br {} {}, label %bb{}, label %bb{}\n",
                    INDENT,
                    self.body.get_operand_type(&value),
                    reg,
                    label_true.0,
                    label_false.0
                ).as_str()

            },
            Some(Terminator::Goto { target }) => {
                self.code += format!(
                    "{}br label %bb{}\n",
                    INDENT,
                    target.0
                ).as_str()  
            },
            _ => {}
        }
    }

    /// Builds a basic block assignment
    /// e.g.
    /// %1 = add i32 %0, 0
    fn build_assign(
        &mut self, 
        place: &Place, 
        rvalue: &Rvalue, 
    ) -> String {
        let mut reg: String;
        let ty: String;
        let mut ptr: bool = false;

        match place {
            Place::Local(id) => {
                let decl: &LocalDecl = &self.body.local_decls[*id];
                ty = decl.ty.to_string();
                match &decl.local_info {
                    LocalInfo::User(id) => {
                        reg = format!("%{}", id);                        
                        ptr = true;
                    },
                    LocalInfo::Temp => {
                        reg = self.get_unique_reg();
                    }
                }
            }
        }
        
        match rvalue {
            Rvalue::Use(operand) => {
                self.build_use(operand, &reg, &ty, ptr);       
            },
            Rvalue::BinaryOp(op, operand1, operand2) => {
                if ptr {
                    let reg1 = self.get_unique_reg();
                    self.build_binary(op, operand1, operand2, &reg1);
                    self.store(&reg1, &ty, place);
                    } else {
                        self.build_binary(op, operand1, operand2, &reg);
                    }
                }
                
                Rvalue::UnaryOp(op, operand) => {
                    if ptr {
                        let reg1 = self.get_unique_reg();
                        self.load(&reg1, &ty, place);
                        self.place_map.insert(place.clone(), reg1.clone());
                        let reg2 = self.get_unique_reg();
                        self.build_unary(op, operand, &reg2, &ty);
                        self.store(&reg2, &ty, place);
                    } else {
                        self.build_unary(op, operand, &reg, &ty);
                    }
            }
        }
        
        // Update the hashmap (after to avoid self-referential issues in UnaryOp)
        self.place_map.insert(place.clone(), reg.clone());
        
        reg
    }

    fn build_use(
        &mut self,
        operand: &Operand,
        reg: &String,
        ty: &String,
        ptr: bool
    ) {
        match operand {
            Operand::Constant(constant) => {
                if ptr {
                    self.code += format!("{}store {} {}, {}* {}\n", 
                        INDENT, ty, constant.value, ty, reg
                    ).as_str();
                } else {
                    self.code += format!("{}{} = add {} {}, 0\n", 
                        INDENT, reg, ty, constant.value
                    ).as_str();
                }
            },
            Operand::Copy(place) => {
                self.load(reg, ty, place);
            }
            _ => {}
        }
    }

    /// Lowers binary operations
    fn build_binary(
        &mut self, 
        op: &BinOp, 
        operand1: &Box<Operand>, 
        operand2: &Box<Operand>,
        reg: &String,
    ) {
        let op_str = match op {
            BinOp::Add => "add",
            BinOp::Sub => "sub",
            BinOp::Mul => "mul",
            BinOp::Div => "sdiv",
            BinOp::Rem => "srem",
            BinOp::ShiftLeft => "shl",
            BinOp::ShiftRight => "lshr",
            BinOp::BitAnd => "and",
            BinOp::BitOr => "or",
            BinOp::BitXor => "xor",
            BinOp::Eq => "icmp eq",
            BinOp::Ne => "icmp ne",
            BinOp::Lt => "icmp slt",
            BinOp::Gt => "icmp sgt",
            BinOp::Le => "icmp sle",
            BinOp::Ge => "icmp sge",
            _ => unimplemented!()
        };

        // Get the register name for the operands
        let reg1 = self.get_operand_reg(operand1);
        let reg2 = self.get_operand_reg(operand2);

        let ty = self.get_binary_type(operand1, operand2).to_string();

        self.code += format!("{}{} = {} {} {}, {}\n", 
            INDENT, reg, op_str, ty, reg1, reg2
        ).as_str();
    }

    /// Lowers unary operations
    fn build_unary(
        &mut self, 
        op: &UnOp, 
        operand: &Box<Operand>,
        reg: &String,
        ty: &String
    ) {
        // Get the register name for the operand
        let reg1 = self.get_operand_reg(operand);

        // Update the hashmap
        self.place_map.insert(Place::Local(self.reg_id), reg.clone());

        match op {
            UnOp::Neg => {
                self.code += format!("{}{} = sub {} 0, {}\n", 
                    INDENT, reg, ty, reg1
                ).as_str();
            },
            UnOp::Not => {
                self.code += format!("{}{} = xor {} {}, -1\n", 
                    INDENT, reg, ty, reg1
                ).as_str();
            },
        }
    }

    /// Helper function to store
    /// Output: store i32 %reg, i32* %place
    fn store(&mut self, reg: &String, ty: &String, place: &Place) {
        match place {
            Place::Local(id) => {
                let decl: &LocalDecl = &self.body.local_decls[*id];
                match &decl.local_info {
                    LocalInfo::User(id) => {
                        self.code += format!("{}store {} {}, {}* %{}\n", 
                            INDENT, ty, reg, ty, id
                        ).as_str();
                    },
                    LocalInfo::Temp => {
                        // Without the * in the store instruction
                        self.code += format!("{}store {} {}, {} {}\n", 
                            INDENT, ty, reg, ty, self.place_map[place]
                        ).as_str();

                    },
                }            
            }
        }
    }

    /// Helper function to load
    /// Output: %reg = load i32, i32* %place
    fn load(&mut self, reg: &String, ty: &String, place: &Place) {
        self.code += format!("{}{} = load {}, {}* {}\n", 
            INDENT, reg, ty, ty, self.place_map[place]
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

    /// Helper function to get the types of the operand in binary
    fn get_binary_type(&self, operand1: &Operand, operand2: &Operand) -> ast::Type {
        let ty1 = self.get_unary_type(operand1);
        let ty2 = self.get_unary_type(operand2);

        // Return the higher type
        if ty1.size() > ty2.size() {
            ty1
        } else {
            ty2
        }
    }

    /// Helper function to get the types of the operand in unary
    fn get_unary_type(&self, operand: &Operand) -> ast::Type {
        match operand {
            Operand::Copy(place) => {
                let idx = match place {
                    Place::Local(idx) => *idx,
                };
                self.body.local_decls[idx].ty
            }
            Operand::Constant(constant) => constant.ty,
            _ => unimplemented!()
        }
    }

    /// Get a unique identifier for a register for SSA
    fn get_unique_reg(&mut self) -> String {
        let id = self.reg_id;
        self.reg_id += 1;
        format!("%{}", id)
    }

    /// Helper function to get the return declaration
    fn return_decl(&self) -> &LocalDecl {
        &self.body.local_decls[0]
    }
}
