use swc_core::plugin::{
    plugin_transform,
    proxies::TransformPluginProgramMetadata
};

use swc_core::ecma::{
    ast::*,
    visit::VisitMut,
    atoms::JsWord
};

use swc_core::common::{DUMMY_SP, AstNode};
use swc_core::ecma::visit::{VisitMutWith, self};
use swc_ecma_utils::quote_ident;

use swc_ecma_quote::{
    quote,
    quote_expr
};

const PROGRAM_TOP_INDEX: usize = 0;
const PROGRAM_VOID_STMT_INDEX: usize = 1;

macro_rules! impl_node_type {
    (for $($t:ty),+) => {
        $(impl NodeType for $t {})*
    }
}


pub trait NodeType {}

impl_node_type!(
    for
    Class
);

pub struct PluginModuleExports {
    pub updated_body: Vec<ModuleItem>,

    pub has_module_exports_stmt: bool,
    pub has_void_stmt: bool
}

fn create_plugin_module_exports() -> impl VisitMut {
    PluginModuleExports {
        updated_body: vec![],

        has_module_exports_stmt: false,
        has_void_stmt: false
    }
}

impl PluginModuleExports {
    fn get_index(&self, module: &ModuleItem) -> usize {
        self.updated_body
            .iter()
            .position(|node| node == module)
            .expect("Cannot get index of a node.")
    }

    fn increment(&self, index: &mut usize) {
        *index += 1;
    }

    fn append_module_exports_stmt(&mut self, index: &usize) {
        self.has_module_exports_stmt = true;

        self.updated_body.insert(*index, quote!(
            r#"Object.defineProperty(exports, "__esModule", { value: true });"# as ModuleItem
        ))
    }

    fn append_void_stmt(&mut self, index: &usize, ident: &Ident) {
        fn create_void_stmt(ident: &Ident) -> ModuleItem {
            quote!(
                "exports.$identifier = void 0;" as ModuleItem,
                identifier = ident.clone()
            )
        }

        fn create_void_stmt_as_expr(ident: &Ident) -> Box<Expr> {
            quote_expr!(
                "exports.$identifier = void 0;",
                identifier = ident.clone()
            )
        }

        if !self.has_void_stmt {

            self.updated_body.insert(
                *index,
                create_void_stmt(ident)
            );

            self.has_void_stmt = true;

            return;
        }

        if let ModuleItem::Stmt(Stmt::Expr(ref mut expr_stmt)) = self.updated_body[*index] {
            if let Expr::Assign(ref mut assign_expr) = &mut *expr_stmt.expr {
                let mut previous_assign_expr = assign_expr;

                loop {
                    if let Expr::Assign(ref mut next_assign_expr) = *previous_assign_expr.right {
                        previous_assign_expr = next_assign_expr;
                    } else {
                        previous_assign_expr.right = create_void_stmt_as_expr(ident);
                        break;
                    }
                }
            }
        }
    }

    fn remove_by_index(&mut self, index: &usize) {
        self.updated_body.remove(*index);
    }

    fn remove_by_node(&mut self, node: &ModuleItem) {
        self.updated_body
            .retain(|body_node| {
                body_node == node
            })
    }
}

impl VisitMut for PluginModuleExports {
    fn visit_mut_module(&mut self, module: &mut Module) {
        self.updated_body = module.body.clone();

        module.visit_mut_children_with(self);

        module.body = self.updated_body.clone();
    }

    fn visit_mut_export_decl(&mut self, export_decl: &mut ExportDecl) {
        let node_from_module = &ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl.clone()));
        let mut export_decl_index = self.get_index(node_from_module);

        if !self.has_module_exports_stmt {
            self.increment(&mut export_decl_index);
            self.append_module_exports_stmt(&PROGRAM_TOP_INDEX);
        }

        if !self.has_void_stmt {
            self.increment(&mut export_decl_index);
        }

        match &mut export_decl.decl {
            _ => {},
            
            Decl::Class(class_decl) => {
                self.append_void_stmt(
                    &PROGRAM_VOID_STMT_INDEX,
                    &class_decl.ident
                )
            }

            Decl::Fn(func_decl) => {
                self.append_void_stmt(
                    &PROGRAM_VOID_STMT_INDEX,
                    &func_decl.ident
                )
            }

            Decl::Var(var_decl) => {
                for decl in &mut var_decl.decls {
                    if let Pat::Ident(ref binding_ident) = decl.name {
                        let ident = &binding_ident.id;

                        self.append_void_stmt(
                            &PROGRAM_VOID_STMT_INDEX,
                            ident
                        );

                        if let Option::Some(init) = &decl.init {
                            pub struct VarStmt {
                                
                            }

                            fn create_var_stmt<T>(kind: &VarDeclKind, node: &ModuleItem, value: &T) where T: AstNode {
                                match kind {
                                    VarDeclKind::Var => {

                                    },

                                    VarDeclKind::Let => {

                                    },

                                    VarDeclKind::Const => {

                                    }
                                }
                            }

                            let var_decl_clone = decl.clone();

                            match &**init {
                                _ => {},

                                Expr::Class(class_expr) => {
                                    create_var_stmt(
                                        &var_decl.kind,
                                        &quote!(
                                            "let teste;" as ModuleItem
                                        ),
                                        class_expr
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }

        self.remove_by_index(&export_decl_index);
    }
}

#[plugin_transform]
pub fn module_exports_transform(mut program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.visit_mut_with(&mut create_plugin_module_exports());
    program
}
